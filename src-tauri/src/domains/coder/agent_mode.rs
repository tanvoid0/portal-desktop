//! Agent mode semantics: permission flags, tool filtering, and per-turn instructions.

use super::tools;
use super::types::{CoderAgentMode, PermissionMode};

/// Platform API flags: `(allow_commands, auto_approve_commands)`.
pub fn permission_flags(agent: CoderAgentMode, permission: PermissionMode) -> (bool, bool) {
    let auto = permission == PermissionMode::AutoAcceptAll;
    match agent {
        CoderAgentMode::Ask => (false, false),
        CoderAgentMode::Plan | CoderAgentMode::Debug | CoderAgentMode::Multitask
        | CoderAgentMode::Auto => (true, auto),
    }
}

/// Whether a tool should be advertised to the model for this mode.
pub fn includes_tool(mode: CoderAgentMode, tool: &str) -> bool {
    match mode {
        CoderAgentMode::Ask => tools::is_read_only(tool),
        CoderAgentMode::Plan => {
            tools::is_read_only(tool) || matches!(tool, "run_command" | "write_file" | "edit_file")
        }
        CoderAgentMode::Debug | CoderAgentMode::Auto | CoderAgentMode::Multitask => true,
    }
}

/// Plan mode may write only under `.cursor/plans/**/*.plan.md`.
pub fn is_plan_write_allowed(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.starts_with(".cursor/plans/") && normalized.ends_with(".plan.md")
}

pub fn mode_instruction(mode: CoderAgentMode) -> &'static str {
    match mode {
        CoderAgentMode::Plan => concat!(
            "You are in Plan mode. Explore the codebase with read tools and run_command as needed. ",
            "Ask clarifying questions before committing to an approach. ",
            "Produce a step-by-step implementation plan and save it to `.cursor/plans/<slug>.plan.md`. ",
            "Do not modify source files until the user approves the plan."
        ),
        CoderAgentMode::Debug => concat!(
            "You are in Debug mode. Form hypotheses about the root cause, add targeted logging or ",
            "instrumentation, run repro commands, analyze output, and make evidence-based fixes. ",
            "Prefer minimal, targeted changes."
        ),
        CoderAgentMode::Multitask => concat!(
            "You are in Multitask mode on a coordinator thread. Fan out independent tasks to parallel ",
            "worktree-backed sub-agents when work can run separately. Use spawn_parallel_tasks or paste ",
            "multiple GitHub issue URLs."
        ),
        CoderAgentMode::Ask => concat!(
            "You are in Ask mode. Answer questions and explore the codebase read-only. ",
            "Do not write files, edit files, or run shell commands."
        ),
        CoderAgentMode::Auto => concat!(
            "You are in Auto mode. Choose the best strategy for the user's request: read-only exploration ",
            "for questions, planning workflow for complex features, debug workflow for bugs, and direct ",
            "implementation for straightforward tasks."
        ),
    }
}

pub fn mode_as_str(mode: CoderAgentMode) -> &'static str {
    mode_label(mode)
}

/// Strip legacy `[mode]\n{instruction}\n\n{user}` prefixes from platform history.
pub fn unwrap_legacy_wrapped_user_message(content: &str) -> String {
    for mode in [
        CoderAgentMode::Plan,
        CoderAgentMode::Debug,
        CoderAgentMode::Multitask,
        CoderAgentMode::Ask,
        CoderAgentMode::Auto,
    ] {
        let label = mode_label(mode);
        let prefix = format!("[{label} mode]\n");
        if !content.starts_with(&prefix) {
            continue;
        }
        let rest = &content[prefix.len()..];
        let instruction = mode_instruction(mode);
        if rest.starts_with(instruction) {
            let after = &rest[instruction.len()..];
            if let Some(user) = after.strip_prefix("\n\n") {
                return user.to_string();
            }
        }
        if let Some(idx) = rest.find("\n\n") {
            return rest[idx + 2..].to_string();
        }
    }
    content.to_string()
}

pub fn normalize_platform_user_messages(messages: &mut [super::types::ChatMessage]) {
    for message in messages.iter_mut() {
        if message.role != "user" {
            continue;
        }
        if let Some(content) = message.content.take() {
            message.content = Some(unwrap_legacy_wrapped_user_message(&content));
        }
    }
}

fn mode_label(mode: CoderAgentMode) -> &'static str {
    match mode {
        CoderAgentMode::Plan => "plan",
        CoderAgentMode::Debug => "debug",
        CoderAgentMode::Multitask => "multitask",
        CoderAgentMode::Ask => "ask",
        CoderAgentMode::Auto => "auto",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_write_path_allowed() {
        assert!(is_plan_write_allowed(".cursor/plans/foo.plan.md"));
        assert!(!is_plan_write_allowed("src/main.rs"));
        assert!(!is_plan_write_allowed(".cursor/plans/readme.md"));
    }

    #[test]
    fn ask_excludes_mutating_tools() {
        assert!(includes_tool(CoderAgentMode::Ask, "read_file"));
        assert!(!includes_tool(CoderAgentMode::Ask, "write_file"));
        assert!(!includes_tool(CoderAgentMode::Ask, "run_command"));
    }

    #[test]
    fn plan_includes_run_command() {
        assert!(includes_tool(CoderAgentMode::Plan, "run_command"));
        // Plan mode advertises write_file so the model can save the .plan.md;
        // the permission gate restricts the path to .cursor/plans/*.plan.md.
        assert!(includes_tool(CoderAgentMode::Plan, "write_file"));
    }

    #[test]
    fn unwrap_legacy_auto_mode_prefix() {
        let wrapped = format!(
            "[auto mode]\n{}\n\nHello",
            mode_instruction(CoderAgentMode::Auto)
        );
        assert_eq!(
            unwrap_legacy_wrapped_user_message(&wrapped),
            "Hello".to_string()
        );
        assert_eq!(
            unwrap_legacy_wrapped_user_message("plain hello"),
            "plain hello".to_string()
        );
    }
}
