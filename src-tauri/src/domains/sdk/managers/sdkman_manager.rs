use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * SDKMAN Manager Implementation
 */
use async_trait::async_trait;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct SdkmanManager;

impl SdkmanManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_shell_command(&self, command: &str) -> Result<String, SDKError> {
        // SDKMAN is typically installed in zsh, so we need to source it first
        // IMPORTANT:
        // `sdk list ...` often renders through a pager (like `less`) which can require
        // interactive "q" to exit. Since we run in a non-interactive context, we force
        // paging to off by setting `PAGER=cat`.
        let full_command = format!(
            "export PAGER=cat LESS='-SRFX' && source ~/.sdkman/bin/sdkman-init.sh && {}",
            command
        );

        let result = CommandExecutor::execute_shell(&full_command, None)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to execute command: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::ManagerNotFound(format!(
                "Command failed: {}",
                result.stderr
            )))
        }
    }

    fn sdkman_sdk_name(&self) -> &str {
        // SDKMAN can manage multiple "candidates" (java, kotlin, ...),
        // but in this app we model SDKMAN as the manager for Java only.
        self.sdk_type()
    }

    fn parse_versions_from_sdk_list_output(
        &self,
        sdk_list_output: &str,
        only_installed: bool,
    ) -> Vec<String> {
        fn strip_ansi(input: &str) -> String {
            // Remove common ANSI escape sequences used for terminal coloring.
            // Example: "\x1b[32m" ... "\x1b[0m"
            let mut out = String::with_capacity(input.len());
            let bytes = input.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == 0x1B {
                    // ESC sequence
                    i += 1;
                    while i < bytes.len() {
                        // CSI sequences end with a letter
                        if (bytes[i] as char).is_ascii_alphabetic() || bytes[i] == b'@' {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                } else {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            }
            out
        }

        let sdk_list_output = strip_ansi(sdk_list_output).replace("\r\n", "\n");
        let mut versions: HashSet<String> = HashSet::new();

        for raw_line in sdk_list_output.lines() {
            let line = raw_line.trim();

            // SDKMAN uses `|`-separated tables for versions.
            if !line.contains('|') {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            // Expected columns:
            // Vendor | Use | Version | Dist | Status | Identifier
            if parts.len() < 6 {
                continue;
            }

            let version = parts[2].trim();
            if version.is_empty() {
                continue;
            }

            // Skip header rows.
            if version.eq_ignore_ascii_case("version") || version.eq_ignore_ascii_case("identifier") {
                continue;
            }

            let status = parts[4].trim().to_lowercase();
            let is_installed = status.contains("installed") || status.contains("local only");

            if only_installed && !is_installed {
                continue;
            }
            versions.insert(version.to_string());
        }

        let mut out: Vec<String> = versions.into_iter().collect();
        out.sort(); // stable-ish fallback ordering
        out
    }

    fn parse_current_version(&self, sdk_current_output: &str) -> Option<String> {
        // Common forms we might see:
        // - "Using: java 21.0.3-tem"
        // - "Using java version 21.0.3-tem"
        // - "SDKMAN 5.x ..."
        //
        // We'll take the first "word token" after the word "version" or the first
        // token that looks like a version (contains digits + a dot).
        for raw_line in sdk_current_output.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            let lower = line.to_lowercase();
            if lower.contains("version") {
                if let Some(idx) = lower.find("version") {
                    let after = line[idx + "version".len()..].trim();
                    if let Some(token) = after.split_whitespace().next() {
                        let token = token.trim().trim_matches('"').trim_matches('\'');
                        if token.contains('.') && token.chars().any(|c| c.is_ascii_digit()) {
                            return Some(token.to_string());
                        }
                    }
                }
            }

            // Fallback heuristic: first token that looks like a version.
            if lower.contains(self.sdkman_sdk_name()) && lower.contains("using") {
                for token in line.split_whitespace() {
                    let token_lower = token.to_lowercase();
                    if token_lower.contains('.') && token_lower.chars().any(|c| c.is_ascii_digit()) {
                        return Some(token.to_string());
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl SDKManager for SdkmanManager {
    fn name(&self) -> &'static str {
        "sdk"
    }
    fn display_name(&self) -> &'static str {
        "SDKMAN"
    }
    fn sdk_type(&self) -> &'static str {
        "java"
    }
    fn category(&self) -> &'static str {
        "language"
    }
    async fn is_installed(&self) -> Result<bool, SDKError> {
        match self.execute_shell_command("sdk version").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let output = self.execute_shell_command("sdk version").await?;
        Ok(output.trim().to_string())
    }

    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> {
        // TEMPORARY DISABLED:
        // SDKMAN output parsing is complex (it includes interactive/pager output and
        // multiple "candidates" like java/kotlin/etc.). We disable version/subtool
        // support for SDKMAN for now.
        Ok(vec![])
    }
    async fn get_current_version(&self) -> Result<Option<String>, SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        Ok(None)
    }
    async fn switch_version(&self, version: &str) -> Result<(), SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        let _ = version;
        Ok(())
    }
    async fn switch_version_for_project(
        &self,
        version: &str,
        _project_path: &str,
    ) -> Result<(), SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        let _ = version;
        Ok(())
    }
    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        let _ = version;
        Ok(false)
    }

    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        let _ = version;
        Ok(())
    }
    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        let _ = version;
        Ok(())
    }
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        // TEMPORARY DISABLED (see list_versions).
        Ok(vec![])
    }
    fn supports_installation(&self) -> bool {
        false
    }

    // === Environment Management ===
    async fn create_project_environment(
        &self,
        _version: &str,
        _project_path: &str,
    ) -> Result<String, SDKError> {
        Ok("".to_string())
    }
    async fn get_environment_variables(
        &self,
        _version: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        Ok(HashMap::new())
    }

    // === Configuration ===
    async fn get_project_config(
        &self,
        _project_path: &str,
    ) -> Result<HashMap<String, String>, SDKError> {
        Ok(HashMap::new())
    }
    async fn set_project_config(
        &self,
        _project_path: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), SDKError> {
        Ok(())
    }

    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok("SDKMAN help - not implemented".to_string())
    }
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec!["sdk install java 11.0.0".to_string()])
    }
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec!["SDKMAN not implemented".to_string()])
    }

    // === Information ===
    async fn get_info(&self) -> Result<HashMap<String, String>, SDKError> {
        Ok(HashMap::new())
    }
}

#[async_trait]
impl SDKManagerDefaults for SdkmanManager {}

#[async_trait]
impl SDKManagerHelpers for SdkmanManager {}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_sdkman_manager_creation() {
        let manager = SdkmanManager::new();
        assert_eq!(manager.name(), "sdk");
        assert_eq!(manager.display_name(), "SDKMAN");
        assert_eq!(manager.sdk_type(), "java");
        assert_eq!(manager.category(), "language");
    }

    #[tokio::test]
    async fn test_sdkman_is_installed() {
        let manager = SdkmanManager::new();
        let result = manager.is_installed().await;

        println!("SDKMAN is_installed result: {:?}", result);

        // This should return Ok(true) if SDKMAN is properly installed
        match result {
            Ok(true) => println!("✅ SDKMAN is detected as installed"),
            Ok(false) => println!("❌ SDKMAN is detected as NOT installed"),
            Err(e) => println!("❌ Error checking SDKMAN installation: {}", e),
        }
    }

    #[tokio::test]
    #[ignore = "requires SDKMAN installed locally"]
    async fn test_sdkman_get_version() {
        let manager = SdkmanManager::new();
        let result = manager.get_manager_version().await;

        println!("SDKMAN get_manager_version result: {:?}", result);

        match result {
            Ok(version) => {
                println!("✅ SDKMAN version detected: {}", version);
                assert!(!version.is_empty(), "Version should not be empty");
            }
            Err(e) => {
                println!("❌ Error getting SDKMAN version: {}", e);
                panic!("Failed to get SDKMAN version: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires SDKMAN installed locally"]
    async fn test_sdkman_shell_command_execution() {
        let manager = SdkmanManager::new();

        // Test the internal shell command execution
        let result = manager.execute_shell_command("sdk version").await;

        println!("SDKMAN shell command result: {:?}", result);

        match result {
            Ok(output) => {
                println!("✅ SDKMAN shell command successful: {}", output);
                assert!(!output.is_empty(), "Output should not be empty");
                assert!(output.contains("SDKMAN"), "Output should contain 'SDKMAN'");
            }
            Err(e) => {
                println!("❌ SDKMAN shell command failed: {}", e);
                panic!("SDKMAN shell command execution failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires SDKMAN installed locally"]
    async fn test_sdkman_direct_shell_test() {
        // Test the exact command our manager uses
        let command = "source ~/.sdkman/bin/sdkman-init.sh && sdk version";

        let result = CommandExecutor::execute_shell(command, None).await;

        println!("Direct shell test result: {:?}", result);

        match result {
            Ok(cmd_result) => {
                println!("✅ Direct shell command successful");
                println!("Success: {}", cmd_result.success);
                println!("Stdout: {}", cmd_result.stdout);
                println!("Stderr: {}", cmd_result.stderr);

                if cmd_result.success {
                    assert!(!cmd_result.stdout.is_empty(), "Stdout should not be empty");
                    assert!(
                        cmd_result.stdout.contains("SDKMAN"),
                        "Output should contain 'SDKMAN'"
                    );
                } else {
                    panic!("Command failed with stderr: {}", cmd_result.stderr);
                }
            }
            Err(e) => {
                println!("❌ Direct shell command failed: {}", e);
                panic!("Direct shell command execution failed: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires SDKMAN installed locally"]
    async fn test_sdkman_file_existence() {
        // Check if SDKMAN files exist
        let sdkman_dir = std::env::var("HOME").unwrap() + "/.sdkman";
        let init_script = sdkman_dir.clone() + "/bin/sdkman-init.sh";

        println!("Checking SDKMAN directory: {}", sdkman_dir);
        println!("Checking init script: {}", init_script);

        let dir_exists = std::path::Path::new(&sdkman_dir).exists();
        let script_exists = std::path::Path::new(&init_script).exists();

        println!("SDKMAN directory exists: {}", dir_exists);
        println!("SDKMAN init script exists: {}", script_exists);

        assert!(dir_exists, "SDKMAN directory should exist");
        assert!(script_exists, "SDKMAN init script should exist");
    }
}
