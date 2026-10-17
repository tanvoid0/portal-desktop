//! `.portal/pipeline.yml`, read from Rust so a workflow can run without the webview.
//!
//! The same file the frontend's `pipelineFile.ts` reads (`actions`, `workflows`),
//! plus a `release:` block that names which workflow ships and which one
//! rehearses it. See `docs/development/ACTIONS.md`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const CANDIDATES: [&str; 3] = [
    ".portal/pipeline.yml",
    ".portal/pipeline.yaml",
    ".portal/pipeline.json",
];

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PipelineFile {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub actions: BTreeMap<String, FileAction>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workflows: BTreeMap<String, FileWorkflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseSpec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub run: String,
    /// Relative to the project root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default, rename = "longRunning", skip_serializing_if = "std::ops::Not::not")]
    pub long_running: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileWorkflow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub steps: Vec<FileStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileStep {
    pub action: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
}

/// What makes a workflow a release: which one ships, which one rehearses it.
/// `kind` is a label for templates and the UI; nothing branches on it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Workflow id that really deploys.
    pub ship: String,
    /// Workflow id run by default. Omitted: `ship` runs with `RELEASE_DRY_RUN=1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rehearse: Option<String>,
    /// Exposed to steps as `${BUMP}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump: Option<String>,
    /// Applied to every step; a step's own `env` wins.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

pub fn parse(raw: &str) -> Result<PipelineFile, String> {
    // JSON is YAML, so one parser covers all three file names.
    let file: PipelineFile =
        serde_yaml::from_str(raw).map_err(|e| format!("Invalid pipeline file: {e}"))?;
    if let Some(r) = &file.release {
        for id in std::iter::once(&r.ship).chain(r.rehearse.iter()) {
            if !file.workflows.contains_key(id) {
                return Err(format!("release names workflow '{id}', which is not defined"));
            }
        }
    }
    Ok(file)
}

/// The file as YAML text, with the `version: 1` the frontend requires.
pub fn to_yaml(file: &PipelineFile) -> Result<String, String> {
    let body = serde_yaml::to_string(file).map_err(|e| e.to_string())?;
    Ok(format!("version: 1
{body}"))
}

/// `Ok(None)` when the project has no pipeline file.
pub fn load(project_path: &str) -> Result<Option<PipelineFile>, String> {
    for rel in CANDIDATES {
        let path = Path::new(project_path).join(rel);
        if path.is_file() {
            let raw = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            return parse(&raw).map(Some);
        }
    }
    Ok(None)
}

/// A workflow as the runner's `steps_json` shape: `{id, name, dependsOn, config}`.
pub fn build_steps(file: &PipelineFile, workflow_id: &str) -> Result<Vec<Value>, String> {
    let wf = file
        .workflows
        .get(workflow_id)
        .ok_or_else(|| format!("No workflow '{workflow_id}'"))?;
    let release_env = file.release.as_ref().map(|r| &r.env);

    wf.steps
        .iter()
        .map(|step| {
            let action = file
                .actions
                .get(&step.action)
                .ok_or_else(|| format!("Workflow '{workflow_id}' uses undefined action '{}'", step.action))?;
            let mut env: HashMap<&String, &String> = release_env.into_iter().flatten().collect();
            env.extend(action.env.iter());
            Ok(json!({
                "id": step.action,
                "name": action.name.clone().unwrap_or_else(|| step.action.clone()),
                "dependsOn": step.needs,
                "config": {
                    "command": action.run,
                    "cwd": action.cwd,
                    "env": env,
                    "longRunning": action.long_running,
                },
            }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const YML: &str = r#"
release: { ship: ship, rehearse: ship-dry, env: { A: "1", B: "1" } }
actions:
  gate: { run: echo gate }
  go: { run: echo go, cwd: server, env: { B: "2" } }
workflows:
  ship: { steps: [ { action: gate }, { action: go, needs: [gate] } ] }
  ship-dry: { steps: [ { action: gate } ] }
"#;

    #[test]
    fn builds_steps_with_deps_cwd_and_env_precedence() {
        let file = parse(YML).unwrap();
        let steps = build_steps(&file, "ship").unwrap();
        assert_eq!(steps[1]["dependsOn"], json!(["gate"]));
        assert_eq!(steps[1]["config"]["cwd"], "server");
        // release env fills in, the step's own env wins
        assert_eq!(steps[1]["config"]["env"]["A"], "1");
        assert_eq!(steps[1]["config"]["env"]["B"], "2");
    }

    #[test]
    fn rejects_release_naming_a_missing_workflow() {
        let bad = YML.replace("rehearse: ship-dry", "rehearse: nope");
        assert!(parse(&bad).unwrap_err().contains("nope"));
    }
}
