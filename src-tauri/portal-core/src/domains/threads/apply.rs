//! `apply`: fold one event into the state. Pure; `last_seq` is the caller's job.

use super::types::*;

pub fn apply(s: &mut ThreadState, e: &ThreadEvent) {
    use ThreadEvent as E;
    match e {
        E::Created { provider, model, llm_provider, workspace_root, project_id, kind, title } => {
            s.meta = ThreadMeta {
                provider: provider.clone(),
                model: model.clone(),
                llm_provider: llm_provider.clone(),
                workspace_root: workspace_root.clone(),
                project_id: *project_id,
                kind: kind.clone(),
                title: title.clone(),
            };
            s.status = ThreadStatus::Idle;
        }
        E::MessageAppended { message } => s.messages.push(message.clone()),
        E::TurnStarted { .. } => {
            s.status = ThreadStatus::Running;
            s.error = None;
        }
        E::TurnEnded { outcome, error, .. } => {
            s.error = error.clone();
            s.status = if *outcome == TurnOutcome::Error { ThreadStatus::Error } else { ThreadStatus::Idle };
        }
        E::ApprovalRequested { approval } => {
            match s.approvals.iter_mut().find(|a| a.call_id == approval.call_id) {
                Some(slot) => *slot = approval.clone(),
                None => s.approvals.push(approval.clone()),
            }
            s.status = ThreadStatus::Waiting;
        }
        E::ApprovalResolved { call_id, .. } => {
            s.approvals.retain(|a| &a.call_id != call_id);
            if s.approvals.is_empty() && s.status == ThreadStatus::Waiting {
                s.status = ThreadStatus::Running;
            }
        }
        E::UsageUpdated { total_cost_usd, model_usage } => {
            if total_cost_usd.is_some() {
                s.usage.total_cost_usd = *total_cost_usd;
            }
            if model_usage.is_some() {
                s.usage.model_usage = model_usage.clone();
            }
        }
        E::TodosUpdated { items } => s.todos = Some(items.clone()),
        E::SubagentsRunning { count } => s.subagents_running = *count,
        E::SessionLinked { backend_session_id } => s.backend_session_id = Some(backend_session_id.clone()),
        // Nothing can answer a dead process; a resume must not revive old approvals.
        E::ProcessExited { .. } => s.approvals.clear(),
    }
}
