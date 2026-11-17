use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use super::super::SDKError;
use crate::command_executor::{CommandExecutor, CommandOptions, CommandResult};
/**
 * NVM (Node Version Manager) Implementation
 *
 * Two different tools share the name:
 * - Unix nvm (nvm-sh) is a shell function, so every call sources `nvm.sh` first.
 * - nvm-windows is a real executable (`nvm.exe` under `%NVM_HOME%`) with its own
 *   subcommands and output formats, and no per-directory switching.
 */
use async_trait::async_trait;
use std::collections::HashMap;

/// NVM Manager implementation
#[derive(Default)]
pub struct NvmManager {
    // Add any NVM-specific configuration here
}

impl NvmManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Run `nvm <args>` with whichever nvm this platform has; stdout on success.
    async fn nvm(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = if cfg!(windows) {
            Self::nvm_windows(args).await?
        } else {
            Self::nvm_sh(args, CommandOptions::default()).await?
        };
        if result.success {
            Ok(result.stdout)
        } else {
            Err(Self::failed(args, &result))
        }
    }

    /// nvm-sh is a shell function, so it needs bash, but the args go in as bash's
    /// positional parameters (`nvm "$@"`), never spliced into the script: a version
    /// or path is always one word, whatever characters it holds.
    async fn nvm_sh(args: &[&str], options: CommandOptions) -> Result<CommandResult, SDKError> {
        const SCRIPT: &str = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
            fi
            nvm "$@"
        "#;
        let argv: Vec<&str> = ["-c", SCRIPT, "nvm"]
            .into_iter()
            .chain(args.iter().copied())
            .collect();
        CommandExecutor::execute_with_args("bash", &argv, Some(options))
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to run nvm: {}", e)))
    }

    /// Invoke nvm-windows directly (no shell). `%NVM_HOME%\nvm.exe` first, since the
    /// app may be launched without nvm's dir on PATH; otherwise trust PATH.
    async fn nvm_windows(args: &[&str]) -> Result<CommandResult, SDKError> {
        let exe = std::env::var_os("NVM_HOME")
            .map(|home| std::path::Path::new(&home).join("nvm.exe"))
            .filter(|p| p.is_file())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "nvm".to_string());
        CommandExecutor::execute_with_args(&exe, args, None)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to run nvm: {}", e)))
    }

    /// nvm-windows prints its errors on stdout, so report both streams.
    fn failed(args: &[&str], result: &CommandResult) -> SDKError {
        let output = format!("{}\n{}", result.stdout.trim(), result.stderr.trim());
        SDKError::CommandFailed(format!("nvm {} failed: {}", args.join(" "), output.trim()))
    }

    fn version_arg() -> &'static str {
        if cfg!(windows) {
            "version"
        } else {
            "--version"
        }
    }

    /// Whether `version` names one of the installed versions (`v` prefix optional).
    async fn has_version(&self, version: &str) -> Result<bool, SDKError> {
        let wanted = version.trim_start_matches('v');
        Ok(self.list_versions().await?.iter().any(|v| v == wanted))
    }
}

/// Parse `nvm list` from either nvm. nvm-windows:
/// ```text
///     24.14.1
///   * 22.13.0 (Currently using 64-bit executable)
/// ```
/// nvm-sh marks the current one `->     v18.17.0` and adds alias lines
/// (`default -> 18 (-> v18.17.0)`, `lts/* -> ...`, `system`); those, and
/// "No installations recognized.", don't start with a version, so they are skipped.
fn parse_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .find(|tok| *tok != "*" && *tok != "->")
                .map(|tok| tok.trim_start_matches('v'))
                .filter(|tok| tok.starts_with(|c: char| c.is_ascii_digit()))
                .map(str::to_string)
        })
        .collect()
}

/// Parse `nvm current` (both nvms): `v22.13.0` → `22.13.0`. Anything that isn't a
/// version ("none", "system", nvm-windows' "No current version. ...") is no version.
fn parse_current(output: &str) -> Option<String> {
    let v = output.trim().trim_start_matches('v');
    v.starts_with(|c: char| c.is_ascii_digit())
        .then(|| v.to_string())
}

/// Parse nvm-windows `nvm list available`, a `|`-separated table whose columns are
/// CURRENT | LTS | OLD STABLE | OLD UNSTABLE. Only the first two are worth offering.
fn parse_windows_available(output: &str) -> Vec<String> {
    let mut versions = Vec::new();
    for line in output.lines() {
        for cell in line.split('|').skip(1).take(2) {
            let cell = cell.trim();
            if cell.starts_with(|c: char| c.is_ascii_digit()) && !versions.iter().any(|v| v == cell)
            {
                versions.push(cell.to_string());
            }
        }
    }
    versions
}

#[async_trait]
impl SDKManager for NvmManager {
    fn name(&self) -> &'static str {
        "nvm"
    }

    fn display_name(&self) -> &'static str {
        "Node Version Manager"
    }

    fn sdk_type(&self) -> &'static str {
        "node"
    }

    fn category(&self) -> &'static str {
        "language"
    }

    async fn is_installed(&self) -> Result<bool, SDKError> {
        Ok(self.nvm(&[Self::version_arg()]).await.is_ok())
    }

    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let output = self.nvm(&[Self::version_arg()]).await?;
        Ok(output.trim().to_string())
    }

    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> {
        let args: &[&str] = if cfg!(windows) {
            &["list"]
        } else {
            &["list", "--no-colors"]
        };
        Ok(parse_list(&self.nvm(args).await?))
    }

    async fn get_current_version(&self) -> Result<Option<String>, SDKError> {
        if cfg!(windows) {
            // nvm-windows may exit non-zero when nothing is active; that is "no version",
            // not a failure. Any other non-zero exit is a real error.
            let result = Self::nvm_windows(&["current"]).await?;
            let current = parse_current(&result.stdout);
            if result.success || result.stdout.contains("No current version") {
                return Ok(current);
            }
            return Err(Self::failed(&["current"], &result));
        }

        Ok(parse_current(&self.nvm(&["current"]).await?))
    }

    async fn switch_version(&self, version: &str) -> Result<(), SDKError> {
        if !cfg!(windows) {
            self.nvm(&["use", version]).await?;
            return Ok(());
        }

        // nvm-windows switches by rewriting the %NVM_SYMLINK% link, which can need admin
        // rights and does not always exit non-zero when it fails. Trust what is active
        // afterwards, not the exit code. When it needs admin it launches a UAC prompt
        // without waiting (elevate.vbs), so "not switched yet" may still complete.
        let output = self.nvm(&["use", version]).await?;
        let wanted = version.trim_start_matches('v');
        let current = self.get_current_version().await?;
        let switched = current
            .as_deref()
            .is_some_and(|c| c == wanted || c.starts_with(&format!("{}.", wanted)));
        if switched {
            Ok(())
        } else {
            Err(SDKError::CommandFailed(format!(
                "nvm use {} did not switch (active: {}). If Windows asked for administrator \
                 approval, approve it and refresh. nvm said: {}",
                version,
                current.as_deref().unwrap_or("none"),
                output.trim()
            )))
        }
    }

    async fn switch_version_for_project(
        &self,
        version: &str,
        project_path: &str,
    ) -> Result<(), SDKError> {
        if cfg!(windows) {
            return Err(SDKError::CommandFailed(
                "nvm-windows has no per-project version switching; it only switches the \
                 global version (nvm use)"
                    .to_string(),
            ));
        }

        let args = ["use", version];
        let options = CommandOptions {
            working_directory: Some(project_path.to_string()),
            ..Default::default()
        };
        let result = Self::nvm_sh(&args, options).await?;
        if result.success {
            Ok(())
        } else {
            Err(Self::failed(&args, &result))
        }
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError> {
        if cfg!(windows) {
            return self.has_version(version).await;
        }

        match self.nvm(&["list", version]).await {
            Ok(output) => Ok(output.contains(version)),
            Err(_) => Ok(false),
        }
    }

    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError> {
        let output = self.nvm(&["install", version]).await?;

        // nvm-windows can print an error and still exit 0. For an exact version we can
        // check the result (prereleases like 27.0.0-alpha.0.1.0 included); aliases
        // (lts, latest, 22) are left to the exit code.
        let exact = version.trim_start_matches('v').split('.').count() >= 3;
        if cfg!(windows) && exact && !self.has_version(version).await? {
            return Err(SDKError::CommandFailed(format!(
                "nvm install {} did not install it: {}",
                version,
                output.trim()
            )));
        }
        Ok(())
    }

    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError> {
        if !cfg!(windows) {
            self.nvm(&["uninstall", version]).await?;
            return Ok(());
        }

        // nvm-windows exits 0 for "node vX is not installed", so check both ends.
        if !self.has_version(version).await? {
            return Err(SDKError::VersionNotFound(format!(
                "Node {} is not installed",
                version
            )));
        }
        let output = self.nvm(&["uninstall", version]).await?;
        if self.has_version(version).await? {
            return Err(SDKError::CommandFailed(format!(
                "nvm uninstall {} left it installed: {}",
                version,
                output.trim()
            )));
        }
        Ok(())
    }

    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        if cfg!(windows) {
            let output = self.nvm(&["list", "available"]).await?;
            let versions = parse_windows_available(&output);
            if versions.is_empty() {
                return Err(SDKError::CommandFailed(format!(
                    "Could not read versions from `nvm list available`: {}",
                    output.trim()
                )));
            }
            return Ok(versions);
        }

        // Same line shape as `nvm list`, plus `(LTS: Iron)` suffixes parse_list ignores.
        Ok(parse_list(&self.nvm(&["ls-remote", "--no-colors"]).await?))
    }

    fn supports_installation(&self) -> bool {
        true
    }

    // === Environment Management ===
    async fn create_project_environment(
        &self,
        version: &str,
        project_path: &str,
    ) -> Result<String, SDKError> {
        let script = format!(
            "#!/bin/bash\n# NVM Project Environment\n# Generated for project: {}\n# Node version: {}\n\n# Set NVM version for this project\nexport NVM_DIR=\"$HOME/.nvm\"\n[ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"\nnvm use {}\n",
            project_path, version, version
        );
        Ok(script)
    }

    async fn get_environment_variables(
        &self,
        version: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        let mut env_vars = HashMap::new();
        env_vars.insert("NODE_VERSION".to_string(), version.to_string());
        env_vars.insert("NVM_DIR".to_string(), "$HOME/.nvm".to_string());
        Ok(env_vars)
    }

    // === Configuration ===
    async fn get_project_config(
        &self,
        project_path: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        let mut config = HashMap::new();
        // Check for .nvmrc file
        let nvmrc_path = format!("{}/.nvmrc", project_path);
        if std::path::Path::new(&nvmrc_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&nvmrc_path) {
                config.insert("node_version".to_string(), content.trim().to_string());
            }
        }
        Ok(config)
    }

    async fn set_project_config(
        &self,
        project_path: &str,
        key: &str,
        value: &str,
    ) -> Result<(), SDKError> {
        if key == "node_version" {
            let nvmrc_path = format!("{}/.nvmrc", project_path);
            std::fs::write(&nvmrc_path, value)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write .nvmrc: {}", e)))?;
        }
        Ok(())
    }

    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok("NVM (Node Version Manager) - Manage multiple Node.js versions\n\nUsage:\n  nvm install <version>    Install a Node.js version\n  nvm use <version>       Switch to a version\n  nvm list               List installed versions\n  nvm current            Show current version".to_string())
    }

    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![
            "nvm install 18.0.0".to_string(),
            "nvm use 18.0.0".to_string(),
            "nvm list".to_string(),
            "nvm current".to_string(),
        ])
    }

    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        let mut issues = Vec::new();

        if !self.is_installed().await? {
            issues.push("NVM is not installed".to_string());
        }

        Ok(issues)
    }

    // === Information ===
    async fn get_info(&self) -> Result<HashMap<String, String>, SDKError> {
        let mut info = HashMap::new();
        info.insert("name".to_string(), self.display_name().to_string());
        info.insert("version".to_string(), self.get_manager_version().await?);
        info.insert("sdk_type".to_string(), self.sdk_type().to_string());
        info.insert("category".to_string(), self.category().to_string());
        Ok(info)
    }
}

#[async_trait]
impl SDKManagerDefaults for NvmManager {}

#[async_trait]
impl SDKManagerHelpers for NvmManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nvm_windows_output() {
        let list = "\r\n    24.14.1\r\n  * 22.13.0 (Currently using 64-bit executable)\r\n    22.12.0\r\n\r\n";
        assert_eq!(parse_list(list), ["24.14.1", "22.13.0", "22.12.0"]);
        assert!(parse_list("\r\nNo installations recognized.\r\n").is_empty());

        // nvm-sh `nvm list --no-colors`
        let unix = "->     v18.17.0\n       v16.20.2\ndefault -> 18 (-> v18.17.0)\nlts/* -> lts/hydrogen (-> v18.17.0)\nsystem\n";
        assert_eq!(parse_list(unix), ["18.17.0", "16.20.2"]);
        // nvm-sh `nvm ls-remote --no-colors`
        let remote = "        v20.10.0\n       v20.11.0   (LTS: Iron)\n->     v22.13.0   (Latest LTS: Jod)\n";
        assert_eq!(parse_list(remote), ["20.10.0", "20.11.0", "22.13.0"]);

        assert_eq!(parse_current("v22.13.0\r\n").as_deref(), Some("22.13.0"));
        assert_eq!(
            parse_current("No current version. Run 'nvm use x.x.x' to set a version."),
            None
        );

        let available = "\
|      CURRENT       |     LTS      |  OLD STABLE  | OLD UNSTABLE |
|--------------------|--------------|--------------|--------------|
| 27.0.0-alpha.0.1.0 |   26.13.1    |   0.12.18    |   0.11.16    |
|      26.12.1       |   24.22.3    |   0.12.16    |   0.11.14    |

This is a partial list. For a complete list, visit https://nodejs.org/en/download/releases";
        assert_eq!(
            parse_windows_available(available),
            ["27.0.0-alpha.0.1.0", "26.13.1", "26.12.1", "24.22.3"]
        );
    }

    /// A stub nvm.sh echoes its cwd and argv: hostile args arrive as single words
    /// and run nothing, and a project path with a space is the working directory.
    #[cfg(unix)]
    #[tokio::test]
    async fn nvm_sh_passes_args_as_words() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir(home.path().join(".nvm")).unwrap();
        std::fs::write(
            home.path().join(".nvm/nvm.sh"),
            "nvm() { pwd; printf '%s\\n' \"$@\"; }\n",
        )
        .unwrap();
        let project = home.path().join("my proj");
        std::fs::create_dir(&project).unwrap();

        let hostile = ["use", "1; touch pwned", "$(touch pwned2)", "a b"];
        let options = CommandOptions {
            working_directory: Some(project.to_string_lossy().into_owned()),
            environment: Some(HashMap::from([(
                "HOME".to_string(),
                home.path().to_string_lossy().into_owned(),
            )])),
            ..Default::default()
        };
        let result = NvmManager::nvm_sh(&hostile, options).await.unwrap();

        assert!(result.success, "{}", result.stderr);
        let lines: Vec<&str> = result.stdout.lines().collect();
        assert!(lines[0].ends_with("/my proj"), "cwd: {}", lines[0]);
        assert_eq!(lines[1..], hostile);
        assert!(!project.join("pwned").exists() && !project.join("pwned2").exists());
    }
}
