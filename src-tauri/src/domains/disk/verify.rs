//! Multi-agent verification of cleanup proposals via the Agent Platform.
//!
//! Mirrors what the agentic-ai platform does for any goal: start a process against
//! a planner team template, let the planner DAG + agents run, then read the task
//! outputs. Here the "goal" is a review of deletion candidates, so a roster of
//! agents cross-checks the heuristic proposals before a human ticks anything.
//!
//! Non-negotiable: this is advisory only. It never deletes and never flips a
//! proposal to a lower risk on its own — the agents' verdict is surfaced to the
//! user, who still confirms every item. See `classify.rs` for the source proposals.
//!
//! MIGRATION NOTE: this still uses a bespoke `reqwest::blocking` client against
//! the standalone Agent Platform. A follow-up should rewire it onto the desktop's
//! async `ai` domain (Ollama/Anthropic/OpenAI/Gemini providers) and drop the
//! blocking client. See `docs/development/DISK_UTILITY_MIGRATION.md`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domains::disk::classify::Proposal;

const DEFAULT_BASE: &str = "http://127.0.0.1:18410";
const CLIENT_ID: &str = "portal-disk-utility";
/// Human-facing name of the dedicated planner roster this app provisions.
const TEAM_NAME: &str = "Deletion Verifier";
/// Stable, app-owned fingerprint embedded in the team's description. This — NOT the
/// platform's auto-increment id — is how we recognize our own team, so it survives a
/// platform DB wipe or a different project: we look it up by marker and re-create if
/// absent. Bump the trailing version if the roster shape changes.
const TEAM_MARKER: &str = "managed-by:portal-disk-utility/deletion-verifier@1";
const POLL_INTERVAL: Duration = Duration::from_millis(1500);
const MAX_WAIT: Duration = Duration::from_secs(300);

/// Connection settings for the embedded Agent Platform. Supplied by the frontend
/// (persisted there / in the OS keychain) so no secret is hardcoded.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    /// Base URL, e.g. `http://127.0.0.1:18410`. Falls back to the local default.
    pub base_url: Option<String>,
    /// Bearer credential. Prefer a project-scoped platform token (`agp_…`) minted
    /// from the dashboard for this app — revocable, rate-limited, per-project — NOT
    /// the master key (that's the unrestricted admin/UI secret). Either is accepted
    /// on the wire; both go in `Authorization: Bearer`.
    pub api_token: Option<String>,
    /// Planner roster to verify with. If absent, this app's own team (found by
    /// [`TEAM_MARKER`], created if missing) is used.
    pub team_template_id: Option<i64>,
}

/// One team template as shown in the Settings picker.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamOption {
    pub id: i64,
    pub name: String,
    pub description: String,
    /// True when this is the app's own provisioned team (carries [`TEAM_MARKER`]).
    pub is_app_team: bool,
}

/// One agent task's contribution to the verdict.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentNote {
    pub task_id: i64,
    pub role: String,
    pub status: String,
    pub output: String,
}

/// A structured per-item verdict parsed from the Lead Synthesizer's output, so
/// the UI can badge each proposal inline instead of showing prose only.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemVerdict {
    pub path: String,
    /// One of "safe" | "review" | "dangerous". Anything else is normalized to "review".
    pub verdict: String,
    pub reason: String,
}

/// One agent task's live state, forwarded to the UI while the process runs so
/// the user sees which reviewers are working rather than a frozen spinner.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifyTask {
    pub role: String,
    pub status: String,
}

/// Streamed progress snapshot emitted on every poll of the running process.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VerifyProgress {
    pub process_id: i64,
    pub status: String,
    pub tasks: Vec<VerifyTask>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub process_id: i64,
    pub status: String,
    pub notes: Vec<AgentNote>,
    /// Per-item verdicts parsed from the synthesizer. Empty if none could be parsed.
    pub verdicts: Vec<ItemVerdict>,
    /// True when the run reached a human-gate (approval/review) rather than finishing.
    pub gated: bool,
}

fn base_url(cfg: &AiConfig) -> String {
    cfg.base_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_BASE)
        .trim_end_matches('/')
        .to_string()
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())
}

fn auth(req: reqwest::blocking::RequestBuilder, cfg: &AiConfig) -> reqwest::blocking::RequestBuilder {
    let req = req.header("X-Agent-Platform-Client", CLIENT_ID);
    match cfg.api_token.as_deref().map(str::trim) {
        Some(k) if !k.is_empty() => req.bearer_auth(k),
        _ => req,
    }
}

/// The roster we provision. Advisory reviewers that reason from metadata only; the
/// lead merges to the most cautious verdict. `parent_id` links synthesis under the
/// risk assessor. The marker lives in the description so we can find this team later.
fn team_body() -> Value {
    json!({
        "name": TEAM_NAME,
        "description": format!(
            "Advisory roster for Portal Disk Utility. Cross-checks disk-cleanup \
             deletion candidates from metadata only; never deletes, never lowers risk. \
             Verdict per item: safe / review / dangerous.\n\n[{TEAM_MARKER}]"
        ),
        "color": "#DC2626",
        "category": "Utility",
        "roster": { "roles": [
            {
                "id": "risk-assessor",
                "name": "Risk Assessor",
                "description": "Judge each candidate (path, kind, size, file count, heuristic reason) \
                    as safe / review / dangerous for Recycle Bin deletion. Reason ONLY from the \
                    metadata embedded in the task; never invent files or assume contents.",
                "modality": "text",
                "accent_color": "#DC2626"
            },
            {
                "id": "data-preservation",
                "name": "Data Preservation Checker",
                "description": "Independently flag any candidate that could be irreplaceable user \
                    data (documents, media, source not in VCS, DB files) or hard to regenerate. \
                    Bias toward 'review' when uncertain. Metadata only.",
                "modality": "text",
                "accent_color": "#F59E0B"
            },
            {
                "id": "lead-synthesizer",
                "name": "Lead Synthesizer",
                "description": "Merge the assessors' notes into one concise per-item verdict list \
                    with justification. Advisory only: recommend, never instruct deletion. Preserve \
                    the highest (most cautious) risk when assessors disagree.",
                "modality": "text",
                "parent_id": "risk-assessor",
                "accent_color": "#4F46E5"
            }
        ] }
    })
}

/// Resolve a team template id: explicit config wins; otherwise find our own team by
/// its stable marker, creating it if the platform doesn't have it yet. No hardcoded
/// numeric id — the platform's ids aren't stable across a DB wipe or a fresh project.
fn resolve_team(c: &reqwest::blocking::Client, cfg: &AiConfig, base: &str) -> Result<i64, String> {
    if let Some(id) = cfg.team_template_id {
        return Ok(id);
    }
    // Look up by marker embedded in the description.
    let teams = fetch_teams(c, cfg, base)?;
    if let Some(id) = teams.iter().find_map(|t| {
        let has_marker = t
            .get("description")
            .and_then(Value::as_str)
            .is_some_and(|d| d.contains(TEAM_MARKER));
        has_marker.then(|| t.get("id").and_then(Value::as_i64)).flatten()
    }) {
        return Ok(id);
    }

    // Not present — provision it. Trailing slash avoids the 307 that would strip POST body.
    let created = auth(c.post(format!("{base}/api/v1/teams/")), cfg)
        .json(&team_body())
        .send()
        .map_err(|e| e.to_string())?;
    if !created.status().is_success() {
        let code = created.status();
        let text = created.text().unwrap_or_default();
        return Err(format!("team provisioning failed: HTTP {code}: {text}"));
    }
    let out: Value = created.json().map_err(|e| e.to_string())?;
    out.get("id")
        .and_then(Value::as_i64)
        .ok_or_else(|| "team create returned no id".to_string())
}

/// GET the platform's team templates as a raw array (handles both shapes).
fn fetch_teams(c: &reqwest::blocking::Client, cfg: &AiConfig, base: &str) -> Result<Vec<Value>, String> {
    let resp = auth(c.get(format!("{base}/api/v1/teams/")), cfg)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("teams list failed: HTTP {}", resp.status()));
    }
    let body: Value = resp.json().map_err(|e| e.to_string())?;
    Ok(body
        .get("teams")
        .and_then(Value::as_array)
        .or_else(|| body.as_array())
        .cloned()
        .unwrap_or_default())
}

fn team_option(t: &Value) -> Option<TeamOption> {
    let id = t.get("id").and_then(Value::as_i64)?;
    let description = t
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some(TeamOption {
        id,
        name: t.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
        is_app_team: description.contains(TEAM_MARKER),
        description,
    })
}

/// Public: list all team templates for the Settings picker. `is_app_team` flags
/// the one matching this app's roster so the UI can preselect / warn if absent.
pub fn list_teams(cfg: AiConfig) -> Result<Vec<TeamOption>, String> {
    let c = client()?;
    let base = base_url(&cfg);
    Ok(fetch_teams(&c, &cfg, &base)?
        .iter()
        .filter_map(team_option)
        .collect())
}

/// Public: create this app's team on demand (Settings "Create team" button).
/// Idempotent — returns the existing app team if the marker is already present.
pub fn provision_team(cfg: AiConfig) -> Result<TeamOption, String> {
    let c = client()?;
    let base = base_url(&cfg);
    if let Some(existing) = fetch_teams(&c, &cfg, &base)?
        .iter()
        .find(|t| {
            t.get("description")
                .and_then(Value::as_str)
                .is_some_and(|d| d.contains(TEAM_MARKER))
        })
        .and_then(team_option)
    {
        return Ok(existing);
    }
    let created = auth(c.post(format!("{base}/api/v1/teams/")), &cfg)
        .json(&team_body())
        .send()
        .map_err(|e| e.to_string())?;
    if !created.status().is_success() {
        let code = created.status();
        let text = created.text().unwrap_or_default();
        return Err(format!("team provisioning failed: HTTP {code}: {text}"));
    }
    let out: Value = created.json().map_err(|e| e.to_string())?;
    team_option(&out).ok_or_else(|| "team create returned no id".to_string())
}

/// Build the review goal. Only metadata is sent — never file contents.
/// Shared with the AI-domain path (`verify_ai.rs`) as the user message.
pub(crate) fn build_goal(root: &str, proposals: &[Proposal]) -> String {
    let items: Vec<Value> = proposals
        .iter()
        .map(|p| {
            json!({
                "path": p.path,
                "kind": p.kind,
                "sizeBytes": p.size_bytes,
                "fileCount": p.file_count,
                "heuristicReason": p.reason,
            })
        })
        .collect();
    format!(
        "You are verifying disk-cleanup candidates found under {root}. For each item, \
        judge whether deleting it (to the Recycle Bin) is safe, needs review, or is \
        dangerous, and explain why. Do NOT invent files or assume contents; reason only \
        from the metadata. Flag anything that could be user data or hard to regenerate. \
        Do NOT use placeholder variables like {{candidate}} — there is no substitution \
        step; embed the actual candidate JSON directly in every agent's instructions so \
        each agent sees the real metadata.\n\n\
        The Lead Synthesizer MUST end its output with a single fenced code block tagged \
        `json` containing ONLY a JSON array of verdicts, one object per candidate, each \
        with exactly these keys: \"path\" (the candidate's path, verbatim), \"verdict\" \
        (one of \"safe\", \"review\", \"dangerous\"), and \"reason\" (one short sentence). \
        Include every candidate exactly once. Example:\n\
        ```json\n[{{\"path\":\"C:/x/node_modules\",\"verdict\":\"safe\",\"reason\":\"Regenerable dependencies.\"}}]\n```\n\n\
        Candidates (JSON):\n{}",
        serde_json::to_string_pretty(&items).unwrap_or_default()
    )
}

/// Extract per-item verdicts from the agents' output. Prefers the Lead
/// Synthesizer's note (which is instructed to emit the JSON block), but falls
/// back to scanning every note so a differently-shaped roster still works.
/// Returns empty when nothing parseable is found — the UI degrades to prose.
pub(crate) fn parse_verdicts(notes: &[AgentNote]) -> Vec<ItemVerdict> {
    // Ordered so the synthesizer wins, then any other note as a fallback.
    let ordered = notes
        .iter()
        .filter(|n| n.role.to_lowercase().contains("synth"))
        .chain(notes.iter());
    for n in ordered {
        if let Some(v) = verdicts_from_text(&n.output) {
            if !v.is_empty() {
                return v;
            }
        }
    }
    Vec::new()
}

/// Pull the first JSON array of `{path, verdict, reason}` out of free text —
/// whether fenced in a ```json block or inline.
fn verdicts_from_text(text: &str) -> Option<Vec<ItemVerdict>> {
    // Prefer a fenced block; otherwise take the first bracket-balanced array.
    let candidate = fenced_json(text).or_else(|| first_json_array(text))?;
    let raw: Vec<Value> = serde_json::from_str(&candidate).ok()?;
    let out: Vec<ItemVerdict> = raw
        .iter()
        .filter_map(|v| {
            let path = v.get("path").and_then(Value::as_str)?.to_string();
            let verdict = normalize_verdict(v.get("verdict").and_then(Value::as_str).unwrap_or(""));
            let reason = v
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Some(ItemVerdict {
                path,
                verdict,
                reason,
            })
        })
        .collect();
    Some(out)
}

fn normalize_verdict(s: &str) -> String {
    match s.trim().to_lowercase().as_str() {
        "safe" => "safe",
        "dangerous" | "danger" | "unsafe" => "dangerous",
        _ => "review",
    }
    .to_string()
}

/// Body of the first ```json … ``` fence, if present.
fn fenced_json(text: &str) -> Option<String> {
    let start = text.find("```json").map(|i| i + "```json".len())?;
    let rest = &text[start..];
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

/// First bracket-balanced `[ … ]` span — a cheap array extractor for un-fenced output.
fn first_json_array(text: &str) -> Option<String> {
    let start = text.find('[')?;
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[start..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn progress_tasks(payload: &Value) -> Vec<VerifyTask> {
    payload
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .map(|t| VerifyTask {
                    role: t
                        .get("role")
                        .and_then(Value::as_str)
                        .unwrap_or("agent")
                        .to_string(),
                    status: t
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Runs the whole synchronous flow on a blocking thread: start → poll → collect.
/// `cancel` is polled between waits so the UI's Stop button returns promptly;
/// `on_progress` is fired on every poll with the live per-task status.
fn run<F: FnMut(VerifyProgress)>(
    root: &str,
    proposals: &[Proposal],
    cfg: &AiConfig,
    cancel: &AtomicBool,
    mut on_progress: F,
) -> Result<VerificationResult, String> {
    if proposals.is_empty() {
        return Err("Nothing to verify.".to_string());
    }
    let c = client()?;
    let base = base_url(cfg);
    let team_id = resolve_team(&c, cfg, &base)?;

    let start = auth(c.post(format!("{base}/api/v1/processes")), cfg)
        .json(&json!({
            "goal": build_goal(root, proposals),
            "auto_approve": true,
            "team_template_id": team_id,
            "client_id": CLIENT_ID,
        }))
        .send()
        .map_err(|e| e.to_string())?;
    if !start.status().is_success() {
        let code = start.status();
        let body = start.text().unwrap_or_default();
        return Err(format!("process start failed: HTTP {code}: {body}"));
    }
    let started: Value = start.json().map_err(|e| e.to_string())?;
    let process_id = started
        .get("process_id")
        .and_then(Value::as_i64)
        .ok_or("process start returned no process_id")?;

    let deadline = Instant::now() + MAX_WAIT;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }
        if Instant::now() >= deadline {
            return Err("Timed out waiting for verification to finish.".to_string());
        }
        std::thread::sleep(POLL_INTERVAL);
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".to_string());
        }

        let resp = auth(c.get(format!("{base}/api/v1/processes/{process_id}")), cfg)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("process poll failed: HTTP {}", resp.status()));
        }
        let payload: Value = resp.json().map_err(|e| e.to_string())?;
        let status = payload
            .get("process")
            .and_then(|p| p.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        // Forward the live per-task snapshot so the UI can show reviewers working.
        on_progress(VerifyProgress {
            process_id,
            status: status.clone(),
            tasks: progress_tasks(&payload),
        });

        let terminal = matches!(status.as_str(), "completed" | "failed" | "cancelled");
        let gated = matches!(status.as_str(), "approval_required" | "task_review_required");
        if terminal || gated {
            let notes = collect_notes(&payload);
            let verdicts = parse_verdicts(&notes);
            return Ok(VerificationResult {
                process_id,
                status,
                notes,
                verdicts,
                gated,
            });
        }
    }
}

fn collect_notes(payload: &Value) -> Vec<AgentNote> {
    let mut notes: Vec<AgentNote> = payload
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .filter_map(|t| {
                    let output = t.get("output").and_then(Value::as_str).unwrap_or("");
                    if output.trim().is_empty() {
                        return None;
                    }
                    Some(AgentNote {
                        task_id: t.get("id").and_then(Value::as_i64).unwrap_or(0),
                        role: t
                            .get("role")
                            .and_then(Value::as_str)
                            .unwrap_or("agent")
                            .to_string(),
                        status: t
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        output: output.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    notes.sort_by_key(|n| n.task_id);
    notes
}

/// Public entry: verify proposals via the platform's multi-agent orchestration.
/// Advisory only — returns agent notes; callers never auto-delete on this basis.
pub fn verify<F: FnMut(VerifyProgress)>(
    root: String,
    proposals: Vec<Proposal>,
    cfg: AiConfig,
    cancel: &AtomicBool,
    on_progress: F,
) -> Result<VerificationResult, String> {
    run(&root, &proposals, &cfg, cancel, on_progress)
}
