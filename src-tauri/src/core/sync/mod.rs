#[derive(Debug, Clone, Default)]
pub struct SyncService;

impl SyncService {
    pub fn name(&self) -> &'static str {
        "sync"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}
