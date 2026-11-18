//! Unified run list: script executions and pipeline executions (releases included)
//! as one `RunSummary` row, no output or logs.
// ponytail: throwaway in-memory merge of two tables; Phase 3 replaces it with one query.

use super::execution_service::ExecutionService;
use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{ExecutionRepository, PipelineRepository};
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use crate::domains::scripts::repositories::ScriptExecutionRepository;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn recent_runs(db: &Arc<DatabaseManager>, limit: u64) -> Result<Vec<Value>, String> {
    let scripts = ScriptExecutionRepository::new(db.clone()).get_recent(limit).await?;
    let pipelines = ExecutionRepository::new(db.clone()).get_all(Some(limit)).await?;
    merge(db, scripts, pipelines, limit as usize).await
}

pub async fn running_runs(db: &Arc<DatabaseManager>) -> Result<Vec<Value>, String> {
    let mut scripts = ScriptExecutionRepository::new(db.clone()).get_running().await?;
    scripts.extend(ScriptExecutionRepository::new(db.clone()).get_pending().await?);
    let pipelines = ExecutionRepository::new(db.clone()).get_non_terminal().await?;
    merge(db, scripts, pipelines, usize::MAX).await
}

fn short(s: &str) -> String {
    s.chars().take(200).collect()
}

async fn merge(
    db: &Arc<DatabaseManager>,
    scripts: Vec<crate::entities::script_execution::Model>,
    pipelines: Vec<crate::entities::pipeline_execution::Model>,
    limit: usize,
) -> Result<Vec<Value>, String> {
    let (pipeline_repo, project_repo) = (PipelineRepository::new(db.clone()), ProjectRepository::new(db.clone()));
    let mut rows = Vec::new();
    for s in scripts {
        rows.push((s.started_at, json!({
            "id": s.id, "kind": "action", "title": s.command,
            "workingDirectory": s.working_directory, "status": s.status,
            "startedAt": s.started_at.to_rfc3339(), "finishedAt": s.finished_at.map(|d| d.to_rfc3339()),
            "triggeredBy": s.triggered_by, "workflowRunId": s.workflow_run_id,
            "exitCode": s.exit_code, "error": s.error.as_deref().map(short),
        })));
    }
    for e in pipelines {
        let pipeline = pipeline_repo.get_by_id(e.pipeline_id).await?;
        let workflow = pipeline.as_ref().and_then(|p| p.preset_key.as_deref()).and_then(|k| k.strip_prefix("release:"));
        let (kind, title) = match (workflow, &pipeline) {
            (Some(w), _) => ("release", format!("Release · {w}")),
            (None, p) => ("pipeline", p.as_ref().map_or_else(|| e.pipeline_id.to_string(), |p| p.name.clone())),
        };
        let failed = serde_json::from_str::<Vec<Value>>(&e.step_executions_json)
            .unwrap_or_default()
            .into_iter()
            .rfind(|st| st["status"] == "failed");
        let error = failed.as_ref().and_then(|f| f["error"].as_str()).or(e.error.as_deref()).map(short);
        let project = project_repo.get_by_id(e.project_id).await?;
        rows.push((e.started_at, json!({
            "id": e.id, "kind": kind, "title": title,
            "projectId": e.project_id.to_string(), "projectName": project.map(|p| p.name),
            "status": e.status, "startedAt": e.started_at.to_rfc3339(),
            "finishedAt": e.finished_at.map(|d| d.to_rfc3339()), "triggeredBy": e.triggered_by,
            "exitCode": failed.as_ref().map(|f| f["exitCode"].clone()), "error": error,
            "progress": ExecutionService::execution_to_json(&e)["progress"],
        })));
    }
    for (_, row) in &mut rows {
        row.as_object_mut().unwrap().retain(|_, v| !v.is_null()); // optional fields are omitted, not null
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.truncate(limit);
    Ok(rows.into_iter().map(|(_, v)| v).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn merges_scripts_and_pipelines_newest_first_without_output() {
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
        ScriptExecutionRepository::new(db.clone())
            .create("s".into(), None, "echo hi".into(), "{}".into(), None, "user".into(), Default::default())
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        ExecutionRepository::new(db.clone())
            .create("e".into(), pipeline.id, project.id, "running".into(), "user".into(),
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
        assert!(runs[1].get("exitCode").is_none() && runs[1].get("error").is_none());
        assert_eq!(runs[1]["kind"], "action");
        assert!(runs.iter().all(|r| r.get("output").is_none()));
        assert_eq!(recent_runs(&db, 1).await.unwrap().len(), 1);
        assert!(running_runs(&db).await.unwrap().iter().any(|r| r["id"] == "e"));
    }
}
