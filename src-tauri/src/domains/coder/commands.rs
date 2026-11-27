//! Tauri command surface for the coder agent.
//!
//! Two caps, the same split as `terminal`/`terminal.exec`. `coder` (granted to
//! a paired device by default) reads: threads, history, changes, modes, git
//! status. `coder.exec` (opt-in, `rpc::SENSITIVE_CAPS`) is everything that can
//! change a file or run a process on this machine: writing a file, accepting or
//! editing a change, opening a path with the shell, git commit/push, worktree
//! fan-out — and anything that drives or loosens the agent's turn loop (send,
//! retry, approve, feeding it tool output, permission mode and rules), since an
//! agent in auto-accept mode or with an allow rule runs tools unprompted.
//! Local IPC is never held to either cap.

use std::sync::Arc;

use reqwest::Client;
use tauri::State;

use crate::domains::ai::platform_config::PlatformConfig;
use crate::domains::ai::services::AISettingsService;
use crate::domains::threads::{AckStatus, CommandAck, ThreadCommand};
use crate::rpc::Caller;

use super::service::CoderService;
use super::types::{
    CoderAgentMode, CoderSubAgent, CoderThread, CoderThreadKind, CoderThreadSummary, FileChange,
    MultitaskCancelRequest, MultitaskCleanupRequest, MultitaskSpawnRequest, PermissionMode,
    PermissionRule,
};

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_create_thread(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    model: Option<String>,
    llm_provider: Option<String>,
    thread_kind: Option<CoderThreadKind>,
    project_id: Option<i32>,
) -> Result<CoderThread, String> {
    service.ensure_known_root(&workspace_root).await?;
    Ok(service
        .create_thread(workspace_root, model, llm_provider, thread_kind, project_id)
        .await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_threads(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<CoderThread>, String> {
    Ok(service.list_threads().await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_thread_summaries(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<CoderThreadSummary>, String> {
    Ok(service.list_thread_summaries().await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_get_thread(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<Option<CoderThread>, String> {
    Ok(service.get_thread(&thread_id).await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_delete_thread(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<bool, String> {
    Ok(service.delete_thread(&thread_id).await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
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

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_set_thread_kind(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    thread_kind: CoderThreadKind,
) -> Result<(), String> {
    service.set_thread_kind(&thread_id, thread_kind).await
}

/// The drive commands go through the thread hub (dedup, first-wins); what it decides
/// runs back in `CoderService` as an effect. Accepted and duplicate are both a success;
/// a rejection surfaces as its code (`busy`, `invalid`, `already_resolved`...).
fn accepted(ack: CommandAck) -> Result<(), String> {
    match ack.status {
        AckStatus::Rejected => Err(ack.code.unwrap_or_else(|| "rejected".into())),
        _ => Ok(()),
    }
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_send(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    message: String,
    command_id: Option<String>,
) -> Result<(), String> {
    // The legacy UI neither shows nor drains the hub's queue: a new message supersedes a
    // parked or stopping turn instead of queueing behind it.
    // ponytail: S4a (#1933) switches to a plain Send with a visible queue.
    service.supersede_for_send(&thread_id, &caller.actor()).await?;
    let cmd = ThreadCommand::Send { text: message };
    accepted(service.hub_command(&thread_id, cmd, caller.actor(), command_id).await?)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_retry(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    command_id: Option<String>,
) -> Result<(), String> {
    accepted(service.hub_command(&thread_id, ThreadCommand::Retry, caller.actor(), command_id).await?)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_edit_message(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    message_index: usize,
    content: String,
    command_id: Option<String>,
) -> Result<(), String> {
    let cmd = ThreadCommand::EditMessage { index: message_index, text: content };
    accepted(service.hub_command(&thread_id, cmd, caller.actor(), command_id).await?)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_approve(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    approve: bool,
    remember: Option<bool>,
    edited_pattern: Option<String>,
    command_id: Option<String>,
) -> Result<(), String> {
    let remember = remember.unwrap_or(false);
    let cmd = ThreadCommand::Approve {
        call_id: call_id.clone(),
        allow: approve,
        remember,
        edited_pattern: edited_pattern.clone(),
        edited_command: None,
    };
    let ack = service.hub_command(&thread_id, cmd, caller.actor(), command_id).await?;
    if ack.code.as_deref() == Some("not_found") && service.is_legacy_approval(&thread_id, &call_id).await {
        // An approval from before the hub existed: the old direct path.
        return service
            .resume_approve(&thread_id, call_id, approve, remember, edited_pattern, None)
            .await;
    }
    accepted(ack)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_stop(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    command_id: Option<String>,
) -> Result<bool, String> {
    match service.hub_command(&thread_id, ThreadCommand::Stop, caller.actor(), command_id).await {
        Ok(ack) if ack.status != AckStatus::Rejected => Ok(true),
        // The hub saw no open turn (a run it was never told about): stop it directly.
        _ => Ok(service.stop(&thread_id).await),
    }
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_get_context_usage(
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
) -> Result<Option<crate::domains::ai::context_usage::ContextUsage>, String> {
    service.get_context_usage(&thread_id).await
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_running(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<String>, String> {
    Ok(service.list_running().await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_get_mode(
    service: State<'_, Arc<CoderService>>,
) -> Result<CoderAgentMode, String> {
    Ok(service.get_mode().await)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_set_mode(
    service: State<'_, Arc<CoderService>>,
    mode: CoderAgentMode,
) -> Result<(), String> {
    service.set_mode(mode).await;
    Ok(())
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_get_permission_mode(
    service: State<'_, Arc<CoderService>>,
) -> Result<PermissionMode, String> {
    Ok(service.get_permission_mode().await)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_set_permission_mode(
    service: State<'_, Arc<CoderService>>,
    mode: PermissionMode,
) -> Result<(), String> {
    service.set_permission_mode(mode).await;
    Ok(())
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_rules(
    service: State<'_, Arc<CoderService>>,
) -> Result<Vec<PermissionRule>, String> {
    Ok(service.list_rules().await)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_add_rule(
    service: State<'_, Arc<CoderService>>,
    rule: PermissionRule,
) -> Result<(), String> {
    service.add_rule(rule).await;
    Ok(())
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_remove_rule(
    service: State<'_, Arc<CoderService>>,
    tool: String,
    pattern: String,
) -> Result<(), String> {
    service.remove_rule(&tool, &pattern).await;
    Ok(())
}

// ---- change review ----------------------------------------------------

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_changes(
    service: State<'_, Arc<CoderService>>,
    thread_id: Option<String>,
) -> Result<Vec<FileChange>, String> {
    Ok(service.list_changes(thread_id.as_deref()).await)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_accept_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
) -> Result<(), String> {
    service.accept_change(&change_id).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_reject_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
) -> Result<(), String> {
    service.reject_change(&change_id).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_set_hunk(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
    hunk_index: usize,
    accepted: bool,
) -> Result<(), String> {
    service.set_hunk(&change_id, hunk_index, accepted).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_modify_change(
    service: State<'_, Arc<CoderService>>,
    change_id: String,
    content: String,
) -> Result<(), String> {
    service.modify_change(&change_id, content).await
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_dir(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    path: Option<String>,
) -> Result<Vec<super::types::WorkspaceDirEntry>, String> {
    service.ensure_known_root(&workspace_root).await?;
    let rel = path.unwrap_or_else(|| ".".to_string());
    super::tools::list_dir_entries(&workspace_root, &rel)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_read_file(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    path: String,
) -> Result<String, String> {
    service.ensure_known_root(&workspace_root).await?;
    super::tools::read_file(&workspace_root, &path)
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_write_file(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    path: String,
    content: String,
) -> Result<(), String> {
    service.ensure_known_root(&workspace_root).await?;
    super::tools::write_file(&workspace_root, &path, &content)?;
    Ok(())
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub fn coder_open_in_explorer(workspace_root: String, path: String) -> Result<(), String> {
    let normalized = super::tools::resolve(&workspace_root, &path)?;
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

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_get_git_diff_stats(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
) -> Result<super::git_status::GitDiffStats, String> {
    service.ensure_known_root(&workspace_root).await?;
    Ok(super::git_status::git_diff_stats(&workspace_root))
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_list_git_changes(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
) -> Result<Vec<super::git_status::GitFileChange>, String> {
    service.ensure_known_root(&workspace_root).await?;
    Ok(super::git_status::git_list_changes(&workspace_root))
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_prepare_git_commit(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    use_ai: Option<bool>,
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<super::git_commit::GitCommitDraft, String> {
    service.ensure_known_root(&workspace_root).await?;
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

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_git_commit(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
    title: String,
    summary: Option<String>,
) -> Result<String, String> {
    service.ensure_known_root(&workspace_root).await?;
    super::git_commit::git_commit(&workspace_root, &title, summary.as_deref())
}

/// Pushes the current branch to `origin`. Never force-pushes — see
/// `git_push::push_args`.
#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_git_push(
    service: State<'_, Arc<CoderService>>,
    workspace_root: String,
) -> Result<String, String> {
    service.ensure_known_root(&workspace_root).await?;
    super::git_push::git_push(&workspace_root)
}

/// A host result goes through the hub (first answer wins). Anything else the hub says
/// (unknown call, commit failure) still delivers straight to the waiting oneshot: the
/// run would otherwise time out and execute the command a second time.
async fn resolve_host(
    caller: &Caller,
    service: &CoderService,
    thread_id: String,
    call_id: String,
    result: String,
    command_id: Option<String>,
) -> Result<(), String> {
    let cmd = ThreadCommand::ResolveHost { call_id: call_id.clone(), result: result.clone().into() };
    match service.hub_command(&thread_id, cmd, caller.actor(), command_id).await {
        Ok(ack) if ack.status != AckStatus::Rejected => Ok(()),
        Ok(ack) if ack.code.as_deref() == Some("already_resolved") => accepted(ack),
        _ => service.submit_command_result(thread_id, call_id, result).await,
    }
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_submit_command_result(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    result: String,
    command_id: Option<String>,
) -> Result<(), String> {
    resolve_host(&caller, &service, thread_id, call_id, result, command_id).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_submit_terminal_list(
    caller: Caller,
    service: State<'_, Arc<CoderService>>,
    thread_id: String,
    call_id: String,
    list_json: String,
    command_id: Option<String>,
) -> Result<(), String> {
    resolve_host(&caller, &service, thread_id, call_id, list_json, command_id).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_multitask_spawn(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskSpawnRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.inner().multitask_spawn(request).await
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_multitask_list(
    service: State<'_, Arc<CoderService>>,
    coordinator_thread_id: String,
) -> Result<Vec<CoderSubAgent>, String> {
    Ok(service.list_sub_agents(&coordinator_thread_id).await)
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_multitask_cancel(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskCancelRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.multitask_cancel(request).await
}

#[portal_rpc_macro::rpc(cap = "coder.exec")]
pub async fn coder_multitask_cleanup(
    service: State<'_, Arc<CoderService>>,
    request: MultitaskCleanupRequest,
) -> Result<Vec<CoderSubAgent>, String> {
    service.multitask_cleanup(request).await
}
