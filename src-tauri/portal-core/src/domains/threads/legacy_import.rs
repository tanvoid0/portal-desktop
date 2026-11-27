//! Idempotent startup importer of `coder_threads` / `agent_sessions` into the thread
//! log (design section 6), plus the startup reconcile of threads a dead process left
//! open. Runs before any service can submit to the hub, so the import is the only
//! writer; the reconcile goes through the hub like any other command.
//!
//! Projections only: `threads` + `thread_messages` (`seq` 0), `last_seq` 0, no
//! synthetic events. The old tables are never written. Agent sessions are folded with
//! `decide`/`apply`, the same fold the hub runs live (mirrors `detailToLog`).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};

use super::apply::apply;
use super::decide::decide;
use super::entities::{thread, thread_message};
use super::hub::ThreadHub;
use super::store;
use super::types::*;
use crate::domains::agents::entities::{agent_event, agent_message, agent_session};
use crate::domains::agents::AgentEvent;
use crate::domains::coder::entities::coder_thread;
use crate::domains::coder::types::ChatMessage;
use crate::log_warn;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Threads written from scratch (or re-written while still `last_seq = 0`).
    pub imported: usize,
    /// Hub-created threads that got legacy history prepended.
    pub prepended: usize,
    /// Open turns closed because their process is gone.
    pub reconciled: usize,
}

fn err(what: &str, e: impl std::fmt::Display) -> String {
    format!("threads: legacy import: {what} failed: {e}")
}

/// Import, then reconcile through the hub (the single writer). A failed import is
/// logged and never blocks the reconcile; the next startup retries it.
pub async fn run(db: &DatabaseConnection, hub: &Arc<ThreadHub>) -> Result<Report, String> {
    let mut report = Report::default();
    if let Err(e) = import(db, &mut report).await {
        log_warn!("Threads", "{e}");
    }
    report.reconciled = reconcile(db, hub).await?;
    Ok(report)
}

/// What the importer needs to know about an existing thread.
struct Known {
    last_seq: i64,
    created_at: String,
    updated_at: String,
}

enum Action {
    Skip,
    /// No row, or `last_seq = 0` and the legacy row is newer: write it whole.
    Import,
    /// The hub created it after the legacy history began: put that history in front.
    Prepend,
}

fn action(known: Option<&Known>, legacy_updated: &str, predates: impl Fn(&Known) -> bool) -> Action {
    match known {
        None => Action::Import,
        Some(k) if k.last_seq == 0 => {
            if legacy_updated > k.updated_at.as_str() { Action::Import } else { Action::Skip }
        }
        Some(k) if predates(k) => Action::Prepend,
        Some(_) => Action::Skip,
    }
}

async fn import(db: &DatabaseConnection, report: &mut Report) -> Result<(), String> {
    let known: HashMap<String, Known> = thread::Entity::find()
        .all(db)
        .await
        .map_err(|e| err("list threads", e))?
        .into_iter()
        .map(|t| (t.id, Known { last_seq: t.last_seq, created_at: t.created_at, updated_at: t.updated_at }))
        .collect();

    // A bad row is logged and skipped; the rest still import.
    for row in coder_thread::Entity::find().all(db).await.map_err(|e| err("list coder_threads", e))? {
        // Portal threads are never prepended: the hub only creates one from a row this
        // loop has already imported (`Create`, then a `Reconcile` of the whole transcript).
        if matches!(action(known.get(&row.id), &row.updated_at, |_| false), Action::Import) {
            match import_portal(db, &row).await {
                Ok(()) => report.imported += 1,
                Err(e) => log_warn!("Threads", "{e} (coder thread {})", row.id),
            }
        }
    }

    for s in agent_session::Entity::find().all(db).await.map_err(|e| err("list agent_sessions", e))? {
        // ponytail: ended_at/started_at stand in for "last activity"; a session that
        // gains messages without either moving is picked up when it next settles.
        let stamp = s.ended_at.clone().unwrap_or_else(|| s.started_at.clone());
        let known = known.get(&s.id);
        let done = match action(known, &stamp, |k| s.started_at < k.created_at) {
            Action::Skip => continue,
            Action::Import => import_session(db, &s.id, &s.started_at, &stamp).await.map(|()| true),
            Action::Prepend => prepend_session(db, &s.id, &known.map_or_else(String::new, |k| k.created_at.clone())).await,
        };
        match done {
            Ok(true) if known.is_none_or(|k| k.last_seq == 0) => report.imported += 1,
            Ok(true) => report.prepended += 1,
            Ok(false) => {}
            Err(e) => log_warn!("Threads", "{e} (agent session {})", s.id),
        }
    }
    Ok(())
}

async fn import_portal(db: &DatabaseConnection, row: &coder_thread::Model) -> Result<(), String> {
    let mut messages: Vec<ChatMessage> = serde_json::from_str(&row.messages_json).unwrap_or_default();
    for m in &mut messages {
        // `None` serializes as `""` and reads back `Some("")`: store that form.
        m.content = Some(m.content.take().unwrap_or_default());
    }
    let meta = ThreadMeta {
        provider: "portal".into(),
        model: row.model.clone(),
        llm_provider: row.llm_provider.clone(),
        workspace_root: row.workspace_root.clone(),
        project_id: row.project_id.map(i64::from),
        kind: row.thread_kind.clone(),
        title: row.title.clone(),
    };
    let state = ThreadState { meta, ..Default::default() };
    write_whole(db, &row.id, state, messages, &row.created_at, &row.updated_at).await
}

async fn import_session(db: &DatabaseConnection, id: &str, started_at: &str, stamp: &str) -> Result<(), String> {
    let state = fold_session(db, id, session_log(db, id, None).await?).await?;
    let msgs = state.messages.clone();
    write_whole(db, id, state, msgs, started_at, stamp).await
}

/// A thread the hub created for a run that already had history (a resume of a session
/// that predates the log). True when something was prepended.
///
/// The legacy prompt is persisted before the hub ever sees it, so every hub-created
/// thread has legacy rows just before its `Created`; the longest tail of the legacy
/// fold that equals the start of the hub's own messages is that overlap and is dropped.
///
/// The prepend shifts `idx` but, by design, adds no event and leaves `last_seq` alone:
/// a client holding a pre-restart snapshot gets no reset from `events_since` and keeps
/// a list without the prefix, so clients must re-snapshot after a hub restart (S4).
async fn prepend_session(db: &DatabaseConnection, id: &str, created_at: &str) -> Result<bool, String> {
    // Cheap indexed checks first: already prepended, or nothing beyond the prompt.
    let marked = thread_message::Entity::find()
        .filter(thread_message::Column::ThreadId.eq(id))
        .filter(thread_message::Column::Seq.eq(0))
        .count(db)
        .await
        .map_err(|e| err("count marker", e))?;
    if marked > 0 {
        return Ok(false);
    }
    let before = agent_event::Entity::find()
        .filter(agent_event::Column::SessionId.eq(id))
        .filter(agent_event::Column::CreatedAt.lt(created_at))
        .count(db)
        .await
        .map_err(|e| err("count events", e))?
        + agent_message::Entity::find()
            .filter(agent_message::Column::SessionId.eq(id))
            .filter(agent_message::Column::CreatedAt.lt(created_at))
            .count(db)
            .await
            .map_err(|e| err("count messages", e))?;
    // Invariant: exactly one persisted prompt precedes the hub's first command
    // (`start_run` and `resume_session` persist it before the hub sees it), so more than
    // one legacy row before `Created` is history the hub never saw. Known way to break
    // it: board bug #1976 (the transcript sweep rewriting a resumed session's timestamps).
    if before <= 1 {
        return Ok(false);
    }
    // ponytail: cutoff is the hub's `Created` time; an event persisted in the few ms
    // between its submit and `Created` could land on both sides (the overlap trim covers
    // messages, not other events).
    let log = session_log(db, id, Some(&created_at.to_string())).await?;
    let mut prefix = fold_session(db, id, log).await?.messages;
    let have = load_rows(db, id).await?;
    let mine = decode(&have)?;
    let eq = |a: &ChatMessage, b: &ChatMessage| a.role == b.role && a.content == b.content;
    // The hub's first message is the persisted prompt, so the true overlap is the
    // smallest match of at least one; the largest would eat real history that repeats it.
    let overlap = (1..=prefix.len().min(mine.len()))
        .find(|&j| prefix[prefix.len() - j..].iter().zip(&mine[..j]).all(|(a, b)| eq(a, b)))
        .unwrap_or(0);
    prefix.truncate(prefix.len() - overlap);
    prepend(db, id, &prefix, have).await
}

/// Events plus message rows with no event at their seq (user turns, imported
/// transcripts), seq-ordered; `before` keeps only what was persisted earlier.
async fn session_log(db: &DatabaseConnection, id: &str, before: Option<&String>) -> Result<Vec<(AgentEvent, String)>, String> {
    let events = agent_event::Entity::find()
        .filter(agent_event::Column::SessionId.eq(id))
        .order_by_asc(agent_event::Column::Seq)
        .all(db)
        .await
        .map_err(|e| err("load events", e))?;
    let messages = agent_message::Entity::find()
        .filter(agent_message::Column::SessionId.eq(id))
        .order_by_asc(agent_message::Column::Seq)
        .all(db)
        .await
        .map_err(|e| err("load messages", e))?;
    let seen: HashSet<i64> = events.iter().map(|e| e.seq).collect();
    let mut rows: Vec<(i64, AgentEvent, String)> = events
        .into_iter()
        .filter_map(|e| Some((e.seq, serde_json::from_str(&e.event_json).ok()?, e.created_at)))
        .collect();
    rows.extend(messages.into_iter().filter(|m| !seen.contains(&m.seq)).map(|m| {
        let ev = AgentEvent::Message { role: m.role, text: m.text, parent_tool_use_id: m.parent_tool_use_id };
        (m.seq, ev, m.created_at)
    }));
    rows.sort_by_key(|r| r.0);
    Ok(rows.into_iter().filter(|r| before.is_none_or(|b| r.2 < *b)).map(|r| (r.1, r.2)).collect())
}

/// `decide` + `apply` over the log, as the hub would have run it live.
async fn fold_session(db: &DatabaseConnection, id: &str, log: Vec<(AgentEvent, String)>) -> Result<ThreadState, String> {
    let meta = store::session_meta(db, id).await?.ok_or_else(|| err("session row", id))?;
    let mut state = ThreadState::default();
    let run = |state: &mut ThreadState, cmd: ThreadCommand, at: &str| {
        let ctx = Ctx { now: at.into(), actor: "system".into(), thinking: String::new(), queue_id: String::new() };
        for e in decide(state, &cmd, &ctx).unwrap_or_default() {
            apply(state, &e, "system");
        }
    };
    run(&mut state, ThreadCommand::Create(meta), "");
    for (ev, at) in log {
        run(&mut state, ThreadCommand::Provider(ev), &at);
    }
    // No process survives a restart: a turn the log left open ended with it.
    if state.turn_open() {
        run(&mut state, ThreadCommand::Provider(AgentEvent::Exited { code: None }), "");
    }
    state.subagents_running = 0;
    Ok(state)
}

async fn load_rows(db: &DatabaseConnection, id: &str) -> Result<Vec<thread_message::Model>, String> {
    thread_message::Entity::find()
        .filter(thread_message::Column::ThreadId.eq(id))
        .order_by_asc(thread_message::Column::Idx)
        .all(db)
        .await
        .map_err(|e| err("load thread messages", e))
}

fn decode(rows: &[thread_message::Model]) -> Result<Vec<ChatMessage>, String> {
    rows.iter().map(|r| serde_json::from_str(&r.message_json).map_err(|e| err("parse message", e))).collect()
}

fn message_row(id: &str, idx: usize, m: &ChatMessage, seq: i64) -> Result<thread_message::ActiveModel, String> {
    Ok(thread_message::ActiveModel {
        thread_id: Set(id.into()),
        idx: Set(idx as i64),
        role: Set(m.role.clone()),
        message_json: Set(serde_json::to_string(m).map_err(|e| err("serialize message", e))?),
        seq: Set(seq),
    })
}

/// Replace a thread's whole projection (`last_seq` stays 0, every message `seq` 0).
async fn write_whole(
    db: &DatabaseConnection,
    id: &str,
    mut state: ThreadState,
    messages: Vec<ChatMessage>,
    created_at: &str,
    updated_at: &str,
) -> Result<(), String> {
    state.last_seq = 0;
    let m = &state.meta;
    let row = thread::ActiveModel {
        id: Set(id.into()),
        provider: Set(m.provider.clone()),
        kind: Set(m.kind.clone()),
        title: Set(m.title.clone()),
        workspace_root: Set(m.workspace_root.clone()),
        project_id: Set(m.project_id),
        model: Set(m.model.clone()),
        llm_provider: Set(m.llm_provider.clone()),
        status: Set(state.status.as_str().into()),
        state_json: Set(serde_json::to_string(&state).map_err(|e| err("serialize state", e))?),
        last_seq: Set(0),
        message_count: Set(messages.len() as i64),
        created_at: Set(created_at.into()),
        updated_at: Set(updated_at.into()),
    };
    let rows = messages.iter().enumerate().map(|(i, m)| message_row(id, i, m, 0)).collect::<Result<Vec<_>, _>>()?;
    let txn = db.begin().await.map_err(|e| err("begin", e))?;
    use thread::Column as C;
    thread::Entity::insert(row)
        .on_conflict(
            OnConflict::column(C::Id)
                .update_columns([
                    C::Provider, C::Kind, C::Title, C::WorkspaceRoot, C::ProjectId, C::Model, C::LlmProvider,
                    C::Status, C::StateJson, C::LastSeq, C::MessageCount, C::CreatedAt, C::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_without_returning(&txn)
        .await
        .map_err(|e| err("write thread", e))?;
    replace_messages(&txn, id, rows).await?;
    txn.commit().await.map_err(|e| err("commit", e))
}

async fn replace_messages(txn: &impl ConnectionTrait, id: &str, rows: Vec<thread_message::ActiveModel>) -> Result<(), String> {
    thread_message::Entity::delete_many()
        .filter(thread_message::Column::ThreadId.eq(id))
        .exec(txn)
        .await
        .map_err(|e| err("clear messages", e))?;
    for chunk in rows.chunks(100) {
        thread_message::Entity::insert_many(chunk.to_vec()).exec_without_returning(txn).await.map_err(|e| err("insert messages", e))?;
    }
    Ok(())
}

/// Put `prefix` (seq 0) in front of the thread's rows, renumbering `idx`; false if empty.
/// Events are not rewritten: a `MessagesRewritten` already in the log keeps its old `keep`.
async fn prepend(db: &DatabaseConnection, id: &str, prefix: &[ChatMessage], have: Vec<thread_message::Model>) -> Result<bool, String> {
    if prefix.is_empty() {
        return Ok(false);
    }
    let mut rows = prefix.iter().enumerate().map(|(i, m)| message_row(id, i, m, 0)).collect::<Result<Vec<_>, _>>()?;
    rows.extend(have.iter().map(|r| thread_message::ActiveModel {
        thread_id: Set(id.into()),
        idx: Set(r.idx + prefix.len() as i64),
        role: Set(r.role.clone()),
        message_json: Set(r.message_json.clone()),
        seq: Set(r.seq),
    }));
    let total = rows.len() as i64;
    let txn = db.begin().await.map_err(|e| err("begin", e))?;
    replace_messages(&txn, id, rows).await?;
    let t = thread::ActiveModel { id: Set(id.into()), message_count: Set(total), ..Default::default() };
    t.update(&txn).await.map_err(|e| err("update thread", e))?;
    txn.commit().await.map_err(|e| err("commit", e))?;
    Ok(true)
}

/// Close every turn a dead process left open, as a real event at `last_seq + 1`,
/// through the hub (the single writer). A CLI run (`running` or `waiting`) is gone with
/// its process, so it ends as `Exited` (TurnEnded interrupted, approvals cleared). A
/// Portal `running` run ends the same way; a Portal `waiting` thread stays, because the
/// platform still holds the pending call. Neither command has a `react` effect, so no
/// run starts. Imported rows are never open (`fold_session` closes them).
async fn reconcile(db: &DatabaseConnection, hub: &Arc<ThreadHub>) -> Result<usize, String> {
    let open = thread::Entity::find()
        .filter(thread::Column::Status.is_in(["running", "waiting"]))
        .all(db)
        .await
        .map_err(|e| err("list open threads", e))?;
    let mut n = 0;
    for row in open {
        let cmd = match (row.provider == "portal", row.status.as_str()) {
            (true, "waiting") => continue,
            (true, _) => ThreadCommand::Portal(PortalInput::Finished { outcome: TurnOutcome::Interrupted, final_text: None }),
            (false, _) => ThreadCommand::Provider(AgentEvent::Exited { code: None }),
        };
        match hub.command(&row.id, Envelope { cmd, actor: "system".into(), command_id: None }).await {
            Ok(ack) if matches!(ack.status, AckStatus::Accepted) => n += 1,
            Ok(_) => {}
            Err(e) => log_warn!("Threads", "{e} (reconcile {})", row.id),
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::{ActiveValue::Set, EntityTrait};
    use serde_json::json;

    use super::super::ThreadHub;
    use super::*;
    use crate::database::DatabaseManager;
    use crate::domains::coder::types::PendingApproval;
    use crate::events::EventSink;

    #[derive(Debug)]
    struct NoopSink;
    impl EventSink for NoopSink {
        fn emit_json(&self, _: &str, _: serde_json::Value) -> bool {
            true
        }
    }

    const OLD: &str = "2026-01-01T00:00:00Z";

    fn cm(role: &str, text: &str) -> ChatMessage {
        ChatMessage { role: role.into(), content: Some(text.into()), tool_calls: None, tool_call_id: None, timestamp: None }
    }

    async fn coder(db: &DatabaseConnection, id: &str, msgs: &[ChatMessage]) {
        coder_thread::Entity::insert(coder_thread::ActiveModel {
            id: Set(id.into()),
            title: Set(format!("t {id}")),
            workspace_root: Set("/w".into()),
            project_id: Set(Some(7)),
            model: Set(Some("m".into())),
            platform_thread_id: Set(None),
            llm_provider: Set(Some("ollama".into())),
            messages_json: Set(serde_json::to_string(msgs).unwrap()),
            thread_kind: Set("coder".into()),
            created_at: Set(OLD.into()),
            updated_at: Set(OLD.into()),
        })
        .exec(db)
        .await
        .unwrap();
    }

    async fn session(db: &DatabaseConnection, id: &str, backend: &str) {
        agent_session::Entity::insert(agent_session::ActiveModel {
            id: Set(id.into()),
            backend_id: Set(backend.into()),
            model: Set(None),
            cwd: Set("/w".into()),
            status: Set("exited".into()),
            started_at: Set(OLD.into()),
            ended_at: Set(Some("2026-01-01T01:00:00Z".into())),
            exit_code: Set(None),
            title: Set(None),
            task_id: Set(None),
            backend_session_id: Set(None),
        })
        .exec(db)
        .await
        .unwrap();
    }

    async fn event(db: &DatabaseConnection, sid: &str, seq: i64, at: &str, e: AgentEvent) {
        agent_event::Entity::insert(agent_event::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            session_id: Set(sid.into()),
            seq: Set(seq),
            event_json: Set(serde_json::to_string(&e).unwrap()),
            created_at: Set(at.into()),
        })
        .exec(db)
        .await
        .unwrap();
    }

    async fn message(db: &DatabaseConnection, sid: &str, seq: i64, role: &str, text: &str, at: &str) {
        agent_message::Entity::insert(agent_message::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            session_id: Set(sid.into()),
            seq: Set(seq),
            role: Set(role.into()),
            text: Set(text.into()),
            parent_tool_use_id: Set(None),
            created_at: Set(at.into()),
        })
        .exec(db)
        .await
        .unwrap();
    }

    fn msg(role: &str, text: &str) -> AgentEvent {
        AgentEvent::Message { role: role.into(), text: text.into(), parent_tool_use_id: None }
    }

    /// Every thread-log row, as text, to compare two runs.
    async fn dump(db: &DatabaseConnection) -> String {
        format!(
            "{:?}{:?}{:?}",
            thread::Entity::find().order_by_asc(thread::Column::Id).all(db).await.unwrap(),
            thread_message::Entity::find()
                .order_by_asc(thread_message::Column::ThreadId)
                .order_by_asc(thread_message::Column::Idx)
                .all(db)
                .await
                .unwrap(),
            super::super::entities::thread_event::Entity::find().all(db).await.unwrap(),
        )
    }

    async fn texts(db: &DatabaseConnection, id: &str) -> Vec<(String, String, i64)> {
        load_rows(db, id)
            .await
            .unwrap()
            .into_iter()
            .map(|r| {
                let m: ChatMessage = serde_json::from_str(&r.message_json).unwrap();
                (m.role, m.content.unwrap_or_default(), r.seq)
            })
            .collect()
    }

    fn portal_meta() -> ThreadMeta {
        ThreadMeta { provider: "portal".into(), kind: "coder".into(), title: "p".into(), workspace_root: "/w".into(), ..Default::default() }
    }

    #[tokio::test]
    async fn imports_legacy_idempotently_and_reconciles() {
        let dir = tempfile::tempdir().unwrap();
        let dbm = Arc::new(DatabaseManager::new(dir.path().to_path_buf()).await.unwrap());
        let db = dbm.get_connection();
        let full = [cm("user", "hi"), cm("assistant", "hello"), cm("user", "q2"), cm("assistant", "a2")];

        // Legacy rows.
        coder(db, "c1", &full).await;
        coder(db, "c-empty", &[]).await;
        session(db, "s1", "claude-code").await;
        message(db, "s1", 0, "user", "build it", OLD).await;
        message(db, "s1", 2, "assistant", "hello", OLD).await; // same seq as the event below
        event(db, "s1", 1, OLD, AgentEvent::SessionStarted { session_id: "b".into() }).await;
        event(db, "s1", 2, OLD, msg("assistant", "hello")).await;
        event(db, "s1", 3, OLD, AgentEvent::ToolCall { id: "t1".into(), name: "Bash".into(), input: json!({"command": "ls"}), parent_tool_use_id: None }).await;
        event(db, "s1", 4, OLD, AgentEvent::ToolResult { tool_use_id: "t1".into(), content: "ok".into(), is_error: false }).await;
        event(db, "s1", 5, OLD, AgentEvent::Message { role: "assistant".into(), text: "sub".into(), parent_tool_use_id: Some("p".into()) }).await;
        event(db, "s1", 6, OLD, AgentEvent::TurnDone { final_text: Some("done".into()), is_error: false }).await;
        session(db, "s2", "codex").await; // transcript: message rows only
        message(db, "s2", 0, "user", "q", OLD).await;
        message(db, "s2", 1, "assistant", "a", OLD).await;
        session(db, "s3", "cursor").await; // log ends on an open approval
        event(db, "s3", 1, OLD, msg("assistant", "x")).await;
        event(db, "s3", 2, OLD, AgentEvent::PermissionRequest { request_id: "r".into(), tool_name: "Bash".into(), tool_input: json!({}), suggestions: vec![] }).await;
        session(db, "s4", "claude-code").await; // resumed after the thread log began
        message(db, "s4", 0, "user", "old", OLD).await;
        event(db, "s4", 1, "2026-01-01T00:00:01Z", msg("assistant", "old reply")).await;
        for id in ["k-run", "k-wait"] {
            session(db, id, "claude-code").await;
        }

        // Threads the hub created itself.
        let hub = ThreadHub::new(dbm.clone(), Arc::new(NoopSink));
        for id in ["p-run", "p-wait"] {
            hub.submit(id, ThreadCommand::Create(portal_meta()));
        }
        for id in ["p-run", "p-wait"] {
            hub.submit(id, ThreadCommand::Send { text: "go".into() });
        }
        let approval = PendingApproval { call_id: "a1".into(), tool: "Bash".into(), arguments: json!({}), suggested_rule: String::new(), summary: "ls".into() };
        hub.submit("p-wait", ThreadCommand::Portal(PortalInput::Pending(approval)));
        hub.submit("k-run", ThreadCommand::Provider(AgentEvent::TextDelta { text: "t".into(), parent_tool_use_id: None }));
        hub.submit("k-wait", ThreadCommand::Provider(AgentEvent::PermissionRequest { request_id: "r".into(), tool_name: "Bash".into(), tool_input: json!({}), suggestions: vec![] }));
        hub.submit("s4", ThreadCommand::Provider(msg("assistant", "new")));
        for id in ["p-run", "p-wait", "k-run", "k-wait", "s4"] {
            hub.snapshot(id).await.unwrap().unwrap(); // the mailbox is ordered: the commands are committed
        }
        let status = |id: &'static str| async move { store::summary(db, id).await.unwrap().unwrap().status };
        let seq = |id: &'static str| async move { store::summary(db, id).await.unwrap().unwrap().last_seq };
        assert_eq!(
            (status("p-run").await, status("p-wait").await, status("k-run").await, status("k-wait").await),
            ("running".into(), "waiting".into(), "running".into(), "waiting".into())
        );
        let (p_run_seq, p_wait_seq) = (seq("p-run").await, seq("p-wait").await);

        // First run.
        let startup = ThreadHub::new(dbm.clone(), Arc::new(NoopSink)); // a restart: no live actors
        let r = run(db, &startup).await.unwrap();
        // p-run, k-run, k-wait, and s4 (its "new" message opened a turn) are reconciled.
        assert_eq!(r, Report { imported: 5, prepended: 1, reconciled: 4 });

        let t = |role: &str, text: &str, seq: i64| (role.to_string(), text.to_string(), seq);
        assert_eq!(texts(db, "c1").await.len(), 4);
        assert_eq!(texts(db, "c-empty").await.len(), 0);
        let s1 = texts(db, "s1").await;
        assert_eq!(
            s1.iter().map(|m| (m.0.as_str(), m.1.as_str())).collect::<Vec<_>>(),
            [("user", "build it"), ("assistant", "hello"), ("assistant", ""), ("tool", "ok")]
        );
        assert_eq!(texts(db, "s2").await, [t("user", "q", 0), t("assistant", "a", 0)]);
        assert_eq!(texts(db, "s3").await, [t("assistant", "x", 0)]);
        let s4 = texts(db, "s4").await;
        assert_eq!(s4.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(), ["old", "old reply", "new"]);
        assert_eq!((s4[0].2, s4[1].2, s4[2].2 > 0), (0, 0, true));

        // Imported rows: projection only, idle, nothing pending.
        let s1_row = store::summary(db, "s1").await.unwrap().unwrap();
        assert_eq!((s1_row.last_seq, s1_row.message_count, s1_row.status.as_str(), s1_row.provider.as_str()), (0, 4, "idle", "claude-code"));
        let s3 = store::load(db, "s3").await.unwrap().unwrap();
        assert!(s3.approvals.is_empty() && !s3.turn_open());
        assert_eq!(store::load(db, "s1").await.unwrap().unwrap().backend_session_id.as_deref(), Some("b"));
        let c1 = store::summary(db, "c1").await.unwrap().unwrap();
        assert_eq!((c1.provider.as_str(), c1.project_id, c1.last_seq, c1.message_count), ("portal", Some(7), 0, 4));

        // Reconcile: open turns end as real events; a Portal `waiting` thread is kept.
        assert_eq!((status("p-run").await, status("k-run").await, status("k-wait").await), ("idle".into(), "idle".into(), "idle".into()));
        assert_eq!(seq("p-run").await, p_run_seq + 1);
        assert_eq!((status("p-wait").await, seq("p-wait").await), ("waiting".into(), p_wait_seq));
        assert_eq!(store::load(db, "p-wait").await.unwrap().unwrap().approvals.len(), 1);
        assert!(store::load(db, "k-wait").await.unwrap().unwrap().approvals.is_empty());
        let tail = hub.events_since("p-run", p_run_seq).await.unwrap().events;
        assert!(matches!(
            tail[..],
            [ThreadEventFrame { event: ThreadEvent::TurnEnded { outcome: TurnOutcome::Interrupted, .. }, .. }]
        ));

        // Second run changes nothing.
        let before = dump(db).await;
        assert_eq!(run(db, &startup).await.unwrap(), Report::default());
        assert_eq!(dump(db).await, before);

        // A fresh hub serves an imported thread: snapshot, no events, no reset.
        let hub = ThreadHub::new(dbm.clone(), Arc::new(NoopSink));
        let snap = hub.snapshot("s1").await.unwrap().unwrap();
        assert_eq!((snap.seq, snap.messages.len(), snap.thread.message_count), (0, 4, 4));
        let page = hub.events_since("s1", 0).await.unwrap();
        assert!(page.events.is_empty() && !page.reset && page.head_seq == 0);
    }

    /// The agents service persists the prompt before the hub sees it, so the prompt is in
    /// both places: start order must not duplicate it, resume order keeps the old history.
    #[tokio::test]
    async fn hub_created_threads_keep_one_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let dbm = Arc::new(DatabaseManager::new(dir.path().to_path_buf()).await.unwrap());
        let db = dbm.get_connection();
        let done = || ThreadCommand::Provider(AgentEvent::TurnDone { final_text: None, is_error: false });

        // start_run: prompt row (seq 0) and an event row, then the hub's own copy of the prompt.
        session(db, "start", "claude-code").await;
        event(db, "start", 1, OLD, AgentEvent::SessionStarted { session_id: "b".into() }).await;
        message(db, "start", 0, "user", "go", OLD).await;
        // resume_session: old history, then the resume prompt, then the hub's copy of it.
        session(db, "resume", "claude-code").await;
        message(db, "resume", 0, "user", "old q", OLD).await;
        event(db, "resume", 1, OLD, msg("assistant", "old a")).await;
        message(db, "resume", 2, "user", "again", OLD).await;

        // Repeated text: old turn "go"/"ok", resume prompt "go"; the hub then sees go, ok, go.
        // The overlap is the one prompt, not the longest run that happens to match.
        session(db, "repeat", "claude-code").await;
        message(db, "repeat", 0, "user", "go", OLD).await;
        event(db, "repeat", 1, OLD, msg("assistant", "ok")).await;
        message(db, "repeat", 2, "user", "go", OLD).await;

        let hub = ThreadHub::new(dbm.clone(), Arc::new(NoopSink));
        for (id, turns) in [
            ("start", vec![("go", "reply")]),
            ("resume", vec![("again", "ok")]),
            ("repeat", vec![("go", "ok"), ("go", "fin")]),
        ] {
            for (prompt, reply) in turns {
                hub.submit(id, ThreadCommand::Provider(msg("user", prompt)));
                hub.submit(id, ThreadCommand::Provider(msg("assistant", reply)));
                hub.submit(id, done());
            }
            hub.snapshot(id).await.unwrap().unwrap();
        }

        let startup = ThreadHub::new(dbm.clone(), Arc::new(NoopSink));
        assert_eq!(run(db, &startup).await.unwrap(), Report { imported: 0, prepended: 2, reconciled: 0 });
        let repeat = texts(db, "repeat").await;
        assert_eq!(repeat.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(), ["go", "ok", "go", "ok", "go", "fin"]);
        assert_eq!(repeat.iter().map(|m| m.2 == 0).collect::<Vec<_>>(), [true, true, false, false, false, false]);
        let start = texts(db, "start").await;
        assert_eq!(start.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(), ["go", "reply"]);
        let resume = texts(db, "resume").await;
        assert_eq!(resume.iter().map(|m| m.1.as_str()).collect::<Vec<_>>(), ["old q", "old a", "again", "ok"]);
        assert_eq!(resume.iter().map(|m| m.2 == 0).collect::<Vec<_>>(), [true, true, false, false]);

        let before = dump(db).await;
        assert_eq!(run(db, &startup).await.unwrap(), Report::default());
        assert_eq!(dump(db).await, before);
    }
}
