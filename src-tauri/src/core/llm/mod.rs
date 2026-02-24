#[derive(Debug, Clone, Default)]
pub struct LlmService;

impl LlmService {
    pub fn name(&self) -> &'static str {
        "llm"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}
