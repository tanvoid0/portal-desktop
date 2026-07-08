use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Returns step IDs grouped in execution order (topological sort batches).
pub fn resolve_execution_order(steps: &[Value]) -> Result<Vec<Vec<String>>, String> {
    if steps.is_empty() {
        return Ok(vec![]);
    }

    let mut step_ids: Vec<String> = Vec::new();
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

    for step in steps {
        let id = step
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Pipeline step missing id".to_string())?
            .to_string();

        let deps: Vec<String> = step
            .get("dependsOn")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        step_ids.push(id.clone());
        dependencies.insert(id, deps);
    }

    let step_id_set: HashSet<String> = step_ids.iter().cloned().collect();
    for (step_id, deps) in &dependencies {
        for dep in deps {
            if !step_id_set.contains(dep) {
                return Err(format!("Step {} depends on missing step {}", step_id, dep));
            }
        }
    }

    let mut executed: HashSet<String> = HashSet::new();
    let mut groups: Vec<Vec<String>> = Vec::new();

    while executed.len() < step_ids.len() {
        let mut ready: Vec<String> = Vec::new();

        for step_id in &step_ids {
            if executed.contains(step_id) {
                continue;
            }
            let deps = dependencies.get(step_id).cloned().unwrap_or_default();
            if deps.iter().all(|d| executed.contains(d)) {
                ready.push(step_id.clone());
            }
        }

        if ready.is_empty() {
            return Err("Circular dependency detected in pipeline steps".to_string());
        }

        groups.push(ready.clone());
        for id in ready {
            executed.insert(id);
        }
    }

    Ok(groups)
}

#[allow(dead_code)]
pub fn resolve_dependencies(steps: &[Value]) -> Vec<Vec<String>> {
    resolve_execution_order(steps).unwrap_or_else(|_| {
        vec![steps
            .iter()
            .filter_map(|s| s.get("id").and_then(|v| v.as_str()).map(String::from))
            .collect()]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn orders_steps_by_dependencies() {
        let steps = vec![
            json!({ "id": "install", "dependsOn": [] }),
            json!({ "id": "build", "dependsOn": ["install"] }),
        ];
        let order = resolve_execution_order(&steps).unwrap();
        assert_eq!(order[0], vec!["install"]);
        assert_eq!(order[1], vec!["build"]);
    }
}
