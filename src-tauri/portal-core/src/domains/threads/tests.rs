use std::sync::{Arc, Mutex};

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
        apply(&mut state, &f.event);
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
