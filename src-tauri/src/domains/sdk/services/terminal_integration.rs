/**
 * Terminal Integration Service - FlyEnv-style automatic environment switching
 * Automatically sources project-specific environment when opening terminals
 */

use std::path::Path;
use std::fs;
// Using println! for logging as per codebase convention
use crate::domains::sdk::services::SDKService;
use super::super::SDKError;

#[derive(Debug)]
pub struct TerminalIntegrationService {
    #[allow(dead_code)]
    sdk_service: SDKService,
}

impl TerminalIntegrationService {
    pub fn new(sdk_service: SDKService) -> Self {
        Self { sdk_service }
    }

    /// Setup terminal integration for a project directory
    /// This creates shell hooks that automatically source the project environment
    pub async fn setup_project_terminal_integration(&self, project_path: &str) -> Result<(), SDKError> {
        let project_path = Path::new(project_path);
        
        // Check if project has environment configuration
        if !self.has_project_environment(project_path).await? {
            println!("[TerminalIntegration] No project environment found for {}", project_path.display());
            return Ok(());
        }

        // Create shell integration scripts
        self.create_shell_integration_scripts(project_path).await?;
        
        // Create project-specific shell profile
        self.create_project_shell_profile(project_path).await?;

        println!("[TerminalIntegration] Terminal integration setup completed for project {}", project_path.display());
        Ok(())
    }

    /// Check if project has environment configuration
    async fn has_project_environment(&self, project_path: &Path) -> Result<bool, SDKError> {
        let config_files = vec![
            ".nvmrc", ".python-version", ".ruby-version", 
            ".php-version", "rust-toolchain", "go.mod", ".portal_env"
        ];

        for config_file in config_files {
            if project_path.join(config_file).exists() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Create shell integration scripts for automatic environment sourcing
    async fn create_shell_integration_scripts(&self, project_path: &Path) -> Result<(), SDKError> {
        // Create bash integration
        self.create_bash_integration(project_path).await?;
        
        // Create zsh integration
        self.create_zsh_integration(project_path).await?;
        
        // Create fish integration
        self.create_fish_integration(project_path).await?;

        Ok(())
    }

    /// Create bash integration script
    async fn create_bash_integration(&self, project_path: &Path) -> Result<(), SDKError> {
        let bash_script = format!(
            r#"#!/bin/bash
# Portal Desktop Project Environment Integration
# Auto-generated for project: {}

# Function to check if we're in the project directory
is_in_project() {{
    local current_dir="$(pwd)"
    local project_dir="{}"
    [[ "$current_dir" == "$project_dir"* ]]
}}

# Function to source project environment
source_project_env() {{
    if is_in_project && [[ -f "{}/.portal_env" ]]; then
        source "{}/.portal_env"
        echo "🌍 Portal: Project environment activated"
    fi
}}

# Hook into bash prompt
if [[ -n "$PS1" ]]; then
    # Add to PROMPT_COMMAND for automatic sourcing
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="source_project_env"
    else
        PROMPT_COMMAND="$PROMPT_COMMAND; source_project_env"
    fi
fi

# Source immediately if we're already in the project
source_project_env
"#,
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display()
        );

        let script_path = project_path.join(".portal_bash_integration");
        fs::write(&script_path, bash_script)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create bash integration: {}", e)))?;

        // Make it executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        Ok(())
    }

    /// Create zsh integration script
    async fn create_zsh_integration(&self, project_path: &Path) -> Result<(), SDKError> {
        let zsh_script = format!(
            r#"#!/bin/zsh
# Portal Desktop Project Environment Integration
# Auto-generated for project: {}

# Function to check if we're in the project directory
is_in_project() {{
    local current_dir="$(pwd)"
    local project_dir="{}"
    [[ "$current_dir" == "$project_dir"* ]]
}}

# Function to source project environment
source_project_env() {{
    if is_in_project && [[ -f "{}/.portal_env" ]]; then
        source "{}/.portal_env"
        echo "🌍 Portal: Project environment activated"
    fi
}}

# Hook into zsh prompt
autoload -U add-zsh-hook
add-zsh-hook chpwd source_project_env

# Source immediately if we're already in the project
source_project_env
"#,
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display()
        );

        let script_path = project_path.join(".portal_zsh_integration");
        fs::write(&script_path, zsh_script)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create zsh integration: {}", e)))?;

        // Make it executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        Ok(())
    }

    /// Create fish integration script
    async fn create_fish_integration(&self, project_path: &Path) -> Result<(), SDKError> {
        let fish_script = format!(
            r#"# Portal Desktop Project Environment Integration
# Auto-generated for project: {}

function is_in_project
    set current_dir (pwd)
    set project_dir "{}"
    string match -q "$project_dir*" "$current_dir"
end

function source_project_env
    if is_in_project and test -f "{}/.portal_env"
        source "{}/.portal_env"
        echo "🌍 Portal: Project environment activated"
    end
end

# Hook into fish prompt
function fish_prompt
    source_project_env
    # Call the original prompt
    if functions -q fish_prompt_original
        fish_prompt_original
    else
        # Fallback to default prompt
        echo -n (prompt_pwd) "> "
    end
end

# Source immediately if we're already in the project
source_project_env
"#,
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display()
        );

        let script_path = project_path.join(".portal_fish_integration");
        fs::write(&script_path, fish_script)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create fish integration: {}", e)))?;

        Ok(())
    }

    /// Create project-specific shell profile
    async fn create_project_shell_profile(&self, project_path: &Path) -> Result<(), SDKError> {
        let profile_content = format!(
            r#"# Portal Desktop Project Profile
# Auto-generated for project: {}

# Source the appropriate shell integration
if [[ -n "$BASH_VERSION" ]]; then
    source "{}/.portal_bash_integration"
elif [[ -n "$ZSH_VERSION" ]]; then
    source "{}/.portal_zsh_integration"
elif [[ -n "$FISH_VERSION" ]]; then
    source "{}/.portal_fish_integration"
fi

# Project-specific aliases and functions
alias portal-status="echo '🌍 Portal: Project environment active'"
alias portal-info="cat '{}/.portal_env' 2>/dev/null || echo 'No project environment configured'"

# Function to manually activate project environment
portal-activate() {{
    source "{}/.portal_env" 2>/dev/null && echo "🌍 Portal: Project environment activated" || echo "❌ Portal: No project environment found"
}}

# Function to deactivate project environment
portal-deactivate() {{
    unset NVM_DIR PYENV_VERSION RBENV_VERSION PHPENV_VERSION RUSTUP_TOOLCHAIN
    echo "🌍 Portal: Project environment deactivated"
}}
"#,
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display(),
            project_path.display()
        );

        let profile_path = project_path.join(".portal_profile");
        fs::write(&profile_path, profile_content)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create project profile: {}", e)))?;

        println!("[TerminalIntegration] Created project shell profile: {}", profile_path.display());
        Ok(())
    }

    /// Get terminal integration status for a project
    pub async fn get_integration_status(&self, project_path: &str) -> Result<HashMap<String, bool>, SDKError> {
        let project_path = Path::new(project_path);
        let mut status = HashMap::new();

        status.insert("has_environment".to_string(), self.has_project_environment(project_path).await?);
        status.insert("has_bash_integration".to_string(), project_path.join(".portal_bash_integration").exists());
        status.insert("has_zsh_integration".to_string(), project_path.join(".portal_zsh_integration").exists());
        status.insert("has_fish_integration".to_string(), project_path.join(".portal_fish_integration").exists());
        status.insert("has_profile".to_string(), project_path.join(".portal_profile").exists());

        Ok(status)
    }

    /// Remove terminal integration from a project
    pub async fn remove_integration(&self, project_path: &str) -> Result<(), SDKError> {
        let project_path = Path::new(project_path);
        let files_to_remove = vec![
            ".portal_env",
            ".portal_bash_integration", 
            ".portal_zsh_integration",
            ".portal_fish_integration",
            ".portal_profile"
        ];

        for file in files_to_remove {
            let file_path = project_path.join(file);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .map_err(|e| SDKError::CommandFailed(format!("Failed to remove {}: {}", file, e)))?;
                println!("[TerminalIntegration] Removed {}", file_path.display());
            }
        }

        Ok(())
    }
}

use std::collections::HashMap;
