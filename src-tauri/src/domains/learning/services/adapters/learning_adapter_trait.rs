use async_trait::async_trait;
use serde_json::Value;
use crate::domains::learning::services::pattern_matcher::Pattern;

/// Trait for domain-specific learning adapters
/// This allows different domains (Projects, SDK, Terminal, etc.) to contribute
/// to the learning system in an extensible way
#[async_trait]
pub trait LearningAdapter: Send + Sync {
    /// Domain name (e.g., "projects", "sdk", "terminal")
    fn domain_name(&self) -> &str;

    /// Collect patterns from this domain for the given context
    async fn collect_patterns(&self, context: String) -> Vec<Pattern>;

    /// Generate suggestions based on learned patterns and context
    async fn generate_suggestions(&self, context: String, pattern_type: Option<String>) -> Vec<Value>;

    /// Record a domain-specific event for learning
    async fn record_event(&self, event_type: String, event_data: Value, context: Option<String>) -> Result<(), String>;
}
