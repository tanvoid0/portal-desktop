use async_trait::async_trait;
use serde_json::Value;
use crate::domains::learning::services::adapters::LearningAdapter;
use crate::domains::learning::services::pattern_matcher::Pattern;

/// Learning adapter for Projects domain
pub struct ProjectLearningAdapter;

#[async_trait]
impl LearningAdapter for ProjectLearningAdapter {
    fn domain_name(&self) -> &str {
        "projects"
    }

    async fn collect_patterns(&self, context: String) -> Vec<Pattern> {
        // Extract project patterns from context
        // Context format: "fw_<framework>_pm_<package_manager>_proj_<project_name>"
        let parts: Vec<&str> = context.split('_').collect();
        
        let mut patterns = Vec::new();
        
        // Extract framework pattern
        if let Some(fw_idx) = parts.iter().position(|&p| p == "fw") {
            if let Some(fw_name) = parts.get(fw_idx + 1) {
                patterns.push(Pattern {
                    pattern_type: "project_framework".to_string(),
                    pattern_data: serde_json::json!({ "framework": fw_name }),
                    context: context.to_string(),
                    frequency: 1,
                });
            }
        }

        // Extract package manager pattern
        if let Some(pm_idx) = parts.iter().position(|&p| p == "pm") {
            if let Some(pm_name) = parts.get(pm_idx + 1) {
                patterns.push(Pattern {
                    pattern_type: "project_package_manager".to_string(),
                    pattern_data: serde_json::json!({ "package_manager": pm_name }),
                    context: context.to_string(),
                    frequency: 1,
                });
            }
        }

        patterns
    }

    async fn generate_suggestions(&self, context: String, pattern_type: Option<String>) -> Vec<Value> {
        let _ = (context, pattern_type); // Suppress unused warnings
        // Generate project-specific suggestions
        // This would typically query learned patterns and preferences
        vec![]
    }

    async fn record_event(&self, event_type: String, event_data: Value, context: Option<String>) -> Result<(), String> {
        let _ = (event_type, event_data, context); // Suppress unused warnings
        // Record project-related learning events
        // This would typically call the learning service
        Ok(())
    }
}
