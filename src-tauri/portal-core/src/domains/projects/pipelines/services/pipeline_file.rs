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
    #[serde(
        default,
        rename = "longRunning",
        skip_serializing_if = "std::ops::Not::not"
    )]
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
    /// Fan this step out to one runtime node per item. Node ids are
    /// `{action}:{item}`, and any step that `needs` this action depends on all of
    /// them. See `ForEach`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreach: Option<ForEach>,
}

/// Fans a step out to one node per item. Items come from an inline `items` list
/// (known everywhere) or an `items_file` — a path, relative to the project root,
/// of a file listing them one per line — read natively at release start. Until
/// that run — in `release_validate` and `release_describe`, which have no project
/// context — an `items_file` fan-out collapses to one placeholder node named
/// after the action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForEach {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// Path (relative to the project root) of a file listing the items, one per
    /// line; blank lines and `#` comments are ignored. Read natively — no shell —
    /// so the list can be owned by another tool without gluing a script into the
    /// pipeline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_file: Option<String>,
    /// Env var each item is injected into (default `ITEM`).
    #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
    pub as_var: Option<String>,
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
                return Err(format!(
                    "release names workflow '{id}', which is not defined"
                ));
            }
        }
    }
    Ok(file)
}

/// The file as YAML text, with the `version: 1` the frontend requires.
pub fn to_yaml(file: &PipelineFile) -> Result<String, String> {
    let body = serde_yaml::to_string(file).map_err(|e| e.to_string())?;
    Ok(format!(
        "version: 1
{body}"
    ))
}

/// `Ok(None)` when the project has no pipeline file.
pub fn load(project_path: &str) -> Result<Option<PipelineFile>, String> {
    for rel in CANDIDATES {
        let path = Path::new(project_path).join(rel);
        if path.is_file() {
            let raw =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            return parse(&raw).map(Some);
        }
    }
    Ok(None)
}

/// A workflow as the runner's `steps_json` shape: `{id, name, dependsOn, config}`.
///
/// `resolved` supplies the item lists for `foreach` steps whose items come from a
/// `command` (see `ForEach`); pass an empty map where there is no run context
/// (`release_validate`, `release_describe`) and such a step collapses to one
/// placeholder node. A `foreach` with inline `items` expands either way.
pub fn build_steps(
    file: &PipelineFile,
    workflow_id: &str,
    resolved: &HashMap<String, Vec<String>>,
) -> Result<Vec<Value>, String> {
    let wf = file
        .workflows
        .get(workflow_id)
        .ok_or_else(|| format!("No workflow '{workflow_id}'"))?;
    let release_env = file.release.as_ref().map(|r| &r.env);

    // Pass 1: emit the (possibly fanned-out) rows, recording for each source
    // action the node ids it produced, so `needs` can be rewritten in pass 2.
    let mut rows: Vec<Value> = Vec::with_capacity(wf.steps.len());
    let mut produced: HashMap<&str, Vec<String>> = HashMap::new();

    for step in &wf.steps {
        let action = file.actions.get(&step.action).ok_or_else(|| {
            format!(
                "Workflow '{workflow_id}' uses undefined action '{}'",
                step.action
            )
        })?;
        let name = action.name.clone().unwrap_or_else(|| step.action.clone());
        let base_env = || {
            let mut env: HashMap<String, String> = release_env
                .into_iter()
                .flatten()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            env.extend(action.env.iter().map(|(k, v)| (k.clone(), v.clone())));
            env
        };
        let row = |id: &str, name: &str, env: HashMap<String, String>| {
            json!({
                "id": id,
                "name": name,
                "dependsOn": step.needs,          // rewritten in pass 2
                "config": {
                    "command": action.run,
                    "cwd": action.cwd,
                    "env": env,
                    "longRunning": action.long_running,
                },
            })
        };

        // A foreach step fans out when its items are known: inline `items`, or a
        // `command` resolved for this run. Otherwise it stays one placeholder.
        let items: Option<&[String]> = step.foreach.as_ref().and_then(|fe| {
            if !fe.items.is_empty() {
                Some(fe.items.as_slice())
            } else {
                resolved.get(&step.action).map(|v| v.as_slice())
            }
        });

        match (step.foreach.as_ref(), items) {
            (Some(fe), Some(items)) => {
                let var = fe.as_var.as_deref().unwrap_or("ITEM");
                let mut ids = Vec::with_capacity(items.len());
                for item in items {
                    let id = format!("{}:{}", step.action, item);
                    let mut env = base_env();
                    env.insert(var.to_string(), item.clone());
                    rows.push(row(&id, &format!("{name} · {item}"), env));
                    ids.push(id);
                }
                produced.insert(step.action.as_str(), ids);
            }
            _ => {
                rows.push(row(&step.action, &name, base_env()));
                produced.insert(step.action.as_str(), vec![step.action.clone()]);
            }
        }
    }

    // Pass 2: a dependency on a fanned-out action becomes a dependency on all of
    // its node ids; a dependency on a normal action maps to itself.
    for row in &mut rows {
        let deps = row["dependsOn"].as_array().cloned().unwrap_or_default();
        let mut out: Vec<String> = Vec::new();
        for dep in deps {
            let Some(d) = dep.as_str() else { continue };
            match produced.get(d) {
                Some(ids) => out.extend(ids.iter().cloned()),
                None => out.push(d.to_string()),
            }
        }
        row["dependsOn"] = json!(out);
    }

    Ok(rows)
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

    fn no_items() -> HashMap<String, Vec<String>> {
        HashMap::new()
    }

    #[test]
    fn builds_steps_with_deps_cwd_and_env_precedence() {
        let file = parse(YML).unwrap();
        let steps = build_steps(&file, "ship", &no_items()).unwrap();
        assert_eq!(steps[1]["dependsOn"], json!(["gate"]));
        assert_eq!(steps[1]["config"]["cwd"], "server");
        // release env fills in, the step's own env wins
        assert_eq!(steps[1]["config"]["env"]["A"], "1");
        assert_eq!(steps[1]["config"]["env"]["B"], "2");
    }

    /// A typo in our own release file would otherwise surface on release day.
    #[test]
    fn this_repos_own_release_file_builds() {
        let file = parse(include_str!("../../../../../../../.portal/pipeline.yml")).unwrap();
        for id in file.workflows.keys() {
            build_steps(&file, id, &no_items()).unwrap();
        }
        // A rehearsal must never reach the real publish step.
        let rehearse = &file.workflows[&file.release.as_ref().unwrap().rehearse.clone().unwrap()];
        assert!(rehearse.steps.iter().all(|s| s.action != "publish"));
    }

    #[test]
    fn rejects_release_naming_a_missing_workflow() {
        let bad = YML.replace("rehearse: ship-dry", "rehearse: nope");
        assert!(parse(&bad).unwrap_err().contains("nope"));
    }

    const FANOUT: &str = r#"
actions:
  plan: { run: echo plan }
  build: { name: Build, run: 'echo $APP', env: { SHARED: "1" } }
  stage: { run: echo stage }
workflows:
  ship:
    steps:
      - { action: plan }
      - { action: build, needs: [plan], foreach: { items_file: apps.txt, as: APP } }
      - { action: stage, needs: [build] }
"#;

    #[test]
    fn inline_items_fan_out_with_ids_names_and_env() {
        let file = parse(FANOUT).unwrap();
        let mut resolved = HashMap::new();
        resolved.insert("build".to_string(), vec!["gym".to_string(), "task".to_string()]);
        let steps = build_steps(&file, "ship", &resolved).unwrap();

        let ids: Vec<&str> = steps.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["plan", "build:gym", "build:task", "stage"]);

        // name carries the item; the loop var is injected alongside the action env.
        assert_eq!(steps[1]["name"], "Build · gym");
        assert_eq!(steps[1]["config"]["env"]["APP"], "gym");
        assert_eq!(steps[1]["config"]["env"]["SHARED"], "1");
        assert_eq!(steps[2]["config"]["env"]["APP"], "task");

        // each fanned node keeps the source step's needs; stage depends on ALL of them.
        assert_eq!(steps[1]["dependsOn"], json!(["plan"]));
        assert_eq!(steps[3]["dependsOn"], json!(["build:gym", "build:task"]));
    }

    #[test]
    fn items_file_foreach_with_no_resolved_items_is_a_single_placeholder() {
        // validate/describe context: no run, so an items_file fan-out stays one
        // node and dependents still resolve against the action name.
        let file = parse(FANOUT).unwrap();
        let steps = build_steps(&file, "ship", &no_items()).unwrap();
        let ids: Vec<&str> = steps.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["plan", "build", "stage"]);
        assert_eq!(steps[2]["dependsOn"], json!(["build"]));
    }
}
