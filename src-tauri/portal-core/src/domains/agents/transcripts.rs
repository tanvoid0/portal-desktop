//! Imports Claude Code sessions that Portal did not start (terminal, desktop
//! app, other tools) from `~/.claude/projects/<encoded-cwd>/<sessionId>.jsonl`
//! into `agent_sessions` / `agent_messages`, so they show up in the Agents
//! list and can be resumed through the normal `--resume` path.
//!
//! Imported rows use `id == backend_session_id == <sessionId>`; that equality
//! is the "imported" marker. A session Portal launched itself has a uuid id
//! and the same `backend_session_id`, so it is skipped (no duplicates).
//! Re-imported only when the file's mtime is newer than the stored
//! `ended_at`. Subagent transcripts and tool events are not imported.
// ponytail: poll-only (no fs watcher), messages only (no tool events). Add a
// debounced watcher / event rows if 30s latency or tool detail matters.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value;

use crate::database::DatabaseManager;

use super::entities::{agent_message, agent_session};

const BACKEND_ID: &str = "claude-code";
const INSERT_CHUNK: usize = 100;

#[derive(Debug, PartialEq)]
pub struct ParsedTurn {
    pub role: String,
    pub text: String,
    pub at: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ParsedSession {
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub turns: Vec<ParsedTurn>,
}

/// Pure: reads jsonl lines, keeps main-thread user/assistant text turns.
pub fn parse_transcript(reader: impl BufRead) -> ParsedSession {
    let mut out = ParsedSession::default();
    for line in reader.lines().map_while(Result::ok) {
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if out.cwd.is_none() {
            out.cwd = v.get("cwd").and_then(Value::as_str).map(str::to_string);
        }
        let role = match v.get("type").and_then(Value::as_str) {
            Some(r @ ("user" | "assistant")) => r,
            _ => continue,
        };
        if v.get("isSidechain").and_then(Value::as_bool) == Some(true)
            || v.get("isMeta").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        let msg = v.get("message");
        if role == "assistant" && out.model.is_none() {
            out.model = msg
                .and_then(|m| m.get("model"))
                .and_then(Value::as_str)
                .filter(|m| !m.starts_with('<'))
                .map(str::to_string);
        }
        let text = match msg.and_then(|m| m.get("content")) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(blocks)) => blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => continue,
        };
        if text.trim().is_empty() {
            continue;
        }
        out.turns.push(ParsedTurn {
            role: role.to_string(),
            text,
            at: v.get("timestamp").and_then(Value::as_str).map(str::to_string),
        });
    }
    out
}

fn title_of(turns: &[ParsedTurn]) -> Option<String> {
    let first = turns.iter().find(|t| t.role == "user")?;
    let line = first.text.lines().find(|l| !l.trim().is_empty())?.trim();
    Some(if line.chars().count() > 80 {
        let cut: String = line.chars().take(79).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    })
}

fn claude_projects_dir() -> Option<PathBuf> {
    // CLAUDE_CONFIG_DIR is Claude Code's own override for ~/.claude.
    let home = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude")))?;
    Some(home.join("projects"))
}

/// `<projects>/<proj>/<sid>.jsonl` files, with their mtimes.
fn scan(root: &Path) -> Vec<(String, PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let Ok(projs) = std::fs::read_dir(root) else {
        return out;
    };
    for proj in projs.flatten() {
        let Ok(files) = std::fs::read_dir(proj.path()) else {
            continue;
        };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(sid) = p.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
                continue;
            };
            let Ok(mtime) = f.metadata().and_then(|m| m.modified()) else {
                continue;
            };
            out.push((sid, p, mtime));
        }
    }
    out
}

/// One sweep. Returns the ids of the sessions that were (re)imported, for the caller
/// to refresh their threads.
pub async fn sync_claude_transcripts(db: &DatabaseManager) -> Result<Vec<String>, String> {
    let Some(root) = claude_projects_dir() else {
        return Ok(Vec::new());
    };
    sync_from(db, &root).await
}

async fn sync_from(db: &DatabaseManager, root: &Path) -> Result<Vec<String>, String> {
    let conn = db.get_connection();
    let root = root.to_path_buf();
    let mut files = tokio::task::spawn_blocking(move || scan(&root))
        .await
        .map_err(|e| e.to_string())?;
    // A resumed/forked session can leave the same sessionId file in more than
    // one project dir; keep the newest so the id inserts once per sweep.
    files.sort_by(|a, b| a.0.cmp(&b.0).then(b.2.cmp(&a.2)));
    files.dedup_by(|b, a| a.0 == b.0);

    // sessionId -> (row id, ended_at, status) for every claude-code row we know.
    let known: HashMap<String, (String, Option<String>, String)> = agent_session::Entity::find()
        .filter(agent_session::Column::BackendId.eq(BACKEND_ID))
        .all(conn)
        .await
        .map_err(|e| format!("agents: load sessions for import failed: {e}"))?
        .into_iter()
        .filter_map(|r| r.backend_session_id.map(|b| (b, (r.id, r.ended_at, r.status))))
        .collect();

    let mut imported = Vec::new();
    for (sid, path, mtime) in files {
        let mtime: DateTime<Utc> = mtime.into();
        match known.get(&sid) {
            // Portal-launched run: its own persistence is authoritative.
            Some((id, ..)) if *id != sid => continue,
            // Portal is resuming it (its own persistence is authoritative; a replace
            // here would wipe the run's rows, #1976).
            Some((_, _, status)) if status == "running" => continue,
            // Imported earlier and file hasn't moved since.
            Some((_, Some(ended), _)) if DateTime::parse_from_rfc3339(ended)
                .map(|e| mtime <= e.with_timezone(&Utc))
                .unwrap_or(false) =>
            {
                continue
            }
            _ => {}
        }
        let parsed = tokio::task::spawn_blocking(move || {
            std::fs::File::open(&path).map(|f| parse_transcript(BufReader::new(f)))
        })
        .await
        .map_err(|e| e.to_string())?;
        let Ok(parsed) = parsed else { continue };
        let Some(cwd) = parsed.cwd.clone() else { continue };
        if parsed.turns.is_empty() {
            continue;
        }
        if let Err(e) = store(db, &sid, &cwd, &parsed, mtime, known.contains_key(&sid)).await {
            eprintln!("agents: import {sid} failed: {e}");
            continue;
        }
        imported.push(sid);
    }
    Ok(imported)
}

async fn store(
    db: &DatabaseManager,
    sid: &str,
    cwd: &str,
    p: &ParsedSession,
    mtime: DateTime<Utc>,
    replace: bool,
) -> Result<(), String> {
    let conn = db.get_connection();
    let started = p
        .turns
        .iter()
        .find_map(|t| t.at.clone())
        .unwrap_or_else(|| mtime.to_rfc3339());
    let ended = mtime.to_rfc3339();

    if replace {
        agent_message::Entity::delete_many()
            .filter(agent_message::Column::SessionId.eq(sid))
            .exec(conn)
            .await
            .map_err(|e| e.to_string())?;
        agent_session::Entity::delete_by_id(sid.to_string())
            .exec(conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    agent_session::Entity::insert(agent_session::ActiveModel {
        id: Set(sid.to_string()),
        backend_id: Set(BACKEND_ID.to_string()),
        model: Set(p.model.clone()),
        cwd: Set(cwd.to_string()),
        status: Set("exited".to_string()),
        started_at: Set(started.clone()),
        ended_at: Set(Some(ended)),
        exit_code: Set(None),
        title: Set(title_of(&p.turns)),
        task_id: Set(None),
        backend_session_id: Set(Some(sid.to_string())),
    })
    .exec(conn)
    .await
    .map_err(|e| e.to_string())?;

    for (chunk_i, chunk) in p.turns.chunks(INSERT_CHUNK).enumerate() {
        let rows = chunk.iter().enumerate().map(|(i, t)| agent_message::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            session_id: Set(sid.to_string()),
            seq: Set((chunk_i * INSERT_CHUNK + i) as i64),
            role: Set(t.role.clone()),
            text: Set(t.text.clone()),
            parent_tool_use_id: Set(None),
            created_at: Set(t.at.clone().unwrap_or_else(|| started.clone())),
        });
        agent_message::Entity::insert_many(rows)
            .exec(conn)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::ActiveModelTrait;

    #[tokio::test]
    async fn sweep_imports_once_dedups_across_dirs_and_reimports_on_growth() {
        let data = tempfile::tempdir().unwrap();
        let db = DatabaseManager::new(data.path().to_path_buf()).await.unwrap();
        let root = tempfile::tempdir().unwrap();
        let line = r#"{"type":"user","cwd":"D:\\p","timestamp":"2026-11-11T00:00:00Z","message":{"content":"hi"}}"#;
        for proj in ["a", "b"] {
            std::fs::create_dir(root.path().join(proj)).unwrap();
            std::fs::write(root.path().join(proj).join("sid1.jsonl"), line).unwrap();
        }
        assert_eq!(sync_from(&db, root.path()).await.unwrap(), ["sid1"]);
        assert!(sync_from(&db, root.path()).await.unwrap().is_empty()); // unchanged
        let rows = agent_session::Entity::find().all(db.get_connection()).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].backend_session_id.as_deref(), Some("sid1"));

        std::thread::sleep(std::time::Duration::from_millis(50));
        let grown = format!("{line}\n{}", r#"{"type":"assistant","message":{"content":[{"type":"text","text":"yo"}]}}"#);
        std::fs::write(root.path().join("a").join("sid1.jsonl"), grown).unwrap();
        assert_eq!(sync_from(&db, root.path()).await.unwrap(), ["sid1"]);
        let msgs = agent_message::Entity::find().all(db.get_connection()).await.unwrap();
        assert_eq!(msgs.len(), 2);

        // Portal is running it: the sync leaves it alone even though the file moved.
        let mut am: agent_session::ActiveModel = rows[0].clone().into();
        am.status = Set("running".into());
        am.ended_at = Set(None);
        am.update(db.get_connection()).await.unwrap();
        std::fs::write(root.path().join("a").join("sid1.jsonl"), line).unwrap();
        assert!(sync_from(&db, root.path()).await.unwrap().is_empty());
        assert_eq!(agent_message::Entity::find().all(db.get_connection()).await.unwrap().len(), 2);
    }

    #[test]
    fn parses_main_thread_text_turns_only() {
        let jsonl = [
            r#"{"type":"session-owner","sessionId":"s"}"#,
            r#"{"type":"user","cwd":"D:\\p","timestamp":"2026-11-11T00:00:00Z","message":{"role":"user","content":"hello\nworld"}}"#,
            r#"{"type":"assistant","message":{"model":"claude-x","content":[{"type":"thinking","thinking":"x"},{"type":"text","text":"hi"},{"type":"tool_use","id":"t","name":"Bash","input":{}}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t","content":"out"}]}}"#,
            r#"{"type":"user","isSidechain":true,"message":{"content":"sub"}}"#,
            r#"{"type":"user","isMeta":true,"message":{"content":"meta"}}"#,
            "not json",
        ]
        .join("\n");
        let p = parse_transcript(jsonl.as_bytes());
        assert_eq!(p.cwd.as_deref(), Some("D:\\p"));
        assert_eq!(p.model.as_deref(), Some("claude-x"));
        assert_eq!(p.turns.len(), 2);
        assert_eq!(p.turns[0].text, "hello\nworld");
        assert_eq!(p.turns[1].role, "assistant");
        assert_eq!(title_of(&p.turns).as_deref(), Some("hello"));
    }
}
