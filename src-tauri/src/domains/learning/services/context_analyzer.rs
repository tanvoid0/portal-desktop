use serde_json::{json, Value};

/// Project context understanding service
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    /// Extract context from project information
    pub fn extract_context(
        framework: Option<&str>,
        package_manager: Option<&str>,
        _project_path: Option<&str>,
    ) -> String {
        let mut context_parts = Vec::new();

        if let Some(fw) = framework {
            context_parts.push(fw.to_lowercase().replace(" ", "_"));
        }

        if let Some(pm) = package_manager {
            context_parts.push(format!("pm_{}", pm.to_lowercase()));
        }

        if context_parts.is_empty() {
            "global".to_string()
        } else {
            context_parts.join("_")
        }
    }

    /// Build context metadata as JSON
    pub fn build_context_metadata(
        framework: Option<&str>,
        package_manager: Option<&str>,
        project_path: Option<&str>,
        additional_data: Option<&Value>,
    ) -> Value {
        let mut metadata = json!({
            "framework": framework,
            "package_manager": package_manager,
            "project_path": project_path,
        });

        if let Some(additional) = additional_data {
            if let Some(obj) = metadata.as_object_mut() {
                if let Some(additional_obj) = additional.as_object() {
                    for (key, value) in additional_obj {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        metadata
    }

    /// Determine if contexts match (fuzzy matching)
    pub fn contexts_match(context1: &str, context2: &str) -> bool {
        if context1 == context2 {
            return true;
        }

        // Check if one context contains the other (for partial matches)
        if context1.contains(context2) || context2.contains(context1) {
            return true;
        }

        // Extract components for comparison
        let parts1: Vec<&str> = context1.split('_').collect();
        let parts2: Vec<&str> = context2.split('_').collect();

        // Check if they share significant components (at least 50%)
        let common_parts: usize = parts1
            .iter()
            .filter(|&part| parts2.contains(part))
            .count();
        
        let similarity = if parts1.len().max(parts2.len()) > 0 {
            common_parts as f64 / parts1.len().max(parts2.len()) as f64
        } else {
            0.0
        };

        similarity >= 0.5
    }

    /// Get context hierarchy (more specific to less specific)
    pub fn get_context_hierarchy(context: &str) -> Vec<String> {
        let parts: Vec<&str> = context.split('_').collect();
        let mut hierarchy = vec![context.to_string()]; // Most specific

        // Generate less specific contexts by removing parts
        for i in 1..parts.len() {
            let less_specific = parts[..parts.len() - i].join("_");
            if !less_specific.is_empty() {
                hierarchy.push(less_specific);
            }
        }

        hierarchy.push("global".to_string()); // Least specific
        hierarchy
    }
}

