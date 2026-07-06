//! Tauri command surface for the coder agent.

use std::sync::Arc;

use tauri::State;

use super::service::CoderService;
use super::types::{CoderRunResult, CoderThread, PermissionMode, PermissionRule};

#[tauri::command]
pub async fn coder_create_thread(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    model: Option<String>,
) -> Result<CoderThread, String> {
    Ok(service.create_thread(workspace_root, model).await)
}

#[tauri::command]
pub async fn coder_list_threads(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<CoderThread>, String> {
    Ok(service.list_threads().await)
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
pub async fn coder_send(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    message: String,
) -> Result<CoderRunResult, String> {
    service.send(&thread_id, message).await
}

#[tauri::command]
pub async fn coder_approve(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    approve: bool,
    remember: Option<bool>,
    edited_pattern: Option<String>,
) -> Result<CoderRunResult, String> {
    service
        .approve(&thread_id, &call_id, approve, remember.unwrap_or(false), edited_pattern)
        .await
}

#[tauri::command]
pub async fn coder_get_mode(
    service: State<'_, Arc<CoderService>>,
) -> Result<PermissionMode, String> {
    Ok(service.get_mode().await)
}

#[tauri::command]
pub async fn coder_set_mode(
    service: State<'_, Arc<CoderService>>,
    mode: PermissionMode,
) -> Result<(), String> {
    service.set_mode(mode).await;
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
