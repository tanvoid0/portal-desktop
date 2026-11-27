use std::sync::{Arc, Mutex};
use std::time::Duration;

use sea_orm::{ActiveValue::Set, EntityTrait};
use serde_json::{json, Value};

use super::apply::apply;
use super::*;
use crate::database::DatabaseManager;
use crate::domains::agents::entities::agent_session;
use crate::domains::agents::AgentEvent as A;
use crate::events::EventSink;

#[derive(Debug, Default)]
struct Capture(Mutex<Vec<(String, Value)>>);
impl EventSink for Capture {
    fn emit_json(&self, event: &str, payload: Value) -> bool {
        self.0.lock().unwrap().push((event.to_string(), payload));
        true
    }
}

const ID: &str = "run-1";

async fn setup() -> (Arc<ThreadHub>, Arc<DatabaseManager>, Arc<Capture>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db = Arc::new(DatabaseManager::new(dir.path().to_path_buf()).await.unwrap());
    agent_session::Entity::insert(agent_session::ActiveModel {
        id: Set(ID.into()),
        backend_id: Set("claude-code".into()),
        model: Set(Some("opus".into())),
        cwd: Set("/w".into()),
        status: Set("running".into()),
        started_at: Set("2026-01-01T00:00:00Z".into()),
        ended_at: Set(None),
        exit_code: Set(None),
        title: Set(None),
        task_id: Set(None),
        backend_session_id: Set(None),
    })
    .exec(db.get_connection())
    .await
    .unwrap();
    let sink = Arc::new(Capture::default());
    (ThreadHub::new(db.clone(), sink.clone()), db, sink, dir)
}

fn feed(hub: &Arc<ThreadHub>, events: Vec<A>) {
    for e in events {
        hub.submit(ID, ThreadCommand::Provider(e));
    }
}

fn td(text: &str) -> A {
    A::TextDelta { text: text.into(), parent_tool_use_id: None }
}

fn script() -> Vec<A> {
    vec![
        A::Message { role: "user".into(), text: "go".into(), parent_tool_use_id: None },
        A::SessionStarted { session_id: "sess".into() },
        td("he"),
        td("llo"),
        A::Message { role: "assistant".into(), text: "hello".into(), parent_tool_use_id: None },
        A::ThinkingDelta { text: "hmm".into(), parent_tool_use_id: None },
        A::ToolCall { id: "t1".into(), name: "Bash".into(), input: json!({"command": "ls"}), parent_tool_use_id: None },
        A::PermissionRequest { request_id: "r1".into(), tool_name: "Bash".into(), tool_input: json!({"command": "ls"}), suggestions: vec![] },
        A::PermissionResolved { request_id: "r1".into(), allowed: true },
        A::PermissionResolved { request_id: "r1".into(), allowed: true }, // already resolved: no event
        A::ToolResult { tool_use_id: "t1".into(), content: "boom".into(), is_error: true },
        A::Usage { total_cost_usd: Some(0.5), model_usage: None },
        A::Todos { items: json!([{"a": 1}]) },
        A::Todos { items: Value::Null },                         // null: no event, replay-safe
        A::Usage { total_cost_usd: None, model_usage: Some(Value::Null) },
        A::SubagentStarted { tool_use_id: "s".into(), name: None },
        A::SubagentFinished { tool_use_id: "s".into() },
        A::Message { role: "assistant".into(), text: "done".into(), parent_tool_use_id: None },
        A::TurnDone { final_text: Some("done".into()), is_error: false },
        A::Exited { code: Some(0) },
    ]
}

#[tokio::test]
async fn replay_matches_projection() {
    let (hub, _db, sink, _dir) = setup().await;
    feed(&hub, script());
    let snap = hub.snapshot(ID).await.unwrap().unwrap();

    let page = hub.events_since(ID, 0).await.unwrap();
    assert!(!page.reset);
    let mut state = ThreadState::default();
    for (i, f) in page.events.iter().enumerate() {
        assert_eq!(f.seq, i as i64 + 1, "seqs are contiguous");
        apply(&mut state, &f.event, &f.actor);
        state.last_seq = f.seq;
    }
    let mut snap_state = snap.state.clone();
    snap_state.messages = snap.messages.clone();
    assert_eq!(state, snap_state);
    assert_eq!(state.messages, snap.messages);
    assert_eq!(snap.seq, page.head_seq);

    // Spot-check the fold itself, not just replay-vs-projection.
    assert_eq!(state.status, ThreadStatus::Idle);
    assert!(state.approvals.is_empty());
    assert_eq!(state.backend_session_id.as_deref(), Some("sess"));
    assert_eq!(state.usage.total_cost_usd, Some(0.5));
    // user, hello, tool call (carries the thinking), tool result, done
    assert_eq!(state.messages.len(), 5);
    assert_eq!(state.messages[0].role, "user");
    assert_eq!(state.messages[2].content.as_deref(), Some("hmm"));
    assert_eq!(state.messages[3].content.as_deref(), Some("Error: boom"));
    // Deltas are live-only: coalesced frames went out, none were persisted.
    let frames = sink.0.lock().unwrap();
    assert!(frames.iter().any(|(n, _)| n == "coder://thread-delta"));
    let events = frames.iter().filter(|(n, _)| n == "coder://thread-event").count();
    assert_eq!(events, page.events.len());
}

// ---- S2a: client commands, dedup, first-wins, queue, effects ----

#[derive(Default)]
struct Rec(Mutex<Vec<(String, Effect)>>);
#[async_trait::async_trait]
impl ThreadEffects for Rec {
    async fn run(&self, thread_id: &str, effect: Effect) {
        self.0.lock().unwrap().push((thread_id.to_string(), effect));
    }
}

fn record(hub: &Arc<ThreadHub>) -> Arc<Rec> {
    let rec = Arc::new(Rec::default());
    hub.register_effects(rec.clone(), rec.clone());
    rec
}

/// Effects run in spawned tasks; give them a turn.
async fn effects(rec: &Rec) -> Vec<Effect> {
    tokio::time::sleep(Duration::from_millis(50)).await;
    rec.0.lock().unwrap().iter().map(|(_, e)| e.clone()).collect()
}

async fn cmd(hub: &Arc<ThreadHub>, id: &str, c: ThreadCommand, actor: &str, cid: Option<&str>) -> CommandAck {
    let env = Envelope { cmd: c, actor: actor.into(), command_id: cid.map(Into::into) };
    hub.command(id, env).await.unwrap()
}

fn send(text: &str) -> ThreadCommand {
    ThreadCommand::Send { text: text.into() }
}

fn approve(call_id: &str, allow: bool) -> ThreadCommand {
    ThreadCommand::Approve { call_id: call_id.into(), allow, remember: false, edited_pattern: None, edited_command: None }
}

fn perm(id: &str) -> A {
    A::PermissionRequest { request_id: id.into(), tool_name: "Bash".into(), tool_input: json!({"command": "ls"}), suggestions: vec![] }
}

fn say(role: &str, text: &str) -> crate::domains::coder::types::ChatMessage {
    crate::domains::coder::types::ChatMessage { role: role.into(), content: Some(text.into()), tool_calls: None, tool_call_id: None, timestamp: None }
}

async fn create_portal(hub: &Arc<ThreadHub>, id: &str) {
    let meta = ThreadMeta { provider: "portal".into(), workspace_root: "/w".into(), kind: "chat".into(), title: "New".into(), ..Default::default() };
    assert_eq!(cmd(hub, id, ThreadCommand::Create(meta), "local", None).await.status, AckStatus::Accepted);
}

async fn kinds(hub: &Arc<ThreadHub>, id: &str) -> Vec<String> {
    let page = hub.events_since(id, 0).await.unwrap();
    page.events.iter().map(|f| serde_json::to_value(&f.event).unwrap()["type"].as_str().unwrap().to_string()).collect()
}

#[tokio::test]
async fn double_approve_applies_once() {
    let (hub, _db, _sink, _dir) = setup().await;
    let rec = record(&hub);
    feed(&hub, vec![perm("r1")]);
    let first = cmd(&hub, ID, approve("r1", true), "local", Some("c1")).await;
    assert_eq!(first.status, AckStatus::Accepted);
    let second = cmd(&hub, ID, approve("r1", false), "device:phone", Some("c2")).await;
    assert_eq!(second.status, AckStatus::Rejected);
    assert_eq!(second.code.as_deref(), Some("already_resolved"));
    assert_eq!(second.resolved_by.as_deref(), Some("local"));
    assert_eq!(second.allow, Some(true));
    // The provider's own late answer is dropped too.
    feed(&hub, vec![A::PermissionResolved { request_id: "r1".into(), allowed: true }]);
    assert_eq!(kinds(&hub, ID).await.iter().filter(|k| *k == "approval_resolved").count(), 1);
    let fx = effects(&rec).await;
    assert_eq!(fx.len(), 1);
    assert!(matches!(&fx[0], Effect::ResolveApproval { call_id, allow: true, .. } if call_id == "r1"));
}

#[tokio::test]
async fn duplicate_command_id_is_noop() {
    let (hub, _db, _sink, _dir) = setup().await;
    let rec = record(&hub);
    let a = cmd(&hub, ID, send("hi"), "local", Some("same")).await;
    let b = cmd(&hub, ID, send("hi"), "local", Some("same")).await;
    assert_eq!((a.status, b.status), (AckStatus::Accepted, AckStatus::Duplicate));
    assert_eq!(a.seq, b.seq);
    assert_eq!(hub.snapshot(ID).await.unwrap().unwrap().messages.len(), 1);
    assert_eq!(effects(&rec).await.len(), 1);
}

#[tokio::test]
async fn send_while_running_enqueues() {
    let (hub, _db, _sink, _dir) = setup().await;
    feed(&hub, vec![td("busy")]);
    let ack = cmd(&hub, ID, send("later"), "local", None).await;
    assert_eq!(ack.status, AckStatus::Accepted);
    let snap = hub.snapshot(ID).await.unwrap().unwrap();
    assert_eq!(snap.state.queue.len(), 1);
    assert_eq!(snap.state.queue[0].text, "later");
    assert!(snap.messages.is_empty());
    // Removing it is a command too; an unknown id is rejected.
    let qid = snap.state.queue[0].queue_id.clone();
    assert_eq!(cmd(&hub, ID, ThreadCommand::RemoveQueued { queue_id: "nope".into() }, "local", None).await.status, AckStatus::Rejected);
    assert_eq!(cmd(&hub, ID, ThreadCommand::RemoveQueued { queue_id: qid }, "local", None).await.status, AckStatus::Accepted);
    assert!(hub.snapshot(ID).await.unwrap().unwrap().state.queue.is_empty());
}

#[tokio::test]
async fn queue_survives_restart() {
    let (hub, db, sink, _dir) = setup().await;
    feed(&hub, vec![td("busy")]);
    cmd(&hub, ID, send("later"), "local", None).await;

    let reborn = ThreadHub::new(db, sink);
    let rec = record(&reborn);
    assert_eq!(reborn.snapshot(ID).await.unwrap().unwrap().state.queue.len(), 1);
    reborn.submit(ID, ThreadCommand::Provider(A::TurnDone { final_text: None, is_error: false }));
    reborn.snapshot(ID).await.unwrap(); // the drain lands behind this one
    let snap = reborn.snapshot(ID).await.unwrap().unwrap();
    assert!(snap.state.queue.is_empty());
    assert_eq!(snap.messages.last().unwrap().content.as_deref(), Some("later"));
    assert_eq!(snap.state.status, ThreadStatus::Running);
    assert_eq!(effects(&rec).await, [Effect::StartTurn { trigger: TurnTrigger::Queue, text: "later".into() }]);
}

#[tokio::test]
async fn reconcile_tail_only() {
    let (hub, _db, _sink, _dir) = setup().await;
    record(&hub);
    create_portal(&hub, "p1").await;
    cmd(&hub, "p1", send("hi"), "local", None).await;
    let portal = |messages, title: Option<&str>| {
        ThreadCommand::Portal(PortalInput::Reconcile { messages, title: title.map(Into::into) })
    };
    let msgs = vec![say("user", "hi"), say("assistant", "yo")];
    cmd(&hub, "p1", portal(msgs.clone(), Some("Greeting")), "system", None).await;
    let page = hub.events_since("p1", 0).await.unwrap();
    let rewrites: Vec<_> = page.events.iter().filter(|f| matches!(f.event, ThreadEvent::MessagesRewritten { .. })).collect();
    assert_eq!(rewrites.len(), 1);
    assert!(matches!(&rewrites[0].event, ThreadEvent::MessagesRewritten { keep: 1, tail } if tail.len() == 1));
    assert!(page.events.iter().all(|f| f.provider == "portal"));
    // Same list and title again: nothing to write.
    let head = page.head_seq;
    cmd(&hub, "p1", portal(msgs, Some("Greeting")), "system", None).await;
    assert_eq!(hub.events_since("p1", 0).await.unwrap().head_seq, head);
    let snap = hub.snapshot("p1").await.unwrap().unwrap();
    assert_eq!(snap.thread.title, "Greeting");
    assert_eq!(snap.messages.len(), 2);
    // A shorter list truncates (the projection rows follow).
    cmd(&hub, "p1", portal(vec![say("user", "hi")], None), "system", None).await;
    assert_eq!(hub.snapshot("p1").await.unwrap().unwrap().thread.message_count, 1);
}

#[tokio::test]
async fn stop_while_idle_rejected() {
    let (hub, _db, _sink, _dir) = setup().await;
    let ack = cmd(&hub, ID, ThreadCommand::Stop, "local", None).await;
    assert_eq!(ack.status, AckStatus::Rejected);
    assert_eq!(ack.code.as_deref(), Some("not_running"));
    assert_eq!(kinds(&hub, ID).await, ["created"], "a rejected command writes nothing");
}

#[tokio::test]
async fn stop_and_heartbeat() {
    let (hub, _db, sink, _dir) = setup().await;
    let rec = record(&hub);
    feed(&hub, vec![td("x")]);
    assert_eq!(cmd(&hub, ID, ThreadCommand::Stop, "local", None).await.status, AckStatus::Accepted);
    assert!(hub.snapshot(ID).await.unwrap().unwrap().state.stopping);
    feed(&hub, vec![A::Exited { code: None }]);
    assert!(!hub.snapshot(ID).await.unwrap().unwrap().state.stopping);
    assert_eq!(effects(&rec).await, [Effect::Cancel]);
    cmd(&hub, ID, ThreadCommand::Portal(PortalInput::Heartbeat { waited_seconds: 7 }), "system", None).await;
    assert_eq!(hub.snapshot(ID).await.unwrap().unwrap().waited_seconds, 7);
    assert!(sink.0.lock().unwrap().iter().any(|(n, p)| n == "coder://thread-heartbeat" && p["waited_seconds"] == 7));
}

#[tokio::test]
async fn replay_matches_portal_turn() {
    let (hub, _db, _sink, _dir) = setup().await;
    record(&hub);
    let id = "p-replay";
    create_portal(&hub, id).await;
    cmd(&hub, id, send("hi"), "local", Some("c1")).await;
    cmd(&hub, id, send("next"), "device:phone", Some("c2")).await; // running: queued
    let reconcile = |messages| ThreadCommand::Portal(PortalInput::Reconcile { messages, title: Some("T".into()) });
    cmd(&hub, id, reconcile(vec![say("user", "hi"), say("assistant", "yo")]), "system", None).await;
    let pending = crate::domains::coder::types::PendingApproval {
        call_id: "a1".into(), tool: "run".into(), arguments: json!({}), suggested_rule: String::new(), summary: "ls".into(),
    };
    cmd(&hub, id, ThreadCommand::Portal(PortalInput::Pending(pending)), "system", None).await;
    assert_eq!(cmd(&hub, id, approve("a1", true), "local", Some("c3")).await.status, AckStatus::Accepted);
    assert_eq!(cmd(&hub, id, approve("a1", false), "device:phone", Some("c4")).await.status, AckStatus::Rejected);
    let host = ThreadCommand::Portal(PortalInput::HostRequest { call_id: "h1".into(), kind: "ask_user".into(), payload: json!({"q": 1}) });
    cmd(&hub, id, host, "system", None).await;
    let ack = cmd(&hub, id, ThreadCommand::ResolveHost { call_id: "h1".into(), result: json!("ok") }, "device:phone", Some("c5")).await;
    assert_eq!(ack.status, AckStatus::Accepted);
    let ack = cmd(&hub, id, ThreadCommand::ResolveHost { call_id: "h1".into(), result: json!("late") }, "local", Some("c6")).await;
    assert_eq!(ack.resolved_by.as_deref(), Some("device:phone"));
    cmd(&hub, id, ThreadCommand::SetModel { model: Some("m".into()), llm_provider: Some("p".into()) }, "local", None).await;
    cmd(&hub, id, ThreadCommand::ChangeRecorded { change_id: "ch".into(), path: "a.rs".into(), status: "pending".into() }, "system", None).await;
    let finish = ThreadCommand::Portal(PortalInput::Finished { outcome: TurnOutcome::Completed, final_text: None });
    cmd(&hub, id, finish.clone(), "system", None).await;
    hub.snapshot(id).await.unwrap(); // the queue drain lands behind this one
    cmd(&hub, id, ThreadCommand::Portal(PortalInput::Heartbeat { waited_seconds: 3 }), "system", None).await;
    cmd(&hub, id, finish, "system", None).await;
    cmd(&hub, id, ThreadCommand::Retry, "local", None).await;
    cmd(&hub, id, ThreadCommand::Portal(PortalInput::Failed { error: "boom".into() }), "system", None).await;

    let snap = hub.snapshot(id).await.unwrap().unwrap();
    let page = hub.events_since(id, 0).await.unwrap();
    let mut state = ThreadState::default();
    for (i, f) in page.events.iter().enumerate() {
        assert_eq!(f.seq, i as i64 + 1);
        apply(&mut state, &f.event, &f.actor);
        state.last_seq = f.seq;
    }
    let mut live = snap.state.clone();
    live.messages = snap.messages.clone();
    assert_eq!(state, live);
    assert_eq!(state.status, ThreadStatus::Error);
    assert_eq!(state.meta.model.as_deref(), Some("m"));
    assert_eq!(state.recently_resolved.len(), 2);
    assert!(state.queue.is_empty() && state.host_requests.is_empty() && state.approvals.is_empty());
    // user, yo, next (drained), then Retry cut nothing: the last user message is "next".
    assert_eq!(state.messages.last().unwrap().content.as_deref(), Some("next"));
}

fn finished(outcome: TurnOutcome) -> ThreadCommand {
    ThreadCommand::Portal(PortalInput::Finished { outcome, final_text: None })
}

fn pending(id: &str) -> ThreadCommand {
    let p = crate::domains::coder::types::PendingApproval {
        call_id: id.into(), tool: "run".into(), arguments: json!({}), suggested_rule: String::new(), summary: "ls".into(),
    };
    ThreadCommand::Portal(PortalInput::Pending(p))
}

#[tokio::test]
async fn stopped_turn_drops_its_approvals() {
    let (hub, _db, _sink, _dir) = setup().await;
    let rec = record(&hub);
    create_portal(&hub, "p-stop").await;
    cmd(&hub, "p-stop", send("go"), "local", None).await;
    cmd(&hub, "p-stop", pending("a1"), "system", None).await;
    cmd(&hub, "p-stop", ThreadCommand::Stop, "local", None).await;
    cmd(&hub, "p-stop", finished(TurnOutcome::Stopped), "system", None).await;
    let late = cmd(&hub, "p-stop", approve("a1", true), "local", None).await;
    assert_eq!(late.status, AckStatus::Rejected);
    // The next turn is not stuck waiting on the old call.
    cmd(&hub, "p-stop", send("again"), "local", None).await;
    cmd(&hub, "p-stop", pending("a2"), "system", None).await;
    assert_eq!(cmd(&hub, "p-stop", approve("a2", true), "local", None).await.status, AckStatus::Accepted);
    assert_eq!(hub.snapshot("p-stop").await.unwrap().unwrap().state.status, ThreadStatus::Running);
    let fx = effects(&rec).await;
    assert!(!fx.iter().any(|e| matches!(e, Effect::ResolveApproval { call_id, .. } if call_id == "a1")));
}

#[tokio::test]
async fn clean_turn_done_after_stop_is_stopped_and_keeps_the_queue() {
    let (hub, _db, _sink, _dir) = setup().await;
    record(&hub);
    feed(&hub, vec![td("busy")]);
    cmd(&hub, ID, send("later"), "local", None).await;
    cmd(&hub, ID, ThreadCommand::Stop, "local", None).await;
    feed(&hub, vec![A::TurnDone { final_text: None, is_error: false }]);
    let snap = hub.snapshot(ID).await.unwrap().unwrap();
    let page = hub.events_since(ID, 0).await.unwrap();
    assert!(page.events.iter().any(|f| matches!(f.event, ThreadEvent::TurnEnded { outcome: TurnOutcome::Stopped, .. })));
    assert_eq!(snap.state.queue.len(), 1);
    assert!(snap.messages.is_empty());
    assert_eq!(snap.state.status, ThreadStatus::Idle);
}

#[tokio::test]
async fn repeated_stop_cancels_again() {
    let (hub, _db, _sink, _dir) = setup().await;
    let rec = record(&hub);
    feed(&hub, vec![td("x")]);
    let a = cmd(&hub, ID, ThreadCommand::Stop, "local", None).await;
    let b = cmd(&hub, ID, ThreadCommand::Stop, "local", None).await;
    assert_eq!((a.status, b.status), (AckStatus::Accepted, AckStatus::Accepted));
    assert_eq!(a.seq, b.seq, "the repeat writes nothing");
    assert_eq!(effects(&rec).await, [Effect::Cancel, Effect::Cancel]);
}

#[tokio::test]
async fn edit_clears_the_queue() {
    let (hub, _db, _sink, _dir) = setup().await;
    record(&hub);
    create_portal(&hub, "p-edit").await;
    cmd(&hub, "p-edit", send("hi"), "local", None).await;
    cmd(&hub, "p-edit", send("queued"), "local", None).await;
    let reconcile = ThreadCommand::Portal(PortalInput::Reconcile { messages: vec![say("user", "hi"), say("assistant", "yo")], title: None });
    cmd(&hub, "p-edit", reconcile, "system", None).await;
    cmd(&hub, "p-edit", finished(TurnOutcome::Stopped), "system", None).await;
    let ack = cmd(&hub, "p-edit", ThreadCommand::EditMessage { index: 0, text: "hi2".into() }, "local", None).await;
    assert_eq!(ack.status, AckStatus::Accepted);
    let snap = hub.snapshot("p-edit").await.unwrap().unwrap();
    assert!(snap.state.queue.is_empty());
    assert_eq!(snap.messages.len(), 1);
    assert!(kinds(&hub, "p-edit").await.contains(&"dequeued".to_string()));
}

#[tokio::test]
async fn reconcile_normalises_missing_content() {
    let (hub, db, sink, _dir) = setup().await;
    record(&hub);
    create_portal(&hub, "p-norm").await;
    cmd(&hub, "p-norm", send("hi"), "local", None).await;
    let mut call = crate::domains::coder::types::ChatMessage::assistant_tool_call("t", "Bash", &json!({}), None);
    call.content = None;
    let reconcile = ThreadCommand::Portal(PortalInput::Reconcile { messages: vec![say("user", "hi"), call], title: None });
    cmd(&hub, "p-norm", reconcile, "system", None).await;
    let live = hub.snapshot("p-norm").await.unwrap().unwrap();
    assert_eq!(live.messages[1].content.as_deref(), Some(""));
    let reloaded = ThreadHub::new(db, sink).snapshot("p-norm").await.unwrap().unwrap();
    assert_eq!(live.messages, reloaded.messages);
}

#[tokio::test]
async fn late_join_sees_partial() {
    let (hub, _db, _sink, _dir) = setup().await;
    feed(&hub, vec![td("par"), td("tial")]);
    let snap = hub.snapshot(ID).await.unwrap().unwrap();
    assert_eq!(snap.partial_text, "partial");
    assert_eq!(snap.state.status, ThreadStatus::Running);
    // The message supersedes the partial.
    feed(&hub, vec![A::Message { role: "assistant".into(), text: "partial".into(), parent_tool_use_id: None }]);
    assert_eq!(hub.snapshot(ID).await.unwrap().unwrap().partial_text, "");
}

#[tokio::test]
async fn subagent_events_dropped() {
    let (hub, _db, _sink, _dir) = setup().await;
    let p = Some("parent".to_string());
    feed(
        &hub,
        vec![
            A::TextDelta { text: "x".into(), parent_tool_use_id: p.clone() },
            A::Message { role: "assistant".into(), text: "sub".into(), parent_tool_use_id: p.clone() },
            A::ToolCall { id: "c".into(), name: "Read".into(), input: Value::Null, parent_tool_use_id: p },
        ],
    );
    let snap = hub.snapshot(ID).await.unwrap().unwrap();
    assert!(snap.messages.is_empty());
    assert_eq!(snap.partial_text, "");
    assert_eq!(snap.state.status, ThreadStatus::Idle);
}

#[tokio::test]
async fn events_since_resets_on_gap_or_future_seq() {
    let (hub, _db, _sink, _dir) = setup().await;
    feed(&hub, script());
    let head = hub.snapshot(ID).await.unwrap().unwrap().seq;
    let tail = hub.events_since(ID, head - 2).await.unwrap();
    assert!(!tail.reset);
    assert_eq!(tail.events.len(), 2);
    assert!(hub.events_since(ID, head + 1).await.unwrap().reset);
    assert!(hub.events_since("nope", 5).await.unwrap().reset);
}

#[tokio::test]
async fn hydration_reloads_the_same_state() {
    let (hub, db, sink, _dir) = setup().await;
    feed(&hub, script());
    let before = hub.snapshot(ID).await.unwrap().unwrap();
    // A fresh hub on the same database is what an evicted actor's next message sees.
    let reborn = ThreadHub::new(db, sink);
    let after = reborn.snapshot(ID).await.unwrap().unwrap();
    assert_eq!(before.state, after.state);
    assert_eq!(before.messages, after.messages);
    assert_eq!(reborn.list().await.unwrap().len(), 1);
    // And it keeps appending where it left off.
    reborn.submit(ID, ThreadCommand::Provider(A::SessionStarted { session_id: "again".into() }));
    let resumed = reborn.snapshot(ID).await.unwrap().unwrap();
    assert!(resumed.seq > before.seq);
    assert_eq!(resumed.state.status, ThreadStatus::Running);
}

#[tokio::test]
async fn delete_drops_the_thread_and_its_actor() {
    let (hub, _db, _sink, _dir) = setup().await;
    feed(&hub, script());
    assert!(hub.snapshot(ID).await.unwrap().is_some());
    hub.delete(ID).await.unwrap();
    assert!(hub.list().await.unwrap().is_empty());
    assert!(hub.snapshot(ID).await.unwrap().is_none());
    assert!(hub.events_since(ID, 0).await.unwrap().events.is_empty());
}


#[tokio::test]
async fn partial_offset_counts_utf16_units() {
    let (hub, _db, sink, _dir) = setup().await;
    feed(&hub, vec![td("a😀")]);
    hub.snapshot(ID).await.unwrap(); // flushes the pending delta
    feed(&hub, vec![td("b")]);
    hub.snapshot(ID).await.unwrap();
    let frames = sink.0.lock().unwrap();
    let offsets: Vec<_> = frames
        .iter()
        .filter(|(n, _)| n == "coder://thread-delta")
        .map(|(_, p)| p["offset"].as_u64().unwrap())
        .collect();
    assert_eq!(offsets, [0, 3]); // "a" + a surrogate pair = 3 JS units, not 2 chars
}

#[tokio::test]
async fn user_message_mid_turn_keeps_streamed_thinking() {
    let (hub, _db, _sink, _dir) = setup().await;
    feed(
        &hub,
        vec![
            A::ThinkingDelta { text: "hmm".into(), parent_tool_use_id: None },
            A::Message { role: "user".into(), text: "wait".into(), parent_tool_use_id: None },
            A::ToolCall { id: "t".into(), name: "Bash".into(), input: json!({}), parent_tool_use_id: None },
        ],
    );
    let snap = hub.snapshot(ID).await.unwrap().unwrap();
    assert_eq!(snap.messages[0].role, "user");
    assert_eq!(snap.messages[1].content.as_deref(), Some("hmm"));
}

#[tokio::test]
async fn a_failed_commit_reloads_and_recovers() {
    use crate::domains::threads::entities::thread_event;
    let (hub, db, _sink, _dir) = setup().await;
    feed(&hub, vec![A::SessionStarted { session_id: "s".into() }]);
    let before = hub.snapshot(ID).await.unwrap().unwrap();
    let say = |t: &str| A::Message { role: "assistant".into(), text: t.into(), parent_tool_use_id: None };

    // A stray row at the next seq makes the actor's commit fail on the primary key.
    let rogue = thread_event::ActiveModel {
        thread_id: Set(ID.into()),
        seq: Set(before.seq + 1),
        kind: Set("rogue".into()),
        payload: Set("{}".into()),
        command_id: Set(None),
        actor: Set("test".into()),
        at: Set("2026-01-01T00:00:00Z".into()),
    };
    thread_event::Entity::insert(rogue).exec(db.get_connection()).await.unwrap();
    feed(&hub, vec![say("lost")]);
    let after = hub.snapshot(ID).await.unwrap().unwrap();
    assert_eq!((after.seq, after.messages.len()), (before.seq, before.messages.len()));
    assert_eq!(after.state, before.state, "memory was rolled back to the stored state");

    thread_event::Entity::delete_by_id((ID.to_string(), before.seq + 1)).exec(db.get_connection()).await.unwrap();
    feed(&hub, vec![say("kept")]);
    let healed = hub.snapshot(ID).await.unwrap().unwrap();
    assert_eq!(healed.seq, before.seq + 1);
    assert_eq!(healed.messages.last().unwrap().content.as_deref(), Some("kept"));
}
