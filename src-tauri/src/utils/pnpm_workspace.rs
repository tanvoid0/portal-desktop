//! pnpm workspace file quirks shared by pipeline execution and Coder tools.

use std::path::Path;

/// True when `pnpm-workspace.yaml` exists but has no `packages:` field.
pub fn has_broken_pnpm_workspace(project_path: &str) -> bool {
    let workspace_file = Path::new(project_path).join("pnpm-workspace.yaml");
    if !workspace_file.exists() {
        return false;
    }

    let contents = match std::fs::read_to_string(workspace_file) {
        Ok(c) => c,
        Err(_) => return false,
    };

    !contents
        .lines()
        .any(|line| line.trim_start().starts_with("packages:"))
}

/// Human-readable note when [`has_broken_pnpm_workspace`] applies.
pub fn warn_if_broken_pnpm_workspace(project_path: &str) -> Option<String> {
    if !has_broken_pnpm_workspace(project_path) {
        return None;
    }

    Some(
        "warning: pnpm-workspace.yaml is missing a packages field — using --ignore-workspace for this run"
            .to_string(),
    )
}

/// Rewrite pnpm commands to pass `--ignore-workspace` when the workspace file is broken.
pub fn prepare_shell_command(command: &str, working_directory: &str) -> String {
    let trimmed = command.trim();
    if !trimmed.starts_with("pnpm") {
        return trimmed.to_string();
    }
    if trimmed.contains("--ignore-workspace") || !has_broken_pnpm_workspace(working_directory) {
        return trimmed.to_string();
    }

    trimmed.replacen("pnpm", "pnpm --ignore-workspace", 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_root() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("pnpm_ws_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_missing_packages_field() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        fs::write(
            root.join("pnpm-workspace.yaml"),
            "onlyBuiltDependencies:\n  - esbuild\n",
        )
        .unwrap();
        assert!(has_broken_pnpm_workspace(root_s));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn accepts_valid_workspace_file() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        fs::write(
            root.join("pnpm-workspace.yaml"),
            "packages:\n  - packages/*\n",
        )
        .unwrap();
        assert!(!has_broken_pnpm_workspace(root_s));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn injects_ignore_workspace_for_broken_file() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        fs::write(
            root.join("pnpm-workspace.yaml"),
            "onlyBuiltDependencies:\n  - esbuild\n",
        )
        .unwrap();
        let prepared = prepare_shell_command("pnpm run dev", root_s);
        assert_eq!(prepared, "pnpm --ignore-workspace run dev");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn leaves_non_pnpm_commands_unchanged() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        assert_eq!(prepare_shell_command("npm run dev", root_s), "npm run dev");
        fs::remove_dir_all(&root).ok();
    }
}
