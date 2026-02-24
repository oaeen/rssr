#[derive(Debug, Clone, Default)]
pub struct StorageService;

impl StorageService {
    pub fn name(&self) -> &'static str {
        "storage"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}
