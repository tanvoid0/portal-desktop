use std::process::Command;
use std::time::Instant;
use crate::domains::projects::pipelines::executors::{Executor, ExecutionRequest, ExecutionResult};

pub struct DockerExecutor {
    pub image: String,
    pub dockerfile: Option<String>,
    pub context: Option<String>,
}

impl DockerExecutor {
    pub fn new(image: String, dockerfile: Option<String>, context: Option<String>) -> Self {
        Self {
            image,
            dockerfile,
            context,
        }
    }
}

impl Executor for DockerExecutor {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, String> {
        let start_time = Instant::now();

        // Build Docker command
        let mut cmd = Command::new("docker");
        
        // If we need to build first
        if let Some(dockerfile) = &self.dockerfile {
            cmd.arg("build");
            cmd.arg("-t").arg(&self.image);
            cmd.arg("-f").arg(dockerfile);
            if let Some(context) = &self.context {
                cmd.arg(context);
            } else {
                cmd.arg(&request.working_directory);
            }

            let build_output = cmd.output()
                .map_err(|e| format!("Failed to build Docker image: {}", e))?;

            if !build_output.status.success() {
                let error = String::from_utf8_lossy(&build_output.stderr).to_string();
                return Ok(ExecutionResult {
                    success: false,
                    exit_code: build_output.status.code(),
                    output: String::from_utf8_lossy(&build_output.stdout).to_string(),
                    error: Some(error),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }

        // Run the command in the container
        let mut run_cmd = Command::new("docker");
        run_cmd.arg("run");
        run_cmd.arg("--rm");
        
        // Set working directory
        run_cmd.arg("-w").arg(&request.working_directory);
        
        // Set environment variables
        for (key, value) in &request.environment {
            run_cmd.arg("-e").arg(format!("{}={}", key, value));
        }
        
        // Add the image
        run_cmd.arg(&self.image);
        
        // Add the command
        run_cmd.arg("sh").arg("-c").arg(&request.command);

        let output = run_cmd.output()
            .map_err(|e| format!("Failed to execute Docker command: {}", e))?;

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
        execution_type == "docker"
    }
}

