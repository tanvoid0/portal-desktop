/**
 * Unified Command Execution Utility
 * 
 * This module provides a cross-platform command execution utility that handles
 * different operating systems and shell environments consistently.
 */

use std::process::Command;
use std::collections::HashMap;
use std::path::Path;

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

/// Command execution options
#[derive(Debug, Clone)]
pub struct CommandOptions {
    pub working_directory: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<u64>,
    pub shell: Option<ShellType>,
}

/// Shell type for command execution
#[derive(Debug, Clone)]
pub enum ShellType {
    Bash,
    Sh,
    Cmd, // Windows
    PowerShell, // Windows
    Zsh,
    Fish,
}

/// Unified command executor
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command with options
    pub async fn execute(
        command: &str,
        options: Option<CommandOptions>,
    ) -> Result<CommandResult, String> {
        let opts = options.unwrap_or_default();
        
        // Determine shell based on OS and options
        let (shell_cmd, shell_args) = Self::get_shell_command(&opts);
        
        // Build command
        let mut cmd = Command::new(&shell_cmd);
        
        // Add shell arguments
        for arg in shell_args {
            cmd.arg(arg);
        }
        
        // Add the actual command
        cmd.arg(command);
        
        // Set working directory
        if let Some(working_dir) = &opts.working_directory {
            cmd.current_dir(Path::new(working_dir));
        }
        
        // Set environment variables
        if let Some(env_vars) = &opts.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute command
        let output = cmd.output().map_err(|e| {
            format!("Failed to execute command '{}': {}", command, e)
        })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code();
        
        Ok(CommandResult {
            stdout,
            stderr,
            success,
            exit_code,
        })
    }
    
    /// Execute a command with arguments (not shell-based)
    pub async fn execute_with_args(
        command: &str,
        args: &[&str],
        options: Option<CommandOptions>,
    ) -> Result<CommandResult, String> {
        let opts = options.unwrap_or_default();
        
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        // Set working directory
        if let Some(working_dir) = &opts.working_directory {
            cmd.current_dir(Path::new(working_dir));
        }
        
        // Set environment variables
        if let Some(env_vars) = &opts.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute command
        let output = cmd.output().map_err(|e| {
            format!("Failed to execute command '{}' with args {:?}: {}", command, args, e)
        })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code();
        
        Ok(CommandResult {
            stdout,
            stderr,
            success,
            exit_code,
        })
    }
    
    /// Execute a shell command (cross-platform)
    pub async fn execute_shell(
        command: &str,
        options: Option<CommandOptions>,
    ) -> Result<CommandResult, String> {
        let opts = options.unwrap_or_default();
        
        // Determine the shell to use
        let shell_cmd = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "bash"
        };
        
        let shell_args = if cfg!(target_os = "windows") {
            vec!["/C", command]
        } else {
            vec!["-c", command]
        };
        
        let mut cmd = Command::new(shell_cmd);
        cmd.args(&shell_args);
        
        // Set working directory
        if let Some(working_dir) = &opts.working_directory {
            cmd.current_dir(Path::new(working_dir));
        }
        
        // Set environment variables
        if let Some(env_vars) = &opts.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Execute command
        let output = cmd.output().map_err(|e| {
            format!("Failed to execute shell command '{}': {}", command, e)
        })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code();
        
        Ok(CommandResult {
            stdout,
            stderr,
            success,
            exit_code,
        })
    }
    
    /// Check if a command exists in PATH
    pub async fn command_exists(command: &str) -> bool {
        let result = Self::execute_with_args(command, &["--version"], None).await;
        result.is_ok() && result.unwrap().success
    }
    
    /// Get the appropriate shell command for the current OS
    fn get_shell_command(options: &CommandOptions) -> (String, Vec<String>) {
        // If shell type is explicitly specified, use it
        if let Some(shell_type) = &options.shell {
            return match shell_type {
                ShellType::Bash => ("bash".to_string(), vec!["-c".to_string()]),
                ShellType::Sh => ("sh".to_string(), vec!["-c".to_string()]),
                ShellType::Cmd => ("cmd".to_string(), vec!["/C".to_string()]),
                ShellType::PowerShell => ("powershell".to_string(), vec!["-Command".to_string()]),
                ShellType::Zsh => ("zsh".to_string(), vec!["-c".to_string()]),
                ShellType::Fish => ("fish".to_string(), vec!["-c".to_string()]),
            };
        }
        
        // Auto-detect based on OS
        if cfg!(target_os = "windows") {
            ("cmd".to_string(), vec!["/C".to_string()])
        } else {
            // Try bash first, fallback to sh
            if Self::shell_exists("bash") {
                ("bash".to_string(), vec!["-c".to_string()])
            } else {
                ("sh".to_string(), vec!["-c".to_string()])
            }
        }
    }
    
    /// Check if a shell exists
    fn shell_exists(shell: &str) -> bool {
        Command::new(shell)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            working_directory: None,
            environment: None,
            timeout_seconds: None,
            shell: None,
        }
    }
}

/// Convenience functions for common use cases
impl CommandExecutor {
    /// Execute a simple command without options
    pub async fn simple_execute(command: &str) -> Result<String, String> {
        let result = Self::execute(command, None).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(format!("Command failed: {}", result.stderr))
        }
    }
    
    /// Execute a command in a specific directory
    pub async fn execute_in_directory(
        command: &str,
        working_directory: &str,
    ) -> Result<String, String> {
        let options = CommandOptions {
            working_directory: Some(working_directory.to_string()),
            ..Default::default()
        };
        
        let result = Self::execute(command, Some(options)).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(format!("Command failed: {}", result.stderr))
        }
    }
    
    /// Execute a command with environment variables
    pub async fn execute_with_env(
        command: &str,
        environment: HashMap<String, String>,
    ) -> Result<String, String> {
        let options = CommandOptions {
            environment: Some(environment),
            ..Default::default()
        };
        
        let result = Self::execute(command, Some(options)).await?;
        if result.success {
            Ok(result.stdout)
        } else {
            Err(format!("Command failed: {}", result.stderr))
        }
    }
}
