use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportSource {
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportPreview {
    pub new_sources: Vec<ImportSource>,
    pub duplicate_sources: Vec<ImportSource>,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("invalid OPML content: {0}")]
    Opml(String),
    #[error("invalid JSON import format: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default)]
pub struct ImporterService;

impl ImporterService {
    pub fn name(&self) -> &'static str {
        "importer"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum JsonImportItem {
    Url(String),
    Object {
        feed_url: String,
        title: Option<String>,
        site_url: Option<String>,
        category: Option<String>,
    },
}

pub fn parse_opml(opml_content: &str) -> Result<Vec<ImportSource>, ImportError> {
    let doc = roxmltree::Document::parse(opml_content)
        .map_err(|error| ImportError::Opml(error.to_string()))?;
    let mut results = Vec::new();

    for node in doc.descendants().filter(|node| node.has_tag_name("outline")) {
        let Some(feed_url) = node.attribute("xmlUrl") else {
            continue;
        };
        if feed_url.trim().is_empty() {
            continue;
        }

        let title = node
            .attribute("title")
            .or_else(|| node.attribute("text"))
            .unwrap_or(feed_url)
            .to_string();
        let category = node
            .attribute("category")
            .map(ToString::to_string)
            .or_else(|| infer_opml_category(node));
        let source = ImportSource {
            title,
            feed_url: feed_url.to_string(),
            site_url: node.attribute("htmlUrl").map(ToString::to_string),
            category,
        };
        results.push(source);
    }

    Ok(results)
}

pub fn parse_url_list(input: &str) -> Vec<ImportSource> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| line.starts_with("http://") || line.starts_with("https://"))
        .map(|line| ImportSource {
            title: line.to_string(),
            feed_url: line.to_string(),
            site_url: None,
            category: None,
        })
        .collect()
}

pub fn parse_json_sources(input: &str) -> Result<Vec<ImportSource>, ImportError> {
    let items: Vec<JsonImportItem> = serde_json::from_str(input)?;
    let mut sources = Vec::with_capacity(items.len());

    for item in items {
        match item {
            JsonImportItem::Url(feed_url) => {
                sources.push(ImportSource {
                    title: feed_url.clone(),
                    feed_url,
                    site_url: None,
                    category: None,
                });
            }
            JsonImportItem::Object {
                feed_url,
                title,
                site_url,
                category,
            } => {
                sources.push(ImportSource {
                    title: title.unwrap_or_else(|| feed_url.clone()),
                    feed_url,
                    site_url,
                    category,
                });
            }
        }
    }

    Ok(sources)
}

pub fn build_import_preview(
    candidates: Vec<ImportSource>,
    existing_feed_urls: &HashSet<String>,
) -> ImportPreview {
    let mut seen = HashMap::<String, ImportSource>::new();
    let mut duplicate_sources = Vec::new();
    let mut new_sources = Vec::new();

    for source in candidates {
        let normalized = normalize_url(&source.feed_url);
        if normalized.is_empty() {
            continue;
        }

        if existing_feed_urls.contains(&normalized) {
            duplicate_sources.push(source);
            continue;
        }

        if let Some(existing) = seen.insert(normalized, source.clone()) {
            duplicate_sources.push(existing);
            duplicate_sources.push(source);
            continue;
        }

        new_sources.push(source);
    }

    ImportPreview {
        new_sources,
        duplicate_sources,
    }
}

pub fn normalize_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_lowercase()
}

fn infer_opml_category(node: roxmltree::Node<'_, '_>) -> Option<String> {
    for ancestor in node.ancestors() {
        if !ancestor.has_tag_name("outline") {
            continue;
        }
        if ancestor.attribute("xmlUrl").is_some() {
            continue;
        }
        if let Some(name) = ancestor
            .attribute("title")
            .or_else(|| ancestor.attribute("text"))
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_source_with_url(list: &[ImportSource], target: &str) -> bool {
        list.iter().any(|item| item.feed_url == target)
    }

    #[test]
    fn parses_real_opml_fixtures() {
        let a = include_str!("../../../../fixtures/import-samples/hn-popular-blogs-2025.opml");
        let b = include_str!("../../../../fixtures/import-samples/hackerNewsStars.xml");

        let first = parse_opml(a).expect("first opml should parse");
        let second = parse_opml(b).expect("second opml should parse");

        assert!(first.len() > 50);
        assert!(second.len() > 50);
        assert!(has_source_with_url(
            &first,
            "https://simonwillison.net/atom/everything/"
        ));
        assert!(has_source_with_url(
            &second,
            "https://keygen.sh/blog/feed.xml"
        ));
    }

    #[test]
    fn parses_url_list() {
        let input = r#"
            # comment
            https://example.com/feed.xml
            https://example.com/atom.xml
            not-a-url
        "#;
        let items = parse_url_list(input);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].feed_url, "https://example.com/feed.xml");
    }

    #[test]
    fn parses_json_sources_from_string_and_object() {
        let json = r#"
            [
              "https://example.com/feed.xml",
              {
                "feed_url": "https://blog.example.com/rss",
                "title": "Blog",
                "site_url": "https://blog.example.com",
                "category": "tech"
              }
            ]
        "#;

        let items = parse_json_sources(json).expect("json should parse");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "https://example.com/feed.xml");
        assert_eq!(items[1].title, "Blog");
    }

    #[test]
    fn preview_marks_existing_and_duplicate_sources() {
        let candidates = vec![
            ImportSource {
                title: "A".to_string(),
                feed_url: "https://example.com/feed.xml".to_string(),
                site_url: None,
                category: None,
            },
            ImportSource {
                title: "A duplicate".to_string(),
                feed_url: "https://example.com/feed.xml".to_string(),
                site_url: None,
                category: None,
            },
            ImportSource {
                title: "B".to_string(),
                feed_url: "https://another.com/feed.xml".to_string(),
                site_url: None,
                category: None,
            },
        ];
        let existing = HashSet::from([normalize_url("https://another.com/feed.xml")]);
        let preview = build_import_preview(candidates, &existing);

        assert_eq!(preview.new_sources.len(), 1);
        assert_eq!(preview.new_sources[0].title, "A");
        assert_eq!(preview.duplicate_sources.len(), 3);
    }
}
