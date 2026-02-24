use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeedFormat {
    XmlFeed,
    JsonFeed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedEntry {
    pub id: String,
    pub title: String,
    pub link: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedFeed {
    pub format: FeedFormat,
    pub title: String,
    pub home_page_url: Option<String>,
    pub feed_url: Option<String>,
    pub entries: Vec<ParsedEntry>,
}
