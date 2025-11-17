use serde_json::Value;

pub fn resolve_dependencies(steps: &[Value]) -> Vec<Vec<String>> {
    // Simplified dependency resolution
    // In a real implementation, this would do topological sorting
    vec![steps.iter().map(|s| s["id"].as_str().unwrap().to_string()).collect()]
}

