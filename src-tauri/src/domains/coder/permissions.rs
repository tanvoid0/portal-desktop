//! Permission gating: decides whether a tool call runs, is rejected, or needs
//! a human decision — based on the session mode plus a persisted allow/deny
//! rule list.

use serde_json::Value;

use super::tools;
use super::types::{PermissionMode, PermissionRule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Run it now.
    Allow,
    /// Hard-refuse (Plan mode, or an explicit deny rule).
    Deny(String),
    /// Pause and ask the user.
    Prompt,
}

/// Resolve what to do with a tool call.
pub fn decide(
    mode: PermissionMode,
    rules: &[PermissionRule],
    tool: &str,
    args: &Value,
) -> Decision {
    if tools::is_read_only(tool) {
        return Decision::Allow;
    }

    // Explicit deny rules win over everything except read-only.
    for rule in rules.iter().filter(|r| !r.allow && r.tool == tool) {
        if matches(tool, &rule.pattern, args) {
            return Decision::Deny(format!("blocked by deny rule: {} {}", rule.tool, rule.pattern));
        }
    }

    match mode {
        PermissionMode::Plan => {
            Decision::Deny(format!("plan mode: mutating tool `{tool}` is disabled"))
        }
        PermissionMode::AutoAcceptAll => Decision::Allow,
        PermissionMode::Review => {
            for rule in rules.iter().filter(|r| r.allow && r.tool == tool) {
                if matches(tool, &rule.pattern, args) {
                    return Decision::Allow;
                }
            }
            Decision::Prompt
        }
    }
}

/// Does `pattern` cover this call's arguments?
///
/// - `run_command`: prefix match on the command string (token-aware).
/// - path tools: glob match on the target path.
/// - empty pattern: matches anything for that tool.
fn matches(tool: &str, pattern: &str, args: &Value) -> bool {
    if pattern.is_empty() || pattern == "*" {
        return true;
    }
    match tool {
        "run_command" => {
            let cmd = args.get("command").and_then(Value::as_str).unwrap_or("");
            // `git` matches `git ...`; `git status` matches exactly/prefix.
            cmd == pattern || cmd.starts_with(&format!("{pattern} ")) || cmd.starts_with(pattern) && pattern.contains(' ')
        }
        _ => {
            let path = args.get("path").and_then(Value::as_str).unwrap_or("");
            glob_match(pattern, path)
        }
    }
}

/// Minimal glob supporting `*` (any within a segment run) and `**` (any depth).
/// Good enough for allowlist scoping like `src/**` or `src/*`.
fn glob_match(pattern: &str, text: &str) -> bool {
    // Convert to a simple recursive matcher over bytes.
    fn inner(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        if p[0] == b'*' {
            // Collapse ** into *.
            let rest = if p.len() > 1 && p[1] == b'*' { &p[2..] } else { &p[1..] };
            // `*` matches zero or more chars.
            if inner(rest, t) {
                return true;
            }
            for i in 0..t.len() {
                if inner(rest, &t[i + 1..]) {
                    return true;
                }
            }
            return false;
        }
        if !t.is_empty() && (p[0] == t[0]) {
            return inner(&p[1..], &t[1..]);
        }
        false
    }
    inner(pattern.as_bytes(), text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rule(tool: &str, pat: &str, allow: bool) -> PermissionRule {
        PermissionRule { tool: tool.into(), pattern: pat.into(), allow }
    }

    #[test]
    fn read_only_always_allowed() {
        let d = decide(PermissionMode::Plan, &[], "read_file", &json!({"path": "a"}));
        assert_eq!(d, Decision::Allow);
    }

    #[test]
    fn plan_blocks_writes() {
        let d = decide(PermissionMode::Plan, &[], "write_file", &json!({"path": "a"}));
        assert!(matches!(d, Decision::Deny(_)));
    }

    #[test]
    fn review_prompts_without_rule() {
        let d = decide(PermissionMode::Review, &[], "run_command", &json!({"command": "ls"}));
        assert_eq!(d, Decision::Prompt);
    }

    #[test]
    fn allowlist_grants_command_prefix() {
        let rules = [rule("run_command", "git", true)];
        let d = decide(PermissionMode::Review, &rules, "run_command", &json!({"command": "git status"}));
        assert_eq!(d, Decision::Allow);
    }

    #[test]
    fn deny_rule_beats_auto_accept() {
        let rules = [rule("run_command", "rm", false)];
        let d = decide(PermissionMode::AutoAcceptAll, &rules, "run_command", &json!({"command": "rm -rf x"}));
        assert!(matches!(d, Decision::Deny(_)));
    }

    #[test]
    fn glob_scopes_writes() {
        let rules = [rule("write_file", "src/**", true)];
        let allow = decide(PermissionMode::Review, &rules, "write_file", &json!({"path": "src/a/b.rs"}));
        assert_eq!(allow, Decision::Allow);
        let prompt = decide(PermissionMode::Review, &rules, "write_file", &json!({"path": "README.md"}));
        assert_eq!(prompt, Decision::Prompt);
    }
}
