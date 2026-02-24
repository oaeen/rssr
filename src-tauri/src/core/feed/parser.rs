use feed_rs::model::Entry;
use serde::Deserialize;

use super::types::{FeedFormat, ParsedEntry, ParsedFeed};

#[derive(Debug, thiserror::Error)]
pub enum FeedParseError {
    #[error("feed payload is empty")]
    EmptyPayload,
    #[error("xml feed parse error: {0}")]
    Xml(#[from] feed_rs::parser::ParseFeedError),
    #[error("json feed parse error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Deserialize)]
struct JsonFeed {
    title: Option<String>,
    home_page_url: Option<String>,
    feed_url: Option<String>,
    #[serde(default)]
    items: Vec<JsonFeedItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonFeedItem {
    id: Option<String>,
    title: Option<String>,
    url: Option<String>,
    summary: Option<String>,
    content_text: Option<String>,
    content_html: Option<String>,
    date_published: Option<String>,
}

pub fn parse_feed_bytes(raw: &[u8]) -> Result<ParsedFeed, FeedParseError> {
    let trimmed = trim_leading_ascii_whitespace(raw);
    if trimmed.is_empty() {
        return Err(FeedParseError::EmptyPayload);
    }
    if trimmed[0] == b'{' {
        return parse_json_feed(trimmed);
    }
    parse_xml_feed(trimmed)
}

pub fn build_dedup_key(feed_url: &str, entry: &ParsedEntry) -> String {
    if !entry.id.trim().is_empty() {
        return format!("{feed_url}::id::{}", entry.id.trim());
    }
    if !entry.link.trim().is_empty() {
        return format!("{feed_url}::link::{}", entry.link.trim());
    }
    format!(
        "{feed_url}::fallback::{}::{}",
        entry.title.trim(),
        entry.published_at.as_deref().unwrap_or_default()
    )
}

fn parse_xml_feed(raw: &[u8]) -> Result<ParsedFeed, FeedParseError> {
    let feed = feed_rs::parser::parse(raw)?;
    let title = feed
        .title
        .as_ref()
        .map(|text| text.content.clone())
        .unwrap_or_else(|| "Untitled Feed".to_string());
    let home_page_url = feed.links.first().map(|link| link.href.clone());
    let entries = feed.entries.iter().map(entry_from_xml).collect();

    Ok(ParsedFeed {
        format: FeedFormat::XmlFeed,
        title,
        home_page_url,
        feed_url: None,
        entries,
    })
}

fn parse_json_feed(raw: &[u8]) -> Result<ParsedFeed, FeedParseError> {
    let feed: JsonFeed = serde_json::from_slice(raw)?;
    let title = feed.title.unwrap_or_else(|| "Untitled Feed".to_string());
    let entries = feed
        .items
        .into_iter()
        .map(|item| ParsedEntry {
            id: item
                .id
                .or_else(|| item.url.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            title: item.title.unwrap_or_else(|| "Untitled Entry".to_string()),
            link: item.url.unwrap_or_default(),
            summary: item.summary,
            content: item.content_html.or(item.content_text),
            published_at: item.date_published,
        })
        .collect();

    Ok(ParsedFeed {
        format: FeedFormat::JsonFeed,
        title,
        home_page_url: feed.home_page_url,
        feed_url: feed.feed_url,
        entries,
    })
}

fn entry_from_xml(entry: &Entry) -> ParsedEntry {
    let id = if entry.id.trim().is_empty() {
        entry
            .links
            .first()
            .map(|link| link.href.clone())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        entry.id.clone()
    };
    let title = entry
        .title
        .as_ref()
        .map(|text| text.content.clone())
        .unwrap_or_else(|| "Untitled Entry".to_string());
    let link = entry
        .links
        .first()
        .map(|entry_link| entry_link.href.clone())
        .unwrap_or_default();
    let summary = entry.summary.as_ref().map(|text| text.content.clone());
    let content = entry
        .content
        .as_ref()
        .and_then(|content| content.body.clone());
    let published_at = entry
        .published
        .or(entry.updated)
        .map(|timestamp| timestamp.to_rfc3339());

    ParsedEntry {
        id,
        title,
        link,
        summary,
        content,
        published_at,
    }
}

fn trim_leading_ascii_whitespace(raw: &[u8]) -> &[u8] {
    let mut index = 0;
    while index < raw.len() && raw[index].is_ascii_whitespace() {
        index += 1;
    }
    &raw[index..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xml_fixture_feed() {
        let xml = include_bytes!("../../../../fixtures/import-samples/sample.rss.xml");
        let parsed = parse_feed_bytes(xml).expect("xml fixture must parse");

        assert_eq!(parsed.format, FeedFormat::XmlFeed);
        assert!(!parsed.title.trim().is_empty());
        assert_eq!(parsed.entries.len(), 2);
    }

    #[test]
    fn parses_json_feed() {
        let json = include_bytes!("../../../../fixtures/import-samples/sample.jsonfeed.json");
        let parsed = parse_feed_bytes(json).expect("json feed must parse");

        assert_eq!(parsed.format, FeedFormat::JsonFeed);
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].title, "First entry");
    }

    #[test]
    fn dedup_key_prefers_entry_id() {
        let entry = ParsedEntry {
            id: "entry-1".to_string(),
            title: "Title".to_string(),
            link: "https://example.com/entry".to_string(),
            summary: None,
            content: None,
            published_at: Some("2026-02-24T00:00:00Z".to_string()),
        };
        let key = build_dedup_key("https://example.com/feed.xml", &entry);
        assert_eq!(key, "https://example.com/feed.xml::id::entry-1");
    }
}
