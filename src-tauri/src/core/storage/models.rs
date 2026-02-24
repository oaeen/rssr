use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSource {
    pub title: String,
    pub site_url: Option<String>,
    pub feed_url: String,
    pub category: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SourceRecord {
    pub id: i64,
    pub title: String,
    pub site_url: Option<String>,
    pub feed_url: String,
    pub category: Option<String>,
    pub is_active: i64,
    pub failure_count: i64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntryRecord {
    pub id: i64,
    pub source_id: i64,
    pub source_title: String,
    pub guid: Option<String>,
    pub link: String,
    pub title: String,
    pub translated_title: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub published_at: Option<String>,
    pub is_read: i64,
    pub is_starred: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntryTitleRecord {
    pub id: i64,
    pub title: String,
}
