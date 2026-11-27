//! SQL for the thread log. The hub's only door to the database; the one
//! transaction in `commit` holds nothing but SQL.

use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};

use super::entities::{thread, thread_event, thread_message};
use super::types::*;
use crate::domains::agents::entities::agent_session;
use crate::domains::coder::types::ChatMessage;

/// A client further behind than this re-snapshots instead of paging events.
pub const MAX_CATCH_UP: i64 = 2000;

fn err(what: &str, e: impl std::fmt::Display) -> String {
    format!("threads: {what} failed: {e}")
}

impl From<thread::Model> for ThreadSummary {
    fn from(m: thread::Model) -> Self {
        Self {
            id: m.id,
            provider: m.provider,
            kind: m.kind,
            title: m.title,
            workspace_root: m.workspace_root,
            project_id: m.project_id,
            model: m.model,
            llm_provider: m.llm_provider,
            status: m.status,
            last_seq: m.last_seq,
            message_count: m.message_count,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Rebuild a thread from its row and message rows (never by replaying events).
pub async fn load(db: &DatabaseConnection, id: &str) -> Result<Option<ThreadState>, String> {
    let Some(row) = thread::Entity::find_by_id(id.to_string()).one(db).await.map_err(|e| err("load thread", e))? else {
        return Ok(None);
    };
    let mut state: ThreadState = serde_json::from_str(&row.state_json).map_err(|e| err("parse state", e))?;
    let rows = thread_message::Entity::find()
        .filter(thread_message::Column::ThreadId.eq(id))
        .order_by_asc(thread_message::Column::Idx)
        .all(db)
        .await
        .map_err(|e| err("load messages", e))?;
    state.messages = rows
        .iter()
        .map(|r| serde_json::from_str(&r.message_json).map_err(|e| err("parse message", e)))
        .collect::<Result<_, _>>()?;
    state.last_seq = row.last_seq;
    Ok(Some(state))
}

enum MsgOp {
    Insert(thread_message::ActiveModel),
    Truncate(i64),
}

/// The last seq a client `command_id` produced, if it was ever accepted (indexed lookup).
pub async fn command_seq(db: &DatabaseConnection, id: &str, command_id: &str) -> Result<Option<i64>, String> {
    let row = thread_event::Entity::find()
        .filter(thread_event::Column::ThreadId.eq(id))
        .filter(thread_event::Column::CommandId.eq(command_id))
        .order_by_desc(thread_event::Column::Seq)
        .one(db)
        .await
        .map_err(|e| err("command lookup", e))?;
    Ok(row.map(|r| r.seq))
}

/// Insert `frames`, upsert the `threads` row (with `next` as its state) and append
/// the new messages, atomically. `prev_messages` = how many message rows exist.
pub async fn commit(
    db: &DatabaseConnection,
    id: &str,
    prev_messages: usize,
    next: &ThreadState,
    frames: &[ThreadEventFrame],
) -> Result<(), String> {
    let at = frames.last().map(|f| f.at.clone()).unwrap_or_default();
    let m = &next.meta;
    let row = thread::ActiveModel {
        id: Set(id.to_string()),
        provider: Set(m.provider.clone()),
        kind: Set(m.kind.clone()),
        title: Set(m.title.clone()),
        workspace_root: Set(m.workspace_root.clone()),
        project_id: Set(m.project_id),
        model: Set(m.model.clone()),
        llm_provider: Set(m.llm_provider.clone()),
        status: Set(next.status.as_str().to_string()),
        state_json: Set(serde_json::to_string(next).map_err(|e| err("serialize state", e))?),
        last_seq: Set(next.last_seq),
        message_count: Set(next.messages.len() as i64),
        created_at: Set(at.clone()),
        updated_at: Set(at),
    };
    let mut events = vec![];
    // Message-row ops in event order: a rewrite truncates, then its tail inserts.
    let mut messages: Vec<MsgOp> = vec![];
    let mut len = prev_messages;
    let msg_row = |len: &mut usize, message: &ChatMessage, seq: i64| -> Result<MsgOp, String> {
        let m = thread_message::ActiveModel {
            thread_id: Set(id.to_string()),
            idx: Set(*len as i64),
            role: Set(message.role.clone()),
            message_json: Set(serde_json::to_string(message).map_err(|e| err("serialize message", e))?),
            seq: Set(seq),
        };
        *len += 1;
        Ok(MsgOp::Insert(m))
    };
    for f in frames {
        let payload = serde_json::to_value(&f.event).map_err(|e| err("serialize event", e))?;
        events.push(thread_event::ActiveModel {
            thread_id: Set(id.to_string()),
            seq: Set(f.seq),
            kind: Set(payload["type"].as_str().unwrap_or_default().to_string()),
            payload: Set(payload.to_string()),
            command_id: Set(f.command_id.clone()),
            actor: Set(f.actor.clone()),
            at: Set(f.at.clone()),
        });
        match &f.event {
            ThreadEvent::MessageAppended { message } => messages.push(msg_row(&mut len, message, f.seq)?),
            ThreadEvent::MessagesRewritten { keep, tail } => {
                len = len.min(*keep as usize);
                messages.push(MsgOp::Truncate(len as i64));
                for m in tail {
                    messages.push(msg_row(&mut len, m, f.seq)?);
                }
            }
            _ => {}
        }
    }

    let txn = db.begin().await.map_err(|e| err("begin", e))?;
    use thread::Column as C;
    thread::Entity::insert(row)
        .on_conflict(
            OnConflict::column(C::Id)
                .update_columns([
                    C::Provider, C::Kind, C::Title, C::WorkspaceRoot, C::ProjectId, C::Model, C::LlmProvider,
                    C::Status, C::StateJson, C::LastSeq, C::MessageCount, C::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_without_returning(&txn)
        .await
        .map_err(|e| err("write thread", e))?;
    for e in events {
        thread_event::Entity::insert(e).exec_without_returning(&txn).await.map_err(|e| err("insert event", e))?;
    }
    for op in messages {
        match op {
            MsgOp::Insert(m) => {
                thread_message::Entity::insert(m).exec_without_returning(&txn).await.map_err(|e| err("insert message", e))?;
            }
            MsgOp::Truncate(from) => {
                thread_message::Entity::delete_many()
                    .filter(thread_message::Column::ThreadId.eq(id))
                    .filter(thread_message::Column::Idx.gte(from))
                    .exec(&txn)
                    .await
                    .map_err(|e| err("truncate messages", e))?;
            }
        }
    }
    txn.commit().await.map_err(|e| err("commit", e))
}

pub async fn summary(db: &DatabaseConnection, id: &str) -> Result<Option<ThreadSummary>, String> {
    let row = thread::Entity::find_by_id(id.to_string()).one(db).await.map_err(|e| err("load thread", e))?;
    Ok(row.map(Into::into))
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<ThreadSummary>, String> {
    let rows = thread::Entity::find()
        .order_by_desc(thread::Column::UpdatedAt)
        .all(db)
        .await
        .map_err(|e| err("list", e))?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn events_since(db: &DatabaseConnection, id: &str, since: i64) -> Result<ThreadEventsPage, String> {
    let row = thread::Entity::find_by_id(id.to_string()).one(db).await.map_err(|e| err("load thread", e))?;
    let head = row.as_ref().map_or(0, |r| r.last_seq);
    let provider = row.map(|r| r.provider).unwrap_or_default();
    if since > head || head - since > MAX_CATCH_UP {
        return Ok(ThreadEventsPage { events: vec![], head_seq: head, reset: true });
    }
    let rows = thread_event::Entity::find()
        .filter(thread_event::Column::ThreadId.eq(id))
        .filter(thread_event::Column::Seq.gt(since))
        .order_by_asc(thread_event::Column::Seq)
        .all(db)
        .await
        .map_err(|e| err("read events", e))?;
    let events = rows
        .into_iter()
        .map(|r| {
            Ok(ThreadEventFrame {
                thread_id: r.thread_id,
                provider: provider.clone(),
                seq: r.seq,
                at: r.at,
                actor: r.actor,
                command_id: r.command_id,
                event: serde_json::from_str(&r.payload).map_err(|e| err("parse event", e))?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(ThreadEventsPage { events, head_seq: head, reset: false })
}

/// The `Create` meta for a run the agents service already persisted.
pub async fn session_meta(db: &DatabaseConnection, id: &str) -> Result<Option<ThreadMeta>, String> {
    let row = agent_session::Entity::find_by_id(id.to_string()).one(db).await.map_err(|e| err("load session", e))?;
    Ok(row.map(|s| ThreadMeta {
        title: s.title.unwrap_or_else(|| format!("{} session", s.backend_id)),
        provider: s.backend_id,
        model: s.model,
        llm_provider: None,
        workspace_root: s.cwd,
        project_id: None,
        kind: "session".into(),
    }))
}

pub async fn exists(db: &DatabaseConnection, id: &str) -> Result<bool, String> {
    let n = thread::Entity::find_by_id(id.to_string()).count(db).await.map_err(|e| err("count", e))?;
    Ok(n > 0)
}

/// Cascades to the thread's events and messages.
pub async fn delete(db: &DatabaseConnection, id: &str) -> Result<(), String> {
    thread::Entity::delete_by_id(id.to_string()).exec(db).await.map_err(|e| err("delete", e))?;
    Ok(())
}
