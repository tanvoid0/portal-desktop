//! `apply`: fold one event into the state. Pure; `last_seq` is the caller's job.
//! `actor` is who the event's frame names: `recently_resolved` records it.

use super::types::*;

fn resolved(s: &mut ThreadState, call_id: &str, allow: bool, by: &str) {
    s.recently_resolved.push(ResolvedCall { call_id: call_id.into(), allow, by: by.into() });
    if s.recently_resolved.len() > RECENTLY_RESOLVED {
        s.recently_resolved.remove(0);
    }
}

pub fn apply(s: &mut ThreadState, e: &ThreadEvent, actor: &str) {
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
        E::MessagesRewritten { keep, tail } => {
            s.messages.truncate(*keep as usize);
            s.messages.extend(tail.iter().cloned());
        }
        E::TurnStarted { .. } => {
            s.status = ThreadStatus::Running;
            s.error = None;
        }
        E::TurnEnded { outcome, error, .. } => {
            s.error = error.clone();
            s.stopping = false;
            // Nothing can answer a call of a turn that is over.
            s.approvals.clear();
            s.host_requests.clear();
            s.status = if *outcome == TurnOutcome::Error { ThreadStatus::Error } else { ThreadStatus::Idle };
        }
        E::StopRequested => s.stopping = true,
        E::ApprovalRequested { approval } => {
            match s.approvals.iter_mut().find(|a| a.call_id == approval.call_id) {
                Some(slot) => *slot = approval.clone(),
                None => s.approvals.push(approval.clone()),
            }
            s.status = ThreadStatus::Waiting;
        }
        E::ApprovalResolved { call_id, allow, .. } => {
            s.approvals.retain(|a| &a.call_id != call_id);
            resolved(s, call_id, *allow, actor);
            if s.approvals.is_empty() && s.status == ThreadStatus::Waiting {
                s.status = ThreadStatus::Running;
            }
        }
        E::HostRequested { call_id, kind, payload } => {
            let r = HostRequest { call_id: call_id.clone(), kind: kind.clone(), payload: payload.clone() };
            match s.host_requests.iter_mut().find(|h| &h.call_id == call_id) {
                Some(slot) => *slot = r,
                None => s.host_requests.push(r),
            }
        }
        E::HostResolved { call_id } => {
            s.host_requests.retain(|h| &h.call_id != call_id);
            resolved(s, call_id, true, actor);
        }
        E::Enqueued { queue_id, text } => {
            s.queue.push(QueuedMessage { queue_id: queue_id.clone(), text: text.clone() });
        }
        E::Dequeued { queue_id, .. } => s.queue.retain(|q| &q.queue_id != queue_id),
        E::TitleSet { title } => s.meta.title = title.clone(),
        E::ModelSet { model, llm_provider } => {
            s.meta.model = model.clone();
            s.meta.llm_provider = llm_provider.clone();
        }
        // The change body lives in `coder_file_changes`; the log only announces it.
        E::ChangeRecorded { .. } => {}
        E::SubAgentUpdated { sub_agent } => match s.sub_agents.iter_mut().find(|a| a.id == sub_agent.id) {
            Some(slot) => *slot = sub_agent.clone(),
            None => s.sub_agents.push(sub_agent.clone()),
        },
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
