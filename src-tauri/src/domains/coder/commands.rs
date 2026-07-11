//! Tauri command surface for the coder agent.

use std::sync::Arc;
use std::path::PathBuf;

use reqwest::Client;
use tauri::State;

use crate::domains::ai::platform_config::PlatformConfig;
use crate::domains::ai::services::AISettingsService;

use super::service::{AgentTurn, CoderService};
use super::types::{
    CoderSubAgent, CoderThread, CoderThreadKind, CoderThreadSummary, FileChange,
    MultitaskCancelRequest, MultitaskCleanupRequest, MultitaskSpawnRequest, CoderAgentMode,
    PermissionMode, PermissionRule,
};

#[tauri::command]
pub async fn coder_create_thread(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    model: Option<String>,
    llm_provider: Option<String>,
    thread_kind: Option<CoderThreadKind>,
    project_id: Option<i32>,
) -> Result<CoderThread, String> {
    Ok(service
        .create_thread(workspace_root, model, llm_provider, thread_kind, project_id)
        .await)
}

#[tauri::command]
pub async fn coder_list_threads(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<CoderThread>, String> {
    Ok(service.list_threads().await)
}

#[tauri::command]
pub async fn coder_list_thread_summaries(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<CoderThreadSummary>, String> {
    Ok(service.list_thread_summaries().await)
}

#[tauri::command]
pub async fn coder_get_thread(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<Option<CoderThread>, String> {
    Ok(service.get_thread(&thread_id).await)
}

#[tauri::command]
pub async fn coder_delete_thread(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<bool, String> {
    Ok(service.delete_thread(&thread_id).await)
}

#[tauri::command]
pub async fn coder_update_thread_model(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    model: Option<String>,
    llm_provider: Option<String>,
) -> Result<(), String> {
    service
        .update_thread_model(&thread_id, model, llm_provider)
        .await
}

#[tauri::command]
pub async fn coder_set_thread_kind(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    thread_kind: CoderThreadKind,
) -> Result<(), String> {
    service.set_thread_kind(&thread_id, thread_kind).await
}

#[tauri::command]
pub async fn coder_send(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    message: String,
) -> Result<(), String> {
    service.prepare_send(&thread_id, message).await?;
    CoderService::spawn_run(service.inner().clone(), thread_id, AgentTurn::Send);
    Ok(())
}

#[tauri::command]
pub async fn coder_retry(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<(), String> {
    service.prepare_retry(&thread_id).await?;
    CoderService::spawn_run(service.inner().clone(), thread_id, AgentTurn::Retry);
    Ok(())
}

#[tauri::command]
pub async fn coder_edit_message(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    message_index: usize,
    content: String,
) -> Result<(), String> {
    service
        .prepare_edit_message(&thread_id, message_index, content)
        .await?;
    CoderService::spawn_run(service.inner().clone(), thread_id, AgentTurn::Send);
    Ok(())
}

#[tauri::command]
pub async fn coder_approve(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    approve: bool,
    remember: Option<bool>,
    edited_pattern: Option<String>,
    remember_tool: Option<String>,
    remember_args: Option<serde_json::Value>,
) -> Result<(), String> {
    service
        .prepare_approve(
            &thread_id,
            &call_id,
            approve,
            remember.unwrap_or(false),
            edited_pattern.clone(),
            remember_tool,
            remember_args,
        )
        .await?;
    CoderService::spawn_run(
        service.inner().clone(),
        thread_id,
        AgentTurn::Approve {
            call_id,
            approve,
            edited_command: None,
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn coder_stop(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<bool, String> {
    Ok(service.stop(&thread_id).await)
}

#[tauri::command]
pub async fn coder_get_context_usage(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<Option<crate::domains::ai::context_usage::ContextUsage>, String> {
    service.get_context_usage(&thread_id).await
}

#[tauri::command]
pub async fn coder_list_running(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<String>, String> {
    Ok(service.list_running().await)
}

#[tauri::command]
pub async fn coder_get_mode(
    service: State<'_, Arc<CoderService>>,
) -> Result<CoderAgentMode, String> {
    Ok(service.get_mode().await)
}

#[tauri::command]
pub async fn coder_set_mode(
    service: State<'_, Arc<CoderService>>,
    mode: CoderAgentMode,
) -> Result<(), String> {
    service.set_mode(mode).await;
    Ok(())
}

#[tauri::command]
pub async fn coder_get_permission_mode(
    service: State<'_, Arc<CoderService>>,
) -> Result<PermissionMode, String> {
    Ok(service.get_permission_mode().await)
}

#[tauri::command]
pub async fn coder_set_permission_mode(
    service: State<'_, Arc<CoderService>>,
    mode: PermissionMode,
) -> Result<(), String> {
    service.set_permission_mode(mode).await;
    Ok(())
}

#[tauri::command]
pub async fn coder_list_rules(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<PermissionRule>, String> {
    Ok(service.list_rules().await)
}

#[tauri::command]
pub async fn coder_add_rule(
    service: State<'_, Arc<CoderService>>,
    rule: PermissionRule,
) -> Result<(), String> {
    service.add_rule(rule).await;
    Ok(())
}

#[tauri::command]
pub async fn coder_remove_rule(
    service: State<'_, Arc<CoderService>>,
    tool: String,
    pattern: String,
) -> Result<(), String> {
    service.remove_rule(&tool, &pattern).await;
    Ok(())
}

// ---- change review ----------------------------------------------------

#[tauri::command]
pub async fn coder_list_changes(
    service: State<'_, Arc<CoderService>>,
    thread_id: Option<String>,
) -> Result<Vec<FileChange>, String> {
    Ok(service.list_changes(thread_id.as_deref()).await)
}

#[tauri::command]
pub async fn coder_accept_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
) -> Result<(), String> {
    service.accept_change(&change_id).await
}

#[tauri::command]
pub async fn coder_reject_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
) -> Result<(), String> {
    service.reject_change(&change_id).await
}

#[tauri::command]
pub async fn coder_set_hunk(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
    hunk_index: usize,
    accepted: bool,
) -> Result<(), String> {
    service.set_hunk(&change_id, hunk_index, accepted).await
}

#[tauri::command]
pub async fn coder_modify_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
    content: String,
) -> Result<(), String> {
    service.modify_change(&change_id, content).await
}

#[tauri::command]
pub fn coder_list_dir(
    workspace_root: String,
    path: Option<String>,
) -> Result<Vec<super::types::WorkspaceDirEntry>, String> {
    let rel = path.unwrap_or_else(|| ".".to_string());
    super::tools::list_dir_entries(&workspace_root, &rel)
}

#[tauri::command]
pub fn coder_read_file(workspace_root: String, path: String) -> Result<String, String> {
    super::tools::read_file(&workspace_root, &path)
}

#[tauri::command]
pub fn coder_write_file(workspace_root: String, path: String, content: String) -> Result<(), String> {
    super::tools::write_file(&workspace_root, &path, &content)?;
    Ok(())
}

#[tauri::command]
pub fn coder_open_in_explorer(workspace_root: String, path: String) -> Result<(), String> {
    let root = PathBuf::from(&workspace_root);
    let joined = root.join(path);

    // Normalize without requiring the path to already exist.
    let mut normalized = PathBuf::new();
    for comp in joined.components() {
        use std::path::Component::*;
        match comp {
            ParentDir => {
                if !normalized.pop() {
                    return Err("path escapes workspace root".into());
                }
            }
            CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(&root) {
        return Err("path escapes workspace root".into());
    }
    if !normalized.exists() {
        return Err("path does not exist".into());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(normalized)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(normalized)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(normalized)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn coder_get_git_diff_stats(workspace_root: String) -> super::git_status::GitDiffStats {
    super::git_status::git_diff_stats(&workspace_root)
}

#[tauri::command]
pub fn coder_list_git_changes(
    workspace_root: String,
) -> Vec<super::git_status::GitFileChange> {
    super::git_status::git_list_changes(&workspace_root)
}

#[tauri::command]
pub async fn coder_prepare_git_commit(
    workspace_root: String,
    use_ai: Option<bool>,
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<super::git_commit::GitCommitDraft, String> {
    let mut draft = super::git_commit::prepare_commit(&workspace_root)?;
    if use_ai.unwrap_or(true) {
        let cfg = PlatformConfig::resolve(&settings_service);
        let client = Client::new();
        if let Ok((title, summary)) =
            super::git_commit::suggest_commit_message(&client, &cfg, &draft.changes).await
        {
            draft.title = title;
            draft.summary = summary;
            draft.ai_generated = true;
        }
    }
    Ok(draft)
}

#[tauri::command]
pub fn coder_git_commit(
    workspace_root: String,
    title: String,
    summary: Option<String>,
) -> Result<String, String> {
    super::git_commit::git_commit(&workspace_root, &title, summary.as_deref())
}

#[tauri::command]
pub async fn coder_submit_command_result(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    result: String,
) -> Result<(), String> {
    service
        .submit_command_result(thread_id, call_id, result)
        .await
}

#[tauri::command]
pub async fn coder_submit_terminal_list(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    list_json: String,
) -> Result<(), String> {
    service
        .submit_terminal_list(thread_id, call_id, list_json)
        .await
}

#[tauri::command]
pub async fn coder_multitask_spawn(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskSpawnRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.inner().multitask_spawn(request).await
}

#[tauri::command]
pub async fn coder_multitask_list(
    service: State<'_, Arc<CoderService>>,
    coordinator_thread_id: String,
) -> Result<Vec<CoderSubAgent>, String> {
    Ok(service.list_sub_agents(&coordinator_thread_id).await)
}

#[tauri::command]
pub async fn coder_multitask_cancel(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskCancelRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.multitask_cancel(request).await
}

#[tauri::command]
pub async fn coder_multitask_cleanup(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskCleanupRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.multitask_cleanup(request).await
}
