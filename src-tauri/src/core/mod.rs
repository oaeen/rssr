pub mod feed;
pub mod importer;
pub mod llm;
pub mod storage;
pub mod subscription;
pub mod sync;

use std::collections::BTreeMap;

use feed::FeedService;
use importer::ImporterService;
use llm::LlmService;
use storage::StorageService;
use subscription::SubscriptionService;
use sync::SyncService;

#[derive(Debug, Clone, Default)]
pub struct AppServices {
    feed: FeedService,
    importer: ImporterService,
    subscription: SubscriptionService,
    llm: LlmService,
    storage: StorageService,
    sync: SyncService,
}

impl AppServices {
    pub fn health_report(&self) -> BTreeMap<String, String> {
        let mut report = BTreeMap::new();
        report.insert(self.feed.name().to_string(), self.feed.status().to_string());
        report.insert(
            self.importer.name().to_string(),
            self.importer.status().to_string(),
        );
        report.insert(
            self.subscription.name().to_string(),
            self.subscription.status().to_string(),
        );
        report.insert(self.llm.name().to_string(), self.llm.status().to_string());
        report.insert(
            self.storage.name().to_string(),
            self.storage.status().to_string(),
        );
        report.insert(self.sync.name().to_string(), self.sync.status().to_string());
        report
    }
}
