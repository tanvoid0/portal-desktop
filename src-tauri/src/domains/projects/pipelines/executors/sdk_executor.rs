use std::process::Command;
use std::time::Instant;
use crate::domains::projects::pipelines::executors::{Executor, ExecutionRequest, ExecutionResult};

pub struct SDKExecutor {
    pub sdk_type: String,
    pub sdk_version: Option<String>,
}

impl SDKExecutor {
    pub fn new(sdk_type: String, sdk_version: Option<String>) -> Self {
        Self {
            sdk_type,
            sdk_version,
        }
    }

    fn get_sdk_command(&self) -> Result<String, String> {
        match self.sdk_type.as_str() {
            "node" | "nodejs" => {
                if let Some(version) = &self.sdk_version {
                    Ok(format!("node{}", version))
                } else {
                    Ok("node".to_string())
                }
            }
            "python" => {
                if let Some(version) = &self.sdk_version {
                    Ok(format!("python{}", version))
                } else {
                    Ok("python3".to_string())
                }
            }
            "rust" => Ok("cargo".to_string()),
            "go" => Ok("go".to_string()),
            _ => Err(format!("Unsupported SDK type: {}", self.sdk_type)),
        }
    }
}

impl Executor for SDKExecutor {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, String> {
        let start_time = Instant::now();

        // Parse command into program and args
        let parts: Vec<&str> = request.command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        let program = if parts[0].starts_with("./") || parts[0].contains('/') {
            // Use command as-is if it's a path
            parts[0].to_string()
        } else {
            // For SDK commands, use the SDK command
            self.get_sdk_command()?
        };

        let args = if parts[0].starts_with("./") || parts[0].contains('/') {
            parts[1..].to_vec()
        } else {
            // For SDK commands, prepend SDK-specific args
            match self.sdk_type.as_str() {
                "node" | "nodejs" => {
                    // For node, if command starts with npm/yarn/pnpm, use it directly
                    if parts[0] == "npm" || parts[0] == "yarn" || parts[0] == "pnpm" {
                        parts.to_vec()
                    } else {
                        vec!["-e", &request.command]
                    }
                }
                "python" => {
                    if parts[0] == "pip" || parts[0] == "python" || parts[0] == "python3" {
                        parts.to_vec()
                    } else {
                        vec!["-c", &request.command]
                    }
                }
                "rust" => {
                    // Cargo commands
                    parts.to_vec()
                }
                "go" => {
                    // Go commands
                    parts.to_vec()
                }
                _ => parts.to_vec(),
            }
        };

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        cmd.current_dir(&request.working_directory);
        cmd.envs(&request.environment);

        // Note: std::process::Command doesn't support timeout directly
        // Timeout would need to be handled at a higher level or using a different approach

        let output = cmd.output().map_err(|e| format!("Failed to execute command: {}", e))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = exit_code == Some(0);
        let error = if !success && !stderr.is_empty() {
            Some(stderr)
        } else {
            None
        };

        Ok(ExecutionResult {
            success,
            exit_code,
            output: stdout,
            error,
            duration_ms,
        })
    }

    fn can_execute(&self, execution_type: &str) -> bool {
        execution_type == "command" || execution_type == "script"
    }
}

