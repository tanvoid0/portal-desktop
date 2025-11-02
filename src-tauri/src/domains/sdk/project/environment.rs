/**
 * Environment Manager
 * 
 * Manages environment variables and SDK activation
 */

use super::ProjectEnvironment;
use crate::domains::sdk::SDKError;
use std::path::Path;
use std::collections::HashMap;
use tokio::fs;
use std::env;

pub struct EnvironmentManager;

impl EnvironmentManager {
    /// Setup environment for a project
    pub async fn setup_project_environment(
        project_path: &Path,
        environment: &ProjectEnvironment,
    ) -> Result<(), SDKError> {
        // Create environment script
        let env_script = Self::generate_environment_script(project_path, environment);
        let env_script_path = project_path.join(".portal-env.sh");
        
        fs::write(&env_script_path, env_script).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write environment script: {}", e)))?;

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&env_script_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to get file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&env_script_path, perms).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to set permissions: {}", e)))?;
        }

        Ok(())
    }

    /// Activate environment for a project
    pub async fn activate_project_environment(project_path: &Path) -> Result<(), SDKError> {
        let env_script_path = project_path.join(".portal-env.sh");
        
        if !env_script_path.exists() {
            return Ok(());
        }

        // Source the environment script
        let output = std::process::Command::new("bash")
            .arg("-c")
            .arg(&format!("source {} && env", env_script_path.display()))
            .output()
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to source environment: {}", e)))?;

        if !output.status.success() {
            return Err(SDKError::ManagerNotFound(format!("Failed to activate environment: {}", 
                String::from_utf8_lossy(&output.stderr))));
        }

        Ok(())
    }

    /// Deactivate environment for a project
    pub async fn deactivate_project_environment(project_path: &Path) -> Result<(), SDKError> {
        // Remove environment variables
        let env_vars = Self::get_project_environment_variables(project_path).await?;
        
        for var_name in env_vars.keys() {
            env::remove_var(var_name);
        }

        Ok(())
    }

    /// Get environment variables for a project
    pub async fn get_project_environment_variables(
        project_path: &Path,
    ) -> Result<HashMap<String, String>, SDKError> {
        let env_script_path = project_path.join(".portal-env.sh");
        
        if !env_script_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&env_script_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read environment script: {}", e)))?;

        let mut env_vars = HashMap::new();
        
        for line in content.lines() {
            if line.starts_with("export ") {
                let export_line = line.trim_start_matches("export ");
                if let Some((key, value)) = export_line.split_once('=') {
                    let clean_value = value.trim_matches('"').to_string();
                    env_vars.insert(key.to_string(), clean_value);
                }
            }
        }

        Ok(env_vars)
    }

    /// Generate environment script content
    fn generate_environment_script(
        project_path: &Path,
        environment: &ProjectEnvironment,
    ) -> String {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# Portal Desktop Environment Script\n");
        script.push_str("# Auto-generated - do not edit manually\n\n");
        
        // Set project-specific environment variables
        script.push_str(&format!("export PORTAL_PROJECT_PATH=\"{}\"\n", project_path.display()));
        script.push_str(&format!("export PORTAL_PROJECT_NAME=\"{}\"\n", 
            project_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")));
        
        // Set SDK-specific environment variables
        for (sdk_type, version) in &environment.versions {
            match sdk_type.as_str() {
                "node" => {
                    script.push_str(&format!("export NODE_VERSION=\"{}\"\n", version));
                    script.push_str(&format!("export NVM_DIR=\"$HOME/.nvm\"\n"));
                    script.push_str(&format!("[ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"\n"));
                    script.push_str(&format!("nvm use {}\n", version));
                },
                "python" => {
                    script.push_str(&format!("export PYTHON_VERSION=\"{}\"\n", version));
                    script.push_str(&format!("export PYENV_ROOT=\"$HOME/.pyenv\"\n"));
                    script.push_str(&format!("export PATH=\"$PYENV_ROOT/bin:$PATH\"\n"));
                    script.push_str(&format!("eval \"$(pyenv init -)\"\n"));
                    script.push_str(&format!("pyenv local {}\n", version));
                },
                "ruby" => {
                    script.push_str(&format!("export RUBY_VERSION=\"{}\"\n", version));
                    script.push_str(&format!("export RBENV_ROOT=\"$HOME/.rbenv\"\n"));
                    script.push_str(&format!("export PATH=\"$RBENV_ROOT/bin:$PATH\"\n"));
                    script.push_str(&format!("eval \"$(rbenv init -)\"\n"));
                    script.push_str(&format!("rbenv local {}\n", version));
                },
                "go" => {
                    script.push_str(&format!("export GO_VERSION=\"{}\"\n", version));
                    script.push_str(&format!("export GOROOT=\"$HOME/.portal/sdks/go/{}\"\n", version));
                    script.push_str(&format!("export PATH=\"$GOROOT/bin:$PATH\"\n"));
                },
                "rust" => {
                    script.push_str(&format!("export RUST_VERSION=\"{}\"\n", version));
                    script.push_str(&format!("export RUSTUP_HOME=\"$HOME/.rustup\"\n"));
                    script.push_str(&format!("export CARGO_HOME=\"$HOME/.cargo\"\n"));
                    script.push_str(&format!("export PATH=\"$CARGO_HOME/bin:$PATH\"\n"));
                },
                _ => {
                    script.push_str(&format!("export {}_VERSION=\"{}\"\n", 
                        sdk_type.to_uppercase(), version));
                }
            }
        }
        
        // Add custom environment variables
        for (key, value) in &environment.environment_variables {
            script.push_str(&format!("export {}={}\n", key, value));
        }
        
        script.push_str("\n# Portal Desktop Environment Activated\n");
        let project_name = project_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        script.push_str(&format!("echo \"Portal Desktop environment activated for project: {}\"\n", project_name));
        
        script
    }

    /// Check if environment is active for a project
    pub async fn is_environment_active(project_path: &Path) -> bool {
        if let Ok(portal_project_path) = env::var("PORTAL_PROJECT_PATH") {
            Path::new(&portal_project_path) == project_path
        } else {
            false
        }
    }

    /// Get current active project path
    pub fn get_active_project_path() -> Option<String> {
        env::var("PORTAL_PROJECT_PATH").ok()
    }

    /// Clean up environment files for a project
    pub async fn cleanup_project_environment(project_path: &Path) -> Result<(), SDKError> {
        let env_script_path = project_path.join(".portal-env.sh");
        
        if env_script_path.exists() {
            fs::remove_file(&env_script_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to remove environment script: {}", e)))?;
        }

        Ok(())
    }
}
