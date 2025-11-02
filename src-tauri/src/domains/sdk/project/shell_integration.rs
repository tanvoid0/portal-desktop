/**
 * Shell Integration Module
 * 
 * Provides shell integration for automatic version switching
 */

use super::super::SDKError;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

pub struct ShellIntegration;

impl ShellIntegration {
    /// Setup shell integration for a project
    pub async fn setup_shell_integration(
        project_path: &Path,
        environment: &HashMap<String, String>,
    ) -> Result<(), SDKError> {
        let shell = Self::detect_shell()?;
        
        match shell.as_str() {
            "zsh" => Self::setup_zsh_integration(project_path, environment).await,
            "bash" => Self::setup_bash_integration(project_path, environment).await,
            "fish" => Self::setup_fish_integration(project_path, environment).await,
            _ => Err(SDKError::ManagerNotFound(format!("Unsupported shell: {}", shell))),
        }
    }

    /// Detect the current shell
    fn detect_shell() -> Result<String, SDKError> {
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Ok("zsh".to_string());
            } else if shell.contains("bash") {
                return Ok("bash".to_string());
            } else if shell.contains("fish") {
                return Ok("fish".to_string());
            }
        }
        
        // Default to bash if detection fails
        Ok("bash".to_string())
    }

    /// Setup zsh integration
    async fn setup_zsh_integration(
        project_path: &Path,
        environment: &HashMap<String, String>,
    ) -> Result<(), SDKError> {
        let hook_script = Self::generate_zsh_hook(project_path, environment);
        let hook_file = dirs::home_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Home directory not found".to_string()))?
            .join(".zshrc");

        Self::append_to_shell_config(&hook_file, &hook_script)?;
        Ok(())
    }

    /// Setup bash integration
    async fn setup_bash_integration(
        project_path: &Path,
        environment: &HashMap<String, String>,
    ) -> Result<(), SDKError> {
        let hook_script = Self::generate_bash_hook(project_path, environment);
        let hook_file = dirs::home_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Home directory not found".to_string()))?
            .join(".bashrc");

        Self::append_to_shell_config(&hook_file, &hook_script)?;
        Ok(())
    }

    /// Setup fish integration
    async fn setup_fish_integration(
        project_path: &Path,
        environment: &HashMap<String, String>,
    ) -> Result<(), SDKError> {
        let hook_script = Self::generate_fish_hook(project_path, environment);
        let hook_file = dirs::config_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Config directory not found".to_string()))?
            .join("fish")
            .join("conf.d")
            .join("portal.fish");

        fs::create_dir_all(hook_file.parent().unwrap())?;
        fs::write(&hook_file, hook_script)?;
        Ok(())
    }

    /// Generate zsh hook script
    fn generate_zsh_hook(_project_path: &Path, environment: &HashMap<String, String>) -> String {
        let mut script = String::new();
        script.push_str("# Portal SDK Environment Hook\n");
        script.push_str("portal_chpwd() {\n");
        script.push_str("  if [[ -f .portal-version ]]; then\n");
        script.push_str("    source .portal-version\n");
        
        for (key, value) in environment {
            script.push_str(&format!("    export {}={}\n", key, value));
        }
        
        script.push_str("    echo \"Portal environment activated\"\n");
        script.push_str("  else\n");
        script.push_str("    # Deactivate portal environment\n");
        script.push_str("    unset PORTAL_ACTIVE\n");
        script.push_str("  fi\n");
        script.push_str("}\n");
        script.push_str("autoload -U add-zsh-hook\n");
        script.push_str("add-zsh-hook chpwd portal_chpwd\n");
        script.push_str("# Run on startup\n");
        script.push_str("portal_chpwd\n");
        
        script
    }

    /// Generate bash hook script
    fn generate_bash_hook(_project_path: &Path, environment: &HashMap<String, String>) -> String {
        let mut script = String::new();
        script.push_str("# Portal SDK Environment Hook\n");
        script.push_str("portal_chpwd() {\n");
        script.push_str("  if [[ -f .portal-version ]]; then\n");
        script.push_str("    source .portal-version\n");
        
        for (key, value) in environment {
            script.push_str(&format!("    export {}={}\n", key, value));
        }
        
        script.push_str("    echo \"Portal environment activated\"\n");
        script.push_str("  else\n");
        script.push_str("    # Deactivate portal environment\n");
        script.push_str("    unset PORTAL_ACTIVE\n");
        script.push_str("  fi\n");
        script.push_str("}\n");
        script.push_str("PROMPT_COMMAND=\"portal_chpwd; $PROMPT_COMMAND\"\n");
        script.push_str("# Run on startup\n");
        script.push_str("portal_chpwd\n");
        
        script
    }

    /// Generate fish hook script
    fn generate_fish_hook(_project_path: &Path, environment: &HashMap<String, String>) -> String {
        let mut script = String::new();
        script.push_str("# Portal SDK Environment Hook\n");
        script.push_str("function portal_chpwd --on-variable PWD\n");
        script.push_str("  if test -f .portal-version\n");
        script.push_str("    source .portal-version\n");
        
        for (key, value) in environment {
            script.push_str(&format!("    set -gx {} {}\n", key, value));
        }
        
        script.push_str("    echo \"Portal environment activated\"\n");
        script.push_str("  else\n");
        script.push_str("    # Deactivate portal environment\n");
        script.push_str("    set -e PORTAL_ACTIVE\n");
        script.push_str("  end\n");
        script.push_str("end\n");
        script.push_str("# Run on startup\n");
        script.push_str("portal_chpwd\n");
        
        script
    }

    /// Append script to shell config file
    fn append_to_shell_config(config_file: &Path, script: &str) -> Result<(), SDKError> {
        let mut content = if config_file.exists() {
            fs::read_to_string(config_file)?
        } else {
            String::new()
        };

        // Check if portal integration already exists
        if content.contains("# Portal SDK Environment Hook") {
            return Ok(()); // Already integrated
        }

        content.push_str("\n");
        content.push_str(script);
        content.push_str("\n");

        fs::write(config_file, content)?;
        Ok(())
    }

    /// Remove shell integration
    pub async fn remove_shell_integration() -> Result<(), SDKError> {
        let shell = Self::detect_shell()?;
        
        match shell.as_str() {
            "zsh" => Self::remove_zsh_integration().await,
            "bash" => Self::remove_bash_integration().await,
            "fish" => Self::remove_fish_integration().await,
            _ => Err(SDKError::ManagerNotFound(format!("Unsupported shell: {}", shell))),
        }
    }

    /// Remove zsh integration
    async fn remove_zsh_integration() -> Result<(), SDKError> {
        let hook_file = dirs::home_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Home directory not found".to_string()))?
            .join(".zshrc");

        if hook_file.exists() {
            let content = fs::read_to_string(&hook_file)?;
            let lines: Vec<&str> = content.lines().collect();
            let filtered_lines: Vec<&str> = lines
                .into_iter()
                .filter(|line| !line.contains("# Portal SDK Environment Hook"))
                .collect();
            
            fs::write(&hook_file, filtered_lines.join("\n"))?;
        }
        Ok(())
    }

    /// Remove bash integration
    async fn remove_bash_integration() -> Result<(), SDKError> {
        let hook_file = dirs::home_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Home directory not found".to_string()))?
            .join(".bashrc");

        if hook_file.exists() {
            let content = fs::read_to_string(&hook_file)?;
            let lines: Vec<&str> = content.lines().collect();
            let filtered_lines: Vec<&str> = lines
                .into_iter()
                .filter(|line| !line.contains("# Portal SDK Environment Hook"))
                .collect();
            
            fs::write(&hook_file, filtered_lines.join("\n"))?;
        }
        Ok(())
    }

    /// Remove fish integration
    async fn remove_fish_integration() -> Result<(), SDKError> {
        let hook_file = dirs::config_dir()
            .ok_or_else(|| SDKError::ManagerNotFound("Config directory not found".to_string()))?
            .join("fish")
            .join("conf.d")
            .join("portal.fish");

        if hook_file.exists() {
            fs::remove_file(&hook_file)?;
        }
        Ok(())
    }

    /// Check if shell integration is active
    pub fn is_shell_integration_active() -> bool {
        std::env::var("PORTAL_ACTIVE").is_ok()
    }

    /// Get current portal environment
    pub fn get_current_environment() -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with("PORTAL_") || key.starts_with("GOROOT") || key.starts_with("GOPATH") 
                || key.starts_with("NODE_VERSION") || key.starts_with("PYTHON_VERSION") {
                env.insert(key, value);
            }
        }
        
        env
    }
}