//! `decide`: command + state -> events. Pure and deterministic (time and the
//! actor's partial thinking arrive through `Ctx`). The `AgentEvent` fold here is
//! the one `agents/utils/eventsToMessages.ts` used to do on the client.

use serde_json::{json, Value};

use super::types::*;
use crate::domains::agents::AgentEvent;
use crate::domains::coder::types::{ChatMessage, PendingApproval};

pub fn decide(state: &ThreadState, cmd: &ThreadCommand, ctx: &Ctx) -> Result<Vec<ThreadEvent>, Reject> {
    use ThreadCommand as C;
    use ThreadEvent as E;
    match cmd {
        C::Create(_) if state.created() => Err(Reject::AlreadyCreated),
        C::Create(m) => Ok(vec![E::Created {
            provider: m.provider.clone(),
            model: m.model.clone(),
            llm_provider: m.llm_provider.clone(),
            workspace_root: m.workspace_root.clone(),
            project_id: m.project_id,
            kind: m.kind.clone(),
            title: m.title.clone(),
        }]),
        _ if !state.created() => Err(Reject::NotCreated),
        C::Provider(ev) => Ok(provider(state, ev, ctx)),
        C::Portal(p) => Ok(portal(state, p)),
        C::Send { text } if text.trim().is_empty() => Err(Reject::Invalid),
        C::Send { text } if state.turn_open() => {
            Ok(vec![E::Enqueued { queue_id: ctx.queue_id.clone(), text: text.clone() }])
        }
        C::Send { text } => Ok(vec![
            E::MessageAppended { message: msg("user", text.clone(), ctx) },
            E::TurnStarted { trigger: TurnTrigger::Send },
        ]),
        C::RemoveQueued { queue_id } => {
            let found = state.queue.iter().any(|q| &q.queue_id == queue_id);
            found
                .then(|| vec![E::Dequeued { queue_id: queue_id.clone(), reason: DequeueReason::Removed }])
                .ok_or(Reject::NotFound)
        }
        C::EditMessage { .. } | C::Retry if state.turn_open() => Err(Reject::Busy),
        C::EditMessage { index, text } => {
            let text = text.trim();
            let m = state.messages.get(*index).filter(|m| m.role == "user" && !text.is_empty());
            let mut edited = m.ok_or(Reject::Invalid)?.clone();
            edited.content = Some(text.into());
            // The client drops its queue on an edit, so the log does too.
            let mut out: Vec<_> = state
                .queue
                .iter()
                .map(|q| E::Dequeued { queue_id: q.queue_id.clone(), reason: DequeueReason::Removed })
                .collect();
            out.push(E::MessagesRewritten { keep: *index as u32, tail: vec![edited] });
            out.push(E::TurnStarted { trigger: TurnTrigger::Edit });
            Ok(out)
        }
        C::Retry => {
            let last_user = state
                .messages
                .iter()
                .rposition(|m| m.role == "user" && !m.content.as_deref().unwrap_or("").trim().is_empty())
                .ok_or(Reject::Invalid)?;
            let mut out = vec![];
            if last_user + 1 < state.messages.len() {
                out.push(E::MessagesRewritten { keep: last_user as u32 + 1, tail: vec![] });
            }
            out.push(E::TurnStarted { trigger: TurnTrigger::Retry });
            Ok(out)
        }
        // Already stopping: no new event, but `react` fires `Cancel` again (a lost one is retryable).
        C::Stop if state.turn_open() => Ok(if state.stopping { vec![] } else { vec![E::StopRequested] }),
        C::Stop => Err(Reject::NotRunning),
        C::Approve { call_id, allow, .. } => {
            if state.approvals.iter().any(|a| &a.call_id == call_id) {
                Ok(vec![E::ApprovalResolved { call_id: call_id.clone(), allow: *allow, auto: false }])
            } else {
                Err(first_wins(state, call_id))
            }
        }
        C::ResolveHost { call_id, .. } => {
            if state.host_requests.iter().any(|h| &h.call_id == call_id) {
                Ok(vec![E::HostResolved { call_id: call_id.clone() }])
            } else {
                Err(first_wins(state, call_id))
            }
        }
        C::SetModel { model, llm_provider } => {
            let same = *model == state.meta.model && *llm_provider == state.meta.llm_provider;
            Ok(if same { vec![] } else { vec![E::ModelSet { model: model.clone(), llm_provider: llm_provider.clone() }] })
        }
        C::ChangeRecorded { change_id, path, status } => Ok(vec![E::ChangeRecorded {
            change_id: change_id.clone(),
            path: path.clone(),
            status: status.clone(),
        }]),
        C::SubAgentUpdated { sub_agent } => Ok(vec![E::SubAgentUpdated { sub_agent: sub_agent.clone() }]),
        C::DrainQueue => Ok(match state.queue.first() {
            Some(q) if !state.turn_open() => vec![
                E::Dequeued { queue_id: q.queue_id.clone(), reason: DequeueReason::Sent },
                E::MessageAppended { message: msg("user", q.text.clone(), ctx) },
                E::TurnStarted { trigger: TurnTrigger::Queue },
            ],
            _ => vec![],
        }),
    }
}

/// A call that is not pending was either answered (say by whom) or never existed.
fn first_wins(state: &ThreadState, call_id: &str) -> Reject {
    state
        .recently_resolved
        .iter()
        .rev()
        .find(|r| r.call_id == call_id)
        .map_or(Reject::NotFound, |r| Reject::AlreadyResolved { by: r.by.clone(), allow: r.allow })
}

/// Same message, ignoring its timestamp (the platform's list carries none).
fn same(a: &ChatMessage, b: &ChatMessage) -> bool {
    a.role == b.role
        && a.content.as_deref().unwrap_or("") == b.content.as_deref().unwrap_or("")
        && a.tool_calls == b.tool_calls
        && a.tool_call_id == b.tool_call_id
}

fn portal(state: &ThreadState, p: &PortalInput) -> Vec<ThreadEvent> {
    use PortalInput as P;
    use ThreadEvent as E;
    let ends = |outcome, error, final_text| {
        state.turn_open().then(|| E::TurnEnded { outcome, error, final_text }).into_iter().collect()
    };
    match p {
        P::Reconcile { messages, title } => {
            // Common prefix: usually only the final assistant message is new.
            let k = state.messages.iter().zip(messages).take_while(|(a, b)| same(a, b)).count();
            let mut out = vec![];
            if k != state.messages.len() || k != messages.len() {
                // `content: None` serializes as `""` and reads back `Some("")`; store that form
                // so live, reload and replay agree.
                let tail = messages[k..]
                    .iter()
                    .cloned()
                    .map(|m| ChatMessage { content: Some(m.content.unwrap_or_default()), ..m })
                    .collect();
                out.push(E::MessagesRewritten { keep: k as u32, tail });
            }
            if let Some(t) = title.as_ref().filter(|t| **t != state.meta.title) {
                out.push(E::TitleSet { title: t.clone() });
            }
            out
        }
        P::Title { title } if *title != state.meta.title => vec![E::TitleSet { title: title.clone() }],
        P::Title { .. } | P::Heartbeat { .. } => vec![],
        P::HostRequest { call_id, kind, payload } => vec![E::HostRequested {
            call_id: call_id.clone(),
            kind: kind.clone(),
            payload: payload.clone(),
        }],
        P::Pending(approval) => vec![E::ApprovalRequested { approval: approval.clone() }],
        P::Failed { error } => ends(TurnOutcome::Error, Some(error.clone()), None),
        P::Finished { outcome, final_text } => {
            // Else the UI ends the turn silently and offers no Retry.
            let error = (*outcome == TurnOutcome::Exhausted).then(|| "Agent hit the max iteration limit.".to_string());
            ends(*outcome, error, final_text.clone())
        }
    }
}

/// Effects of a committed client command, from its `frames` (empty for a repeated
/// Stop) and the state after them.
/// Producer inputs never react: they report what a provider already did.
pub fn react(state: &ThreadState, cmd: &ThreadCommand, frames: &[ThreadEventFrame]) -> Vec<Effect> {
    use ThreadCommand as C;
    use ThreadEvent as E;
    let started = frames.iter().find_map(|f| match f.event {
        E::TurnStarted { trigger } => Some(trigger),
        _ => None,
    });
    let has = |f: fn(&ThreadEvent) -> bool| frames.iter().any(|fr| f(&fr.event));
    match cmd {
        // In all four the thread now ends on the user message the turn answers.
        C::Send { .. } | C::EditMessage { .. } | C::Retry | C::DrainQueue => started
            .map(|trigger| Effect::StartTurn {
                trigger,
                text: state.messages.last().and_then(|m| m.content.clone()).unwrap_or_default(),
                actor: frames.first().map(|f| f.actor.clone()).unwrap_or_default(),
            })
            .into_iter()
            .collect(),
        C::Stop if state.stopping => vec![Effect::Cancel],
        C::Approve { call_id, allow, remember, edited_pattern, edited_command }
            if has(|e| matches!(e, E::ApprovalResolved { .. })) =>
        {
            vec![Effect::ResolveApproval {
                call_id: call_id.clone(),
                allow: *allow,
                remember: *remember,
                edited_pattern: edited_pattern.clone(),
                edited_command: edited_command.clone(),
            }]
        }
        C::ResolveHost { call_id, result } if has(|e| matches!(e, E::HostResolved { .. })) => {
            vec![Effect::DeliverHostResult { call_id: call_id.clone(), result: result.clone() }]
        }
        _ => vec![],
    }
}

fn msg(role: &str, text: String, ctx: &Ctx) -> ChatMessage {
    ChatMessage {
        role: role.into(),
        // Always `Some`: the wire form writes `""` for `None`, which would read
        // back as `Some("")` and break replay equality.
        content: Some(text),
        tool_calls: None,
        tool_call_id: None,
        timestamp: Some(ctx.now.clone()),
    }
}

/// `parent_tool_use_id` lives only on the event types a subagent turn streams.
fn is_subagent(e: &AgentEvent) -> bool {
    matches!(
        e,
        AgentEvent::TextDelta { parent_tool_use_id: Some(_), .. }
            | AgentEvent::ThinkingDelta { parent_tool_use_id: Some(_), .. }
            | AgentEvent::Message { parent_tool_use_id: Some(_), .. }
            | AgentEvent::ToolCall { parent_tool_use_id: Some(_), .. }
    )
}

fn summarize(tool: &str, input: &Value) -> String {
    ["command", "file_path"]
        .iter()
        .find_map(|k| input.get(k).and_then(Value::as_str))
        .unwrap_or(tool)
        .to_string()
}

fn provider(state: &ThreadState, ev: &AgentEvent, ctx: &Ctx) -> Vec<ThreadEvent> {
    // Subagent chatter stays in `agent_events` (Timeline), not the thread log.
    if is_subagent(ev) {
        return vec![];
    }
    use AgentEvent as A;
    let opens_turn = matches!(
        ev,
        A::SessionStarted { .. }
            | A::TextDelta { .. }
            | A::ThinkingDelta { .. }
            | A::Message { .. }
            | A::ToolCall { .. }
            | A::ToolResult { .. }
            | A::PermissionRequest { .. }
    );
    let mut out = vec![];
    if opens_turn && !state.turn_open() {
        out.push(ThreadEvent::TurnStarted { trigger: TurnTrigger::Send });
    }
    match ev {
        A::SessionStarted { session_id } => out.push(ThreadEvent::SessionLinked {
            backend_session_id: session_id.clone(),
        }),
        // Live-only: the actor keeps the partial; at most they open the turn above.
        A::TextDelta { .. } | A::ThinkingDelta { .. } | A::Raw { .. } => {}
        A::Message { role, text, .. } => out.push(ThreadEvent::MessageAppended {
            message: msg(role, text.clone(), ctx),
        }),
        A::ToolCall { id, name, input, .. } => {
            let args = if input.is_null() { json!({}) } else { input.clone() };
            let mut m = ChatMessage::assistant_tool_call(id, name, &args, Some(ctx.thinking.clone()));
            m.timestamp = Some(ctx.now.clone());
            out.push(ThreadEvent::MessageAppended { message: m });
        }
        A::ToolResult { tool_use_id, content, is_error } => {
            // `isToolResultFailure` detects failure by this prefix.
            let text = if *is_error { format!("Error: {content}") } else { content.clone() };
            let mut m = ChatMessage::tool_result(tool_use_id, text);
            m.timestamp = Some(ctx.now.clone());
            out.push(ThreadEvent::MessageAppended { message: m });
        }
        A::SubagentStarted { .. } | A::SubagentFinished { .. } => {
            let count = if matches!(ev, A::SubagentStarted { .. }) {
                state.subagents_running + 1
            } else {
                state.subagents_running.saturating_sub(1)
            };
            if count != state.subagents_running {
                out.push(ThreadEvent::SubagentsRunning { count });
            }
        }
        A::PermissionRequest { request_id, tool_name, tool_input, .. } => {
            out.push(ThreadEvent::ApprovalRequested {
                approval: PendingApproval {
                    call_id: request_id.clone(),
                    tool: tool_name.clone(),
                    arguments: tool_input.clone(),
                    suggested_rule: String::new(),
                    summary: summarize(tool_name, tool_input),
                },
            });
        }
        A::PermissionResolved { request_id, allowed } => {
            if state.approvals.iter().any(|a| &a.call_id == request_id) {
                out.push(ThreadEvent::ApprovalResolved {
                    call_id: request_id.clone(),
                    allow: *allowed,
                    auto: false,
                });
            }
        }
        // A JSON null reads back as `None`, so replayed state would differ from live.
        A::Todos { items } => {
            if !items.is_null() {
                out.push(ThreadEvent::TodosUpdated { items: items.clone() });
            }
        }
        A::Usage { total_cost_usd, model_usage } => {
            let model_usage = model_usage.clone().filter(|v| !v.is_null());
            if total_cost_usd.is_some() || model_usage.is_some() {
                out.push(ThreadEvent::UsageUpdated { total_cost_usd: *total_cost_usd, model_usage });
            }
        }
        A::TurnDone { final_text, is_error } => {
            if state.turn_open() {
                // A user stop wins: Codex reports an interrupt as a clean `turn/completed`.
                let outcome = if state.stopping {
                    TurnOutcome::Stopped
                } else if *is_error {
                    TurnOutcome::Error
                } else {
                    TurnOutcome::Completed
                };
                out.push(ThreadEvent::TurnEnded {
                    outcome,
                    error: (outcome == TurnOutcome::Error)
                        .then(|| final_text.clone().unwrap_or_else(|| "turn failed".into())),
                    final_text: final_text.clone(),
                });
            }
        }
        A::Exited { code } => {
            if state.turn_open() {
                out.push(ThreadEvent::TurnEnded {
                    outcome: if state.stopping { TurnOutcome::Stopped } else { TurnOutcome::Interrupted },
                    error: None,
                    final_text: None,
                });
            }
            out.push(ThreadEvent::ProcessExited { code: *code });
        }
    }
    out
}
