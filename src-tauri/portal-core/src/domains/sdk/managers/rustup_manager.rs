use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * Rustup Manager Implementation
 *
 * This is a concrete implementation of the SDK manager traits for Rustup.
 * It demonstrates how to implement all required methods with proper error handling.
 */
use async_trait::async_trait;
use std::collections::HashMap;

/// Rustup Manager implementation
#[derive(Default)]
pub struct RustupManager {
    // Add any Rustup-specific configuration here
}

impl RustupManager {
    pub fn new() -> Self {
        Self {}
    }

    async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args(command, args, None)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to execute command: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "{} {} failed: {}",
                command,
                args.join(" "),
                result.stderr.trim()
            )))
        }
    }
}

/// Toolchain names from `rustup toolchain list` / `rustup show active-toolchain`:
/// `stable-x86_64-pc-windows-msvc (active, default)` → `stable-x86_64-pc-windows-msvc`.
/// "no installed toolchains" yields nothing.
fn parse_toolchains(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        // "no installed toolchains", "info:"/"warning:" lines
        .filter(|word| *word != "no" && !word.ends_with(':'))
        .map(str::to_string)
        .collect()
}

const CHANNELS: [&str; 3] = ["stable", "beta", "nightly"];

/// Whether an installed toolchain is the plain `channel` (`stable` or
/// `stable-<host>`), as opposed to a dated one like `nightly-2024-01-01-<host>`.
fn is_channel(toolchain: &str, channel: &str) -> bool {
    match toolchain.strip_prefix(channel) {
        Some("") => true,
        Some(rest) => rest.starts_with('-') && !rest[1..].starts_with(|c: char| c.is_ascii_digit()),
        None => false,
    }
}

#[async_trait]
impl SDKManager for RustupManager {
    fn name(&self) -> &'static str {
        "rustup"
    }

    fn display_name(&self) -> &'static str {
        "Rust Toolchain Manager"
    }

    fn sdk_type(&self) -> &'static str {
        "rust"
    }

    fn category(&self) -> &'static str {
        "language"
    }

    async fn is_installed(&self) -> Result<bool, SDKError> {
        match self.execute_command("rustup", &["--version"]).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let output = self.execute_command("rustup", &["--version"]).await?;
        Ok(output.trim().to_string())
    }

    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self
            .execute_command("rustup", &["toolchain", "list"])
            .await?;
        Ok(parse_toolchains(&output))
    }

    /// The active toolchain's name, so it matches an entry of `list_versions`.
    async fn get_current_version(&self) -> Result<Option<String>, SDKError> {
        let output = self
            .execute_command("rustup", &["show", "active-toolchain"])
            .await?;
        Ok(parse_toolchains(&output).into_iter().next())
    }

    async fn switch_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["default", version])
            .await?;
        Ok(())
    }

    async fn switch_version_for_project(
        &self,
        version: &str,
        project_path: &str,
    ) -> Result<(), SDKError> {
        // Create rust-toolchain file for project-specific version
        let toolchain_file = format!("{}/rust-toolchain", project_path);
        std::fs::write(&toolchain_file, version).map_err(|e| {
            SDKError::ManagerNotFound(format!("Failed to write rust-toolchain: {}", e))
        })?;
        Ok(())
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError> {
        let installed = self.list_versions().await?;
        Ok(installed
            .iter()
            .any(|t| t == version || is_channel(t, version)))
    }

    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["install", version])
            .await?;
        Ok(())
    }

    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["uninstall", version])
            .await?;
        Ok(())
    }

    /// rustup can install any channel, dated nightly or version, but has no "list
    /// remote" command, so offer the three channels not already installed.
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        let installed = self.list_versions().await?;
        Ok(CHANNELS
            .iter()
            .filter(|ch| !installed.iter().any(|t| is_channel(t, ch)))
            .map(|ch| ch.to_string())
            .collect())
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
            "#!/bin/bash\n# Rust Project Environment\n# Generated for project: {}\n# Rust version: {}\n\n# Set Rust toolchain for this project\nexport RUSTUP_TOOLCHAIN={}\n",
            project_path, version, version
        );
        Ok(script)
    }

    async fn get_environment_variables(
        &self,
        version: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        let mut env_vars = HashMap::new();
        env_vars.insert("RUSTUP_TOOLCHAIN".to_string(), version.to_string());
        Ok(env_vars)
    }

    // === Configuration ===
    async fn get_project_config(
        &self,
        project_path: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        let mut config = HashMap::new();
        // Check for rust-toolchain file
        let toolchain_path = format!("{}/rust-toolchain", project_path);
        if std::path::Path::new(&toolchain_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&toolchain_path) {
                config.insert("rust_toolchain".to_string(), content.trim().to_string());
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
        if key == "rust_toolchain" {
            let toolchain_path = format!("{}/rust-toolchain", project_path);
            std::fs::write(&toolchain_path, value).map_err(|e| {
                SDKError::ManagerNotFound(format!("Failed to write rust-toolchain: {}", e))
            })?;
        }
        Ok(())
    }

    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok("Rustup - Rust toolchain installer and version manager\n\nUsage:\n  rustup install <toolchain>    Install a Rust toolchain\n  rustup default <toolchain>     Set default toolchain\n  rustup show                   Show installed toolchains\n  rustup update                 Update all toolchains".to_string())
    }

    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![
            "rustup install stable".to_string(),
            "rustup default stable".to_string(),
            "rustup show".to_string(),
            "rustup update".to_string(),
        ])
    }

    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        let mut issues = Vec::new();

        if !self.is_installed().await? {
            issues.push("Rustup is not installed".to_string());
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
impl SDKManagerDefaults for RustupManager {}

#[async_trait]
impl SDKManagerHelpers for RustupManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toolchains_and_channels() {
        let list = "stable-x86_64-pc-windows-msvc (active, default)\nnightly-2024-01-01-x86_64-pc-windows-msvc\n";
        let toolchains = parse_toolchains(list);
        assert_eq!(
            toolchains,
            [
                "stable-x86_64-pc-windows-msvc",
                "nightly-2024-01-01-x86_64-pc-windows-msvc"
            ]
        );
        assert!(parse_toolchains("no installed toolchains\n").is_empty());

        assert!(is_channel(&toolchains[0], "stable"));
        assert!(is_channel("beta", "beta"));
        assert!(!is_channel(&toolchains[1], "nightly"));
        assert!(!is_channel("stable-x86_64-pc-windows-msvc", "beta"));
    }
}
