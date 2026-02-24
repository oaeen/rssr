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
    pub created_at: String,
    pub updated_at: String,
}
