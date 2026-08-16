use regex::Regex;

use super::types::{GitHubIssueRef, SpawnSubAgentTask};

pub const DEFAULT_MAX_PARALLEL_SUBAGENTS: usize = 4;

pub fn parse_issue_urls(text: &str) -> Vec<String> {
    let re = Regex::new(r"https://github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)").unwrap();
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

pub fn parse_issue_url(url: &str) -> Option<GitHubIssueRef> {
    let re = Regex::new(r"^https://github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)$").ok()?;
    let caps = re.captures(url.trim())?;
    Some(GitHubIssueRef {
        owner: caps.get(1)?.as_str().to_string(),
        repo: caps.get(2)?.as_str().to_string(),
        number: caps.get(3)?.as_str().parse().ok()?,
        url: Some(url.trim().to_string()),
    })
}

pub fn fallback_prompt_for_task(task: &SpawnSubAgentTask) -> String {
    if !task.prompt.trim().is_empty() {
        return task.prompt.trim().to_string();
    }
    if let Some(issue) = &task.github_issue {
        return format!(
            "Work on GitHub issue #{} in {}/{}. Inspect the issue, implement the requested changes, run focused verification, and summarize the result.",
            issue.number, issue.owner, issue.repo
        );
    }
    format!(
        "Complete this task in an isolated worktree and summarize the changes: {}",
        task.title
    )
}
