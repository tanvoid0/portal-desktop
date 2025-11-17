use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

/// Workflow step types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowStep {
    /// Execute a shell command
    Command {
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
        env: Option<HashMap<String, String>>,
    },
    /// Create a file with content
    CreateFile {
        path: String,
        content: String,
    },
    /// Conditional step execution
    Condition {
        condition: String, // Expression to evaluate
        then: Vec<WorkflowStep>,
        else_: Option<Vec<WorkflowStep>>,
    },
    /// Pattern matching trigger
    PatternMatch {
        pattern: String,
        on_match: Vec<WorkflowStep>,
        on_no_match: Option<Vec<WorkflowStep>>,
    },
    /// Wait/delay step
    Wait {
        seconds: u64,
    },
}

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub triggers: Vec<WorkflowTrigger>,
    pub steps: Vec<WorkflowStep>,
    pub enabled: bool,
}

/// Workflow trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowTrigger {
    /// Trigger on command pattern
    CommandPattern { pattern: String },
    /// Trigger on file pattern
    FilePattern { pattern: String },
    /// Trigger on project type
    ProjectType { project_type: String },
    /// Manual trigger
    Manual,
    /// Trigger on event
    Event { event_type: String },
}

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub project_path: Option<PathBuf>,
    pub variables: HashMap<String, String>,
    pub trigger_data: Option<serde_json::Value>,
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub workflow_id: String,
    pub success: bool,
    pub steps_executed: usize,
    pub steps_failed: usize,
    pub output: Vec<StepOutput>,
    pub error: Option<String>,
}

/// Individual step output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub step_index: usize,
    pub step_type: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Lightweight embedded workflow engine
pub struct WorkflowEngine {
    workflows: HashMap<String, Workflow>,
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    /// Register a workflow
    pub fn register_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    /// Load workflows from YAML/JSON
    pub fn load_from_json(&mut self, json: &str) -> Result<(), String> {
        let workflows: Vec<Workflow> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse workflows: {}", e))?;

        for workflow in workflows {
            self.register_workflow(workflow);
        }

        Ok(())
    }

    /// Execute a workflow
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        context: WorkflowContext,
    ) -> Result<WorkflowExecutionResult, String> {
        let workflow = self.workflows.get(workflow_id)
            .ok_or_else(|| format!("Workflow not found: {}", workflow_id))?;

        if !workflow.enabled {
            return Err("Workflow is disabled".to_string());
        }

        let mut result = WorkflowExecutionResult {
            workflow_id: workflow_id.to_string(),
            success: true,
            steps_executed: 0,
            steps_failed: 0,
            output: Vec::new(),
            error: None,
        };

        for (index, step) in workflow.steps.iter().enumerate() {
            let step_result = self.execute_step(step, &context, index).await;
            
            match step_result {
                Ok(output) => {
                    result.steps_executed += 1;
                    result.output.push(output.clone());
                    
                    if !output.success {
                        result.steps_failed += 1;
                        // Optionally stop on first failure
                        // result.success = false;
                        // break;
                    }
                }
                Err(e) => {
                    result.steps_failed += 1;
                    result.success = false;
                    result.output.push(StepOutput {
                        step_index: index,
                        step_type: format!("{:?}", step),
                        success: false,
                        output: None,
                        error: Some(e.clone()),
                        duration_ms: 0,
                    });
                    result.error = Some(format!("Step {} failed: {}", index, e));
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Execute a single workflow step
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &WorkflowContext,
        index: usize,
    ) -> Result<StepOutput, String> {
        let start = std::time::Instant::now();

        match step {
            WorkflowStep::Command { command, args, working_dir, env } => {
                self.execute_command(command, args, working_dir, env, context).await
            }
            WorkflowStep::CreateFile { path, content } => {
                self.create_file(path, content, context).await
            }
            WorkflowStep::Condition { condition, then, else_ } => {
                Box::pin(self.execute_condition(condition, then, else_, context, index)).await
            }
            WorkflowStep::PatternMatch { pattern, on_match, on_no_match } => {
                Box::pin(self.execute_pattern_match(pattern, on_match, on_no_match, context, index)).await
            }
            WorkflowStep::Wait { seconds } => {
                tokio::time::sleep(tokio::time::Duration::from_secs(*seconds)).await;
                Ok(StepOutput {
                    step_index: index,
                    step_type: "wait".to_string(),
                    success: true,
                    output: None,
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                })
            }
        }
    }

    /// Execute a command step
    async fn execute_command(
        &self,
        command: &str,
        args: &[String],
        working_dir: &Option<String>,
        env: &Option<HashMap<String, String>>,
        context: &WorkflowContext,
    ) -> Result<StepOutput, String> {
        let start = std::time::Instant::now();
        
        // Substitute variables in command and args
        let cmd = self.substitute_variables(command, context);
        let cmd_args: Vec<String> = args.iter()
            .map(|a| self.substitute_variables(a, context))
            .collect();

        let mut cmd_builder = Command::new(&cmd);
        cmd_builder.args(&cmd_args);

        // Set working directory
        if let Some(wd) = working_dir {
            let wd_path = self.substitute_variables(wd, context);
            if let Some(project_path) = &context.project_path {
                cmd_builder.current_dir(project_path.join(wd_path));
            } else {
                cmd_builder.current_dir(wd_path);
            }
        } else if let Some(project_path) = &context.project_path {
            cmd_builder.current_dir(project_path);
        }

        // Set environment variables
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd_builder.env(key, self.substitute_variables(value, context));
            }
        }

        // Add context variables to environment
        for (key, value) in &context.variables {
            cmd_builder.env(format!("WF_{}", key), value);
        }

        let output = cmd_builder.output().await
            .map_err(|e| format!("Command execution failed: {}", e))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let error_output = if !success { Some(stderr.clone()) } else { None };
        let output_text = if !stdout.is_empty() { stdout } else { stderr };

        Ok(StepOutput {
            step_index: 0, // Will be set by caller
            step_type: "command".to_string(),
            success,
            output: Some(output_text),
            error: error_output,
            duration_ms,
        })
    }

    /// Create a file step
    async fn create_file(
        &self,
        path: &str,
        content: &str,
        context: &WorkflowContext,
    ) -> Result<StepOutput, String> {
        let start = std::time::Instant::now();
        
        let file_path = self.substitute_variables(path, context);
        let file_content = self.substitute_variables(content, context);

        let final_path = if let Some(project_path) = &context.project_path {
            project_path.join(&file_path)
        } else {
            PathBuf::from(&file_path)
        };

        // Create parent directories if needed
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        tokio::fs::write(&final_path, file_content).await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(StepOutput {
            step_index: 0,
            step_type: "create_file".to_string(),
            success: true,
            output: Some(format!("File created: {}", final_path.display())),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute conditional step
    fn execute_condition(
        &self,
        condition: &str,
        then: &[WorkflowStep],
        else_: &Option<Vec<WorkflowStep>>,
        context: &WorkflowContext,
        index: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<StepOutput, String>> + Send + '_>> {
        let condition = condition.to_string();
        let then = then.to_vec();
        let else_ = else_.clone();
        let context = context.clone();
        let self_ref = self;
        Box::pin(async move {
            // Simple condition evaluation (can be enhanced)
            let condition_result = self_ref.evaluate_condition(&condition, &context);

            let steps_to_execute = if condition_result {
                &then
            } else {
                else_.as_deref().unwrap_or(&[])
            };

            // Execute steps in the branch
            for (i, step) in steps_to_execute.iter().enumerate() {
                self_ref.execute_step(step, &context, index + i + 1).await?;
            }

            Ok(StepOutput {
                step_index: index,
                step_type: "condition".to_string(),
                success: true,
                output: Some(format!("Condition evaluated: {}", condition_result)),
                error: None,
                duration_ms: 0,
            })
        })
    }

    /// Execute pattern match step
    fn execute_pattern_match(
        &self,
        pattern: &str,
        on_match: &[WorkflowStep],
        on_no_match: &Option<Vec<WorkflowStep>>,
        context: &WorkflowContext,
        index: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<StepOutput, String>> + Send + '_>> {
        let pattern = pattern.to_string();
        let on_match = on_match.to_vec();
        let on_no_match = on_no_match.clone();
        let context = context.clone();
        let self_ref = self;
        Box::pin(async move {
            // Simple pattern matching (can be enhanced with regex)
            let matches = context.variables.iter()
                .any(|(k, v)| k.contains(&pattern) || v.contains(&pattern));

            let steps_to_execute = if matches {
                &on_match
            } else {
                on_no_match.as_deref().unwrap_or(&[])
            };

            for (i, step) in steps_to_execute.iter().enumerate() {
                self_ref.execute_step(step, &context, index + i + 1).await?;
            }

            Ok(StepOutput {
                step_index: index,
                step_type: "pattern_match".to_string(),
                success: true,
                output: Some(format!("Pattern match: {}", matches)),
                error: None,
                duration_ms: 0,
            })
        })
    }

    /// Substitute variables in a string
    fn substitute_variables(&self, text: &str, context: &WorkflowContext) -> String {
        let mut result = text.to_string();
        
        for (key, value) in &context.variables {
            result = result.replace(&format!("${{{}}}", key), value);
            result = result.replace(&format!("${}", key), value);
        }

        // Common variables
        if let Some(project_path) = &context.project_path {
            result = result.replace("${PROJECT_PATH}", &project_path.to_string_lossy());
        }

        result
    }

    /// Evaluate a condition (simple boolean logic)
    fn evaluate_condition(&self, condition: &str, context: &WorkflowContext) -> bool {
        // Simple evaluation - check if variable exists or equals value
        // Format: "${VAR}" or "${VAR} == value"
        let parts: Vec<&str> = condition.split("==").map(|s| s.trim()).collect();
        
        if parts.len() == 2 {
            let var_name = parts[0].trim_matches(|c| c == '$' || c == '{' || c == '}');
            let expected = parts[1].trim_matches('"').trim_matches('\'');
            return context.variables.get(var_name)
                .map(|v| v == expected)
                .unwrap_or(false);
        }

        // Check if variable exists
        let var_name = condition.trim_matches(|c| c == '$' || c == '{' || c == '}');
        context.variables.contains_key(var_name)
    }

    /// Get all registered workflows
    pub fn get_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Check if a workflow should trigger based on context
    pub fn should_trigger(&self, workflow_id: &str, trigger_data: &serde_json::Value) -> bool {
        let workflow = match self.workflows.get(workflow_id) {
            Some(w) => w,
            None => return false,
        };

        for trigger in &workflow.triggers {
            if self.matches_trigger(trigger, trigger_data) {
                return true;
            }
        }

        false
    }

    /// Check if a trigger matches the context
    fn matches_trigger(&self, trigger: &WorkflowTrigger, data: &serde_json::Value) -> bool {
        match trigger {
            WorkflowTrigger::CommandPattern { pattern } => {
                data.get("command")
                    .and_then(|v| v.as_str())
                    .map(|cmd| cmd.contains(pattern))
                    .unwrap_or(false)
            }
            WorkflowTrigger::FilePattern { pattern } => {
                data.get("file")
                    .and_then(|v| v.as_str())
                    .map(|file| file.contains(pattern))
                    .unwrap_or(false)
            }
            WorkflowTrigger::ProjectType { project_type } => {
                data.get("project_type")
                    .and_then(|v| v.as_str())
                    .map(|pt| pt == project_type)
                    .unwrap_or(false)
            }
            WorkflowTrigger::Event { event_type } => {
                data.get("event_type")
                    .and_then(|v| v.as_str())
                    .map(|et| et == event_type)
                    .unwrap_or(false)
            }
            WorkflowTrigger::Manual => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_workflow() {
        let mut engine = WorkflowEngine::new();
        
        let workflow = Workflow {
            id: "test".to_string(),
            name: "Test Workflow".to_string(),
            description: None,
            triggers: vec![WorkflowTrigger::Manual],
            steps: vec![
                WorkflowStep::Wait { seconds: 0 },
            ],
            enabled: true,
        };

        engine.register_workflow(workflow);

        let context = WorkflowContext {
            project_path: None,
            variables: HashMap::new(),
            trigger_data: None,
        };

        let result = engine.execute_workflow("test", context).await.unwrap();
        assert!(result.success);
    }
}
