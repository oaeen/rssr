use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

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
}
