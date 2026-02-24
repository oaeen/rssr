#[derive(Debug, Clone, Default)]
pub struct SubscriptionService;

impl SubscriptionService {
    pub fn name(&self) -> &'static str {
        "subscription"
    }

    pub fn status(&self) -> &'static str {
        "ready"
    }
}
