//! Git commit preparation and execution for the coder workspace.

use std::process::Command;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domains::ai::platform_config::{PlatformConfig, DESKTOP_CLIENT_ID};
use crate::process_ext::NoWindowExt;

use super::git_status::{git_diff_stats, git_list_changes, GitFileChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitDraft {
    pub branch: Option<String>,
    pub title: String,
    pub summary: String,
    pub changes: Vec<GitFileChange>,
    pub ai_generated: bool,
}

pub fn prepare_commit(workspace: &str) -> Result<GitCommitDraft, String> {
    let stats = git_diff_stats(workspace);
    if !stats.is_repo {
        return Err("Not a git repository".into());
    }
    if !stats.has_changes {
        return Err("No changes to commit".into());
    }

    let changes = git_list_changes(workspace);
    Ok(GitCommitDraft {
        branch: stats.branch,
        title: fallback_commit_title(&changes),
        summary: fallback_commit_summary(&changes),
        changes,
        ai_generated: false,
    })
}

pub fn fallback_commit_title(changes: &[GitFileChange]) -> String {
    if changes.is_empty() {
        return "chore: update workspace".into();
    }

    let mut modified = 0u32;
    let mut added = 0u32;
    let mut deleted = 0u32;
    for c in changes {
        match c.status.as_str() {
            "added" | "untracked" => added += 1,
            "deleted" => deleted += 1,
            _ => modified += 1,
        }
    }

    let prefix = if added > deleted && added >= modified {
        "feat"
    } else if deleted > added && deleted >= modified {
        "chore"
    } else if modified >= added {
        "fix"
    } else {
        "chore"
    };

    if changes.len() == 1 {
        let path = changes[0]
            .path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&changes[0].path);
        format!("{prefix}: update {path}")
    } else {
        format!("{prefix}: update {} files", changes.len())
    }
}

pub fn fallback_commit_summary(changes: &[GitFileChange]) -> String {
    changes
        .iter()
        .map(|c| format!("- {} ({})", c.path, c.status))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Optional LLM suggestion via agent-platform (same transport as smart titles).
pub async fn suggest_commit_message(
    client: &Client,
    cfg: &PlatformConfig,
    changes: &[GitFileChange],
) -> Result<(String, String), String> {
    let file_list: String = changes
        .iter()
        .map(|c| {
            format!(
                "{} ({}) +{} -{}",
                c.path, c.status, c.additions, c.deletions
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let diffs: String = changes
        .iter()
        .take(15)
        .map(|c| {
            let preview = if c.diff.len() > 400 {
                format!("{}…", &c.diff[..400])
            } else {
                c.diff.clone()
            };
            format!("### {}\n{}", c.path, preview)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let url = format!("{}/v1/chat/completions", cfg.base_url.trim_end_matches('/'));
    let model = cfg
        .default_model
        .as_deref()
        .filter(|m| !m.is_empty())
        .unwrap_or("gpt-4o-mini");

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You write git commit messages using Conventional Commits. Reply with JSON only: {\"title\":\"...\",\"summary\":\"...\"}. Title ≤72 chars, imperative mood. Summary is a short bullet list (plain text, lines starting with -)."
            },
            {
                "role": "user",
                "content": format!("Files:\n{file_list}\n\nDiffs:\n{diffs}")
            }
        ],
        "stream": false,
        "max_tokens": 256,
        "temperature": 0.3
    });

    let mut req = client
        .post(&url)
        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
        .json(&body);
    if let Some(token) = &cfg.api_token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("commit message request failed: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("commit message returned {status}: {text}"));
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| format!("commit message invalid json: {e}"))?;
    let content = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("");

    parse_commit_json(content)
}

fn parse_commit_json(raw: &str) -> Result<(String, String), String> {
    let trimmed = raw.trim();
    let json_str = if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("failed to parse commit JSON: {e}"))?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or("commit JSON missing title")?
        .to_string();
    let summary = value
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    let title = if title.len() > 72 {
        format!("{}…", &title[..69])
    } else {
        title
    };

    Ok((title, summary))
}

pub fn git_commit(workspace: &str, title: &str, summary: Option<&str>) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Commit title is required".into());
    }

    let output = Command::new("git")
        .no_window()
        .args(["add", "-A"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("Failed to run git add: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let mut args = vec!["commit", "-m", title];
    let summary_owned;
    if let Some(body) = summary.map(str::trim).filter(|s| !s.is_empty()) {
        summary_owned = body.to_string();
        args.push("-m");
        args.push(&summary_owned);
    }

    let output = Command::new("git")
        .no_window()
        .args(&args)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("Failed to run git commit: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    git_output(workspace, &["rev-parse", "--short", "HEAD"])
}

fn git_output(workspace: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .no_window()
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_commit_json_extracts_title_and_summary() {
        let (title, summary) = parse_commit_json(
            r#"{"title":"feat: add commit flow","summary":"- Add dialog\n- Wire toolbar"}"#,
        )
        .unwrap();
        assert_eq!(title, "feat: add commit flow");
        assert!(summary.contains("Add dialog"));
    }

    #[test]
    fn fallback_title_single_file() {
        let changes = vec![GitFileChange {
            path: "src/foo.rs".into(),
            status: "modified".into(),
            additions: 1,
            deletions: 0,
            diff: String::new(),
        }];
        assert!(fallback_commit_title(&changes).contains("foo.rs"));
    }
}
