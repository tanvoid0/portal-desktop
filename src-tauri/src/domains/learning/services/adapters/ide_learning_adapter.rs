use async_trait::async_trait;
use serde_json::Value;
use crate::domains::learning::services::adapters::LearningAdapter;
use crate::domains::learning::services::pattern_matcher::Pattern;

/// Learning adapter for IDE domain
pub struct IDELearningAdapter;

#[async_trait]
impl LearningAdapter for IDELearningAdapter {
    fn domain_name(&self) -> &str {
        "ide"
    }

    async fn collect_patterns(&self, context: String) -> Vec<Pattern> {
        let _ = context; // Suppress unused warnings
        // Extract IDE preference patterns per project type
        vec![]
    }

    async fn generate_suggestions(&self, context: String, pattern_type: Option<String>) -> Vec<Value> {
        let _ = (context, pattern_type); // Suppress unused warnings
        // Generate IDE suggestions based on project type
        vec![]
    }

    async fn record_event(&self, event_type: String, event_data: Value, context: Option<String>) -> Result<(), String> {
        let _ = (event_type, event_data, context); // Suppress unused warnings
        // Record IDE preference events
        Ok(())
    }
}
