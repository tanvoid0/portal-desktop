//! Unified run list: action runs, pipeline runs and releases (all rows of
//! `pipeline_executions`) as one `RunSummary` row, no output or logs.

use super::execution_service::ExecutionService;
use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{ExecutionRepository, PipelineRepository};
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn recent_runs(db: &Arc<DatabaseManager>, limit: u64) -> Result<Vec<Value>, String> {
    summarize(db, ExecutionRepository::new(db.clone()).get_recent(None, Some(limit)).await?).await
}

pub async fn running_runs(db: &Arc<DatabaseManager>) -> Result<Vec<Value>, String> {
    summarize(db, ExecutionRepository::new(db.clone()).get_non_terminal().await?).await
}

fn short(s: &str) -> String {
    s.chars().take(200).collect()
}

/// The rows' summaries, in the order given (newest first).
async fn summarize(
    db: &Arc<DatabaseManager>,
    runs: Vec<crate::entities::pipeline_execution::Model>,
) -> Result<Vec<Value>, String> {
    let (pipeline_repo, project_repo) = (PipelineRepository::new(db.clone()), ProjectRepository::new(db.clone()));
    let mut rows = Vec::new();
    for e in runs {
        let steps = serde_json::from_str::<Vec<Value>>(&e.step_executions_json).unwrap_or_default();
        let mut row = if e.kind == "action" {
            let exit_code = steps.iter().rev().find_map(|st| st["exitCode"].as_i64());
            json!({
                "kind": "action", "title": e.command, "workingDirectory": e.working_directory,
                "workflowRunId": e.workflow_run_id, "exitCode": exit_code,
                // A Stop is recorded as `cancelled` with no error; say so (as the Action UI expects).
                "error": e.error.as_deref().or((e.status == "cancelled").then_some("Stopped by user")).map(short),
            })
        } else {
            let pipeline = match e.pipeline_id {
                Some(id) => pipeline_repo.get_by_id(id).await?,
                None => None,
            };
            let workflow = pipeline.as_ref().and_then(|p| p.preset_key.as_deref()).and_then(|k| k.strip_prefix("release:"));
            let (kind, title) = match (workflow, &pipeline) {
                (Some(w), _) => ("release", format!("Release · {w}")),
                (None, p) => ("pipeline", p.as_ref().map_or_else(|| e.pipeline_id.map(|i| i.to_string()).unwrap_or_default(), |p| p.name.clone())),
            };
            let failed = steps.iter().rfind(|st| st["status"] == "failed");
            json!({
                "kind": kind, "title": title,
                "exitCode": failed.map(|f| f["exitCode"].clone()),
                "error": failed.and_then(|f| f["error"].as_str()).or(e.error.as_deref()).map(short),
            })
        };
        let project = match e.project_id {
            Some(id) => project_repo.get_by_id(id).await?,
            None => None,
        };
        row["id"] = json!(e.id);
        row["projectId"] = json!(e.project_id.map(|i| i.to_string()));
        row["projectName"] = json!(project.map(|p| p.name));
        row["status"] = json!(e.status);
        row["startedAt"] = json!(e.started_at.to_rfc3339());
        row["finishedAt"] = json!(e.finished_at.map(|d| d.to_rfc3339()));
        row["triggeredBy"] = json!(e.triggered_by);
        row["progress"] = ExecutionService::execution_to_json(&e)["progress"].clone();
        row.as_object_mut().unwrap().retain(|_, v| !v.is_null()); // optional fields are omitted, not null
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::projects::pipelines::repositories::{ActionMeta, NewRun};

    #[tokio::test]
    async fn one_query_lists_actions_and_pipelines_newest_first_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(DatabaseManager::new(dir.path().to_path_buf()).await.unwrap());
        let project = ProjectRepository::new(db.clone())
            .create("proj".into(), None, dir.path().display().to_string(), vec![], vec![], vec![], None, None, None, None, None, None)
            .await
            .unwrap();
        let pipeline = PipelineRepository::new(db.clone())
            .create("p".into(), None, project.id, "[]".into(), "[]".into(), "[]".into(), "{}".into(), true,
                Some("release:ship".into()), Some("release".into()))
            .await
            .unwrap();
        let repo = ExecutionRepository::new(db.clone());
        repo.insert(NewRun {
            id: "s".into(),
            status: "success".into(),
            triggered_by: "user".into(),
            step_executions_json: r#"[{"stepId":"run","status":"success","exitCode":0,"logs":["secret"],"output":"secret\n"}]"#.into(),
            variables_json: "{}".into(),
            action: Some(ActionMeta { command: "echo hi".into(), working_directory: Some("/w".into()), ..Default::default() }),
            ..Default::default()
        })
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        repo.create("e".into(), Some(pipeline.id), Some(project.id), "running".into(), "user".into(),
            r#"[{"stepId":"a","status":"failed","exitCode":2,"error":"boom"}]"#.into(), "{}".into())
            .await
            .unwrap();

        let runs = recent_runs(&db, 10).await.unwrap();
        let ids: Vec<_> = runs.iter().map(|r| r["id"].as_str().unwrap()).collect();
        assert_eq!(ids[..2], ["e", "s"]); // a fresh DB may carry older seeded rows
        assert_eq!(runs[0]["kind"], "release");
        assert_eq!(runs[0]["title"], "Release · ship");
        assert_eq!(runs[0]["projectName"], "proj");
        assert!(runs[0]["progress"].is_object());
        assert_eq!((&runs[0]["exitCode"], &runs[0]["error"]), (&json!(2), &json!("boom")));
        // The action row, from the same query: its own title, folder and exit code.
        assert_eq!(runs[1]["kind"], "action");
        assert_eq!(runs[1]["title"], "echo hi");
        assert_eq!(runs[1]["workingDirectory"], "/w");
        assert_eq!(runs[1]["exitCode"], 0);
        assert!(runs[1].get("error").is_none() && runs[1].get("projectId").is_none());
        assert!(runs.iter().all(|r| r.get("output").is_none() && r.get("logs").is_none()));
        assert_eq!(recent_runs(&db, 1).await.unwrap().len(), 1);
        // Running: every kind, newest first.
        repo.update_status("e", "queued".into(), None).await.unwrap();
        let running: Vec<_> = running_runs(&db).await.unwrap().into_iter().map(|r| r["id"].clone()).collect();
        assert!(running.contains(&json!("e")) && !running.contains(&json!("s")));
    }
}
