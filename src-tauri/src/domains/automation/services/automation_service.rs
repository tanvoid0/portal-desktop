use serde_json::Value;
use crate::domains::automation::entities::*;

pub struct AutomationService {
    base_url: String,
    api_key: Option<String>,
}

impl AutomationService {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self { base_url, api_key }
    }

    pub async fn trigger_workflow(
        &self,
        workflow_id: &str,
        project_data: &Value,
    ) -> Result<WorkflowResult, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/webhook/{}", self.base_url, workflow_id);
        
        let mut request = client.post(&url).json(project_data);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to trigger workflow: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Workflow trigger failed with status: {}", response.status()));
        }

        let result: WorkflowResult = response.json().await
            .map_err(|e| format!("Failed to parse workflow result: {}", e))?;

        Ok(result)
    }

    pub async fn get_workflow_status(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/executions/{}", self.base_url, execution_id);
        
        let mut request = client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to get workflow status: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get workflow status: {}", response.status()));
        }

        let execution: WorkflowExecution = response.json().await
            .map_err(|e| format!("Failed to parse workflow execution: {}", e))?;

        Ok(execution)
    }

    pub async fn list_available_workflows(&self) -> Result<Vec<AvailableWorkflow>, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/workflows", self.base_url);
        
        let mut request = client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to list workflows: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to list workflows: {}", response.status()));
        }

        let workflows: Vec<AvailableWorkflow> = response.json().await
            .map_err(|e| format!("Failed to parse workflows: {}", e))?;

        Ok(workflows)
    }

    pub async fn get_suggested_workflows(
        &self,
        framework: Option<&str>,
        package_manager: Option<&str>,
    ) -> Result<Vec<AvailableWorkflow>, String> {
        let all_workflows = self.list_available_workflows().await?;
        
        let mut suggested = Vec::new();
        
        // Add AI-powered workflows for all projects
        let ai_workflows = vec![
            "ai_code_analysis",
            "ai_suggestions", 
            "ai_code_review"
        ];
        
        for workflow in all_workflows {
            let matches_framework = framework
                .map(|f| workflow.framework.as_ref().map_or(false, |wf| wf.contains(f)))
                .unwrap_or(true);
                
            let matches_package_manager = package_manager
                .map(|pm| workflow.package_manager.as_ref().map_or(false, |wpm| wpm.contains(pm)))
                .unwrap_or(true);
            
            // Always include AI workflows
            let is_ai_workflow = ai_workflows.iter().any(|&ai| 
                workflow.name.to_lowercase().contains(ai)
            );
            
            if (matches_framework && matches_package_manager) || is_ai_workflow {
                suggested.push(workflow);
            }
        }
        
        Ok(suggested)
    }

    pub async fn check_n8n_health(&self) -> Result<bool, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/health", self.base_url);
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to check n8n health: {}", e))?;

        Ok(response.status().is_success())
    }
}
