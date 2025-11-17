use std::collections::HashMap;

/// Context manager for understanding and managing learning contexts
pub struct ContextManager;

impl ContextManager {
    /// Build comprehensive context from available information
    pub fn build_context(
        framework: Option<&str>,
        package_manager: Option<&str>,
        project_path: Option<&str>,
        sdk_type: Option<&str>,
        additional: Option<&HashMap<String, String>>,
    ) -> String {
        let mut parts = Vec::new();

        if let Some(fw) = framework {
            parts.push(format!("fw_{}", fw.to_lowercase().replace(' ', "_")));
        }

        if let Some(pm) = package_manager {
            parts.push(format!("pm_{}", pm.to_lowercase()));
        }

        if let Some(path) = project_path {
            // Extract project identifier from path (last directory name)
            if let Some(name) = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                parts.push(format!("proj_{}", name.to_lowercase().replace(' ', "_")));
            }
        }

        if let Some(sdk) = sdk_type {
            parts.push(format!("sdk_{}", sdk.to_lowercase()));
        }

        if let Some(additional) = additional {
            for (key, value) in additional {
                parts.push(format!("{}_{}", key.to_lowercase(), value.to_lowercase().replace(' ', "_")));
            }
        }

        if parts.is_empty() {
            "global".to_string()
        } else {
            parts.join("_")
        }
    }

    /// Extract context hierarchy (most specific to least specific)
    pub fn get_context_hierarchy(context: &str) -> Vec<String> {
        let parts: Vec<&str> = context.split('_').collect();
        let mut hierarchy = Vec::new();

        // Build increasingly general contexts
        for i in 1..=parts.len() {
            let partial: Vec<&str> = parts.iter().take(i).copied().collect();
            hierarchy.push(partial.join("_"));
        }

        // Always include global as fallback
        if !hierarchy.contains(&"global".to_string()) {
            hierarchy.push("global".to_string());
        }

        hierarchy
    }

    /// Check if two contexts match (exact or hierarchical)
    pub fn contexts_match(context1: &str, context2: &str) -> bool {
        if context1 == context2 {
            return true;
        }

        // Check if one is a parent of the other
        let hierarchy1 = Self::get_context_hierarchy(context1);
        let hierarchy2 = Self::get_context_hierarchy(context2);

        for ctx in &hierarchy1 {
            if hierarchy2.contains(ctx) {
                return true;
            }
        }

        false
    }

    /// Get the best matching context from a list
    pub fn find_best_match(
        target_context: &str,
        available_contexts: &[String],
    ) -> Option<String> {
        let target_hierarchy = Self::get_context_hierarchy(target_context);

        // Find most specific match
        for ctx in &target_hierarchy {
            if let Some(matched) = available_contexts.iter().find(|c| {
                *c == ctx || Self::contexts_match(ctx, c)
            }) {
                return Some(matched.clone());
            }
        }

        None
    }

    /// Merge contexts (create parent context)
    pub fn merge_contexts(contexts: &[String]) -> String {
        if contexts.is_empty() {
            return "global".to_string();
        }

        // Find common prefix
        let mut common_parts = Vec::new();
        let first_parts: Vec<&str> = contexts[0].split('_').collect();

        for (i, part) in first_parts.iter().enumerate() {
            let mut all_match = true;
            for ctx in contexts.iter().skip(1) {
                let parts: Vec<&str> = ctx.split('_').collect();
                if i >= parts.len() || parts[i] != *part {
                    all_match = false;
                    break;
                }
            }

            if all_match {
                common_parts.push(*part);
            } else {
                break;
            }
        }

        if common_parts.is_empty() {
            "global".to_string()
        } else {
            common_parts.join("_")
        }
    }

    /// Extract context metadata for analytics
    pub fn extract_metadata(context: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let parts: Vec<&str> = context.split('_').collect();

        for part in &parts {
            if part.starts_with("fw_") {
                if let Some(framework) = part.strip_prefix("fw_") {
                    metadata.insert("framework".to_string(), framework.to_string());
                }
            } else if part.starts_with("pm_") {
                if let Some(package_manager) = part.strip_prefix("pm_") {
                    metadata.insert("package_manager".to_string(), package_manager.to_string());
                }
            } else if part.starts_with("proj_") {
                if let Some(project) = part.strip_prefix("proj_") {
                    metadata.insert("project".to_string(), project.to_string());
                }
            } else if part.starts_with("sdk_") {
                if let Some(sdk_type) = part.strip_prefix("sdk_") {
                    metadata.insert("sdk_type".to_string(), sdk_type.to_string());
                }
            }
        }

        metadata.insert("specificity".to_string(), parts.len().to_string());
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_context() {
        let ctx = ContextManager::build_context(
            Some("React"),
            Some("npm"),
            None,
            None,
            None,
        );
        assert_eq!(ctx, "fw_react_pm_npm");
    }

    #[test]
    fn test_context_hierarchy() {
        let hierarchy = ContextManager::get_context_hierarchy("fw_react_pm_npm");
        assert!(hierarchy.contains(&"fw_react".to_string()));
        assert!(hierarchy.contains(&"fw_react_pm_npm".to_string()));
        assert!(hierarchy.contains(&"global".to_string()));
    }

    #[test]
    fn test_contexts_match() {
        assert!(ContextManager::contexts_match("fw_react_pm_npm", "fw_react_pm_npm"));
        assert!(ContextManager::contexts_match("fw_react", "fw_react_pm_npm"));
    }
}

