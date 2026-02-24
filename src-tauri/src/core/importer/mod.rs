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
