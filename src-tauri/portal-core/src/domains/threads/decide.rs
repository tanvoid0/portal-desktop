//! `decide`: command + state -> events. Pure and deterministic (time and the
//! actor's partial thinking arrive through `Ctx`). The `AgentEvent` fold here is
//! the one `agents/utils/eventsToMessages.ts` used to do on the client.

use serde_json::{json, Value};

use super::types::*;
use crate::domains::agents::AgentEvent;
use crate::domains::coder::types::{ChatMessage, PendingApproval};

pub fn decide(state: &ThreadState, cmd: &ThreadCommand, ctx: &Ctx) -> Result<Vec<ThreadEvent>, Reject> {
    match cmd {
        ThreadCommand::Create(_) if state.created() => Err(Reject::AlreadyCreated),
        ThreadCommand::Create(m) => Ok(vec![ThreadEvent::Created {
            provider: m.provider.clone(),
            model: m.model.clone(),
            llm_provider: m.llm_provider.clone(),
            workspace_root: m.workspace_root.clone(),
            project_id: m.project_id,
            kind: m.kind.clone(),
            title: m.title.clone(),
        }]),
        ThreadCommand::Provider(_) if !state.created() => Err(Reject::NotCreated),
        ThreadCommand::Provider(ev) => Ok(provider(state, ev, ctx)),
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
                out.push(ThreadEvent::TurnEnded {
                    outcome: if *is_error { TurnOutcome::Error } else { TurnOutcome::Completed },
                    error: is_error
                        .then(|| final_text.clone().unwrap_or_else(|| "turn failed".into())),
                    final_text: final_text.clone(),
                });
            }
        }
        A::Exited { code } => {
            if state.turn_open() {
                out.push(ThreadEvent::TurnEnded {
                    outcome: TurnOutcome::Interrupted,
                    error: None,
                    final_text: None,
                });
            }
            out.push(ThreadEvent::ProcessExited { code: *code });
        }
    }
    out
}
