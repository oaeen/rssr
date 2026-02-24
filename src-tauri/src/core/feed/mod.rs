pub mod fetcher;
pub mod parser;
pub mod types;

#[derive(Debug, Clone, Default)]
pub struct FeedService;

impl FeedService {
    pub fn name(&self) -> &'static str {
        "feed"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}
