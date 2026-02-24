use sqlx::{sqlite::SqlitePoolOptions, QueryBuilder, Sqlite, SqlitePool};

use super::models::{NewSource, SourceRecord};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

#[derive(Debug, Clone)]
pub struct SourceRepository {
    pool: SqlitePool,
}

impl SourceRepository {
    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn upsert_source(&self, source: &NewSource) -> Result<SourceRecord, StorageError> {
        sqlx::query(
            r#"
            INSERT INTO sources (title, site_url, feed_url, category, is_active)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(feed_url) DO UPDATE SET
              title = excluded.title,
              site_url = excluded.site_url,
              category = excluded.category,
              is_active = excluded.is_active,
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&source.title)
        .bind(&source.site_url)
        .bind(&source.feed_url)
        .bind(&source.category)
        .bind(i64::from(source.is_active))
        .execute(&self.pool)
        .await?;

        let record = sqlx::query_as::<_, SourceRecord>(
            r#"
            SELECT id, title, site_url, feed_url, category, is_active, failure_count, created_at, updated_at
            FROM sources
            WHERE feed_url = ?1
            "#,
        )
        .bind(&source.feed_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list_sources(&self) -> Result<Vec<SourceRecord>, StorageError> {
        let rows = sqlx::query_as::<_, SourceRecord>(
            r#"
            SELECT id, title, site_url, feed_url, category, is_active, failure_count, created_at, updated_at
            FROM sources
            ORDER BY id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_source(&self, id: i64) -> Result<u64, StorageError> {
        let affected = sqlx::query("DELETE FROM sources WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected)
    }

    pub async fn upsert_sources_batch(&self, sources: &[NewSource]) -> Result<usize, StorageError> {
        let mut inserted = 0_usize;
        for source in sources {
            self.upsert_source(source).await?;
            inserted += 1;
        }
        Ok(inserted)
    }

    pub async fn set_sources_active(
        &self,
        source_ids: &[i64],
        is_active: bool,
    ) -> Result<u64, StorageError> {
        if source_ids.is_empty() {
            return Ok(0);
        }

        let mut query = QueryBuilder::<Sqlite>::new(
            "UPDATE sources SET is_active = ",
        );
        query.push_bind(i64::from(is_active));
        query.push(", updated_at = CURRENT_TIMESTAMP WHERE id IN (");
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(*source_id);
        }
        separated.push_unseparated(")");

        let affected = query.build().execute(&self.pool).await?.rows_affected();
        Ok(affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::importer::{build_import_preview, parse_opml};
    use sqlx::Row;
    use std::collections::HashSet;

    fn make_source(title: &str, feed_url: &str) -> NewSource {
        NewSource {
            title: title.to_string(),
            site_url: Some("https://example.com".to_string()),
            feed_url: feed_url.to_string(),
            category: Some("tech".to_string()),
            is_active: true,
        }
    }

    #[tokio::test]
    async fn migration_creates_required_tables() {
        let repository = SourceRepository::connect("sqlite::memory:")
            .await
            .expect("connect must succeed");
        let rows = sqlx::query(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE type = 'table'
              AND name IN ('sources', 'entries', 'llm_cache')
            ORDER BY name
            "#,
        )
        .fetch_all(&repository.pool)
        .await
        .expect("query must succeed");

        let table_names: Vec<String> = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();
        assert_eq!(
            table_names,
            vec![
                "entries".to_string(),
                "llm_cache".to_string(),
                "sources".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn upsert_source_is_idempotent_for_same_feed_url() {
        let repository = SourceRepository::connect("sqlite::memory:")
            .await
            .expect("connect must succeed");
        let first = repository
            .upsert_source(&make_source("Hacker News", "https://news.ycombinator.com/rss"))
            .await
            .expect("first upsert must succeed");

        let second = repository
            .upsert_source(&make_source("HN Updated", "https://news.ycombinator.com/rss"))
            .await
            .expect("second upsert must succeed");

        let all = repository
            .list_sources()
            .await
            .expect("list must succeed");

        assert_eq!(all.len(), 1);
        assert_eq!(first.id, second.id);
        assert_eq!(all[0].title, "HN Updated");
    }

    #[tokio::test]
    async fn delete_source_removes_row() {
        let repository = SourceRepository::connect("sqlite::memory:")
            .await
            .expect("connect must succeed");
        let created = repository
            .upsert_source(&make_source("Rust Blog", "https://blog.rust-lang.org/feed.xml"))
            .await
            .expect("create must succeed");

        let affected = repository
            .delete_source(created.id)
            .await
            .expect("delete must succeed");
        let all = repository
            .list_sources()
            .await
            .expect("list must succeed");

        assert_eq!(affected, 1);
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn set_sources_active_updates_batch_rows() {
        let repository = SourceRepository::connect("sqlite::memory:")
            .await
            .expect("connect must succeed");
        let first = repository
            .upsert_source(&make_source("A", "https://a.com/feed.xml"))
            .await
            .expect("create A");
        let second = repository
            .upsert_source(&make_source("B", "https://b.com/feed.xml"))
            .await
            .expect("create B");

        let affected = repository
            .set_sources_active(&[first.id, second.id], false)
            .await
            .expect("batch update should succeed");
        let rows = repository.list_sources().await.expect("list should succeed");

        assert_eq!(affected, 2);
        assert!(rows.iter().all(|row| row.is_active == 0));
    }

    #[tokio::test]
    async fn e2e_import_then_delete_flow() {
        let repository = SourceRepository::connect("sqlite::memory:")
            .await
            .expect("connect must succeed");
        let opml = include_str!("../../../../fixtures/import-samples/hackerNewsStars.xml");
        let parsed_sources = parse_opml(opml).expect("opml parse should succeed");
        let preview = build_import_preview(parsed_sources, &HashSet::new());
        let batch: Vec<NewSource> = preview
            .new_sources
            .into_iter()
            .take(5)
            .map(|source| NewSource {
                title: source.title,
                site_url: source.site_url,
                feed_url: source.feed_url,
                category: source.category,
                is_active: true,
            })
            .collect();

        repository
            .upsert_sources_batch(&batch)
            .await
            .expect("batch upsert should succeed");
        let current = repository.list_sources().await.expect("list should succeed");
        let deleted = repository
            .delete_source(current[0].id)
            .await
            .expect("delete should succeed");
        let after_delete = repository.list_sources().await.expect("list should succeed");

        assert_eq!(current.len(), 5);
        assert_eq!(deleted, 1);
        assert_eq!(after_delete.len(), 4);
    }
}
