use async_trait::async_trait;
use serde_json::Value;
use crate::domains::learning::services::adapters::LearningAdapter;
use crate::domains::learning::services::pattern_matcher::Pattern;

/// Learning adapter for Terminal domain
pub struct TerminalLearningAdapter;

#[async_trait]
impl LearningAdapter for TerminalLearningAdapter {
    fn domain_name(&self) -> &str {
        "terminal"
    }

    async fn collect_patterns(&self, context: String) -> Vec<Pattern> {
        // Extract command sequence patterns
        // Context format: "cmd_<command_pattern>_project_<project_type>"
        let mut patterns = Vec::new();

        if context.contains("cmd_") {
            patterns.push(Pattern {
                pattern_type: "command_sequence".to_string(),
                pattern_data: serde_json::json!({ "command": context }),
                context: context.to_string(),
                frequency: 1,
            });
        }

        patterns
    }

    async fn generate_suggestions(&self, context: String, pattern_type: Option<String>) -> Vec<Value> {
        let _ = (context, pattern_type); // Suppress unused warnings
        // Generate command completion suggestions
        vec![]
    }

    async fn record_event(&self, event_type: String, event_data: Value, context: Option<String>) -> Result<(), String> {
        let _ = (event_type, event_data, context); // Suppress unused warnings
        // Record terminal command events
        Ok(())
    }
}
