use crate::domains::ai::providers::{GenerationOptions, ProviderType};
use crate::domains::ai::services::AIService;
use crate::domains::tasks::services::story_parser::{StoryParser, ParsedStory};
use crate::domains::tasks::commands::TaskContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Generated task structure from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTaskStructure {
    /// Main task
    pub main_task: GeneratedTask,
    /// Subtasks
    pub subtasks: Vec<GeneratedSubtask>,
    /// Suggested project link
    pub suggested_project: Option<ProjectSuggestion>,
    /// Suggested labels/tags
    pub suggested_labels: Vec<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Model used for generation
    pub model_used: String,
}

/// Generated main task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTask {
    pub title: String,
    pub description: String,
    pub priority: String, // low, medium, high
    pub type_: String,    // Story, Bug, Feature, etc.
    pub estimated_time: Option<u32>, // minutes
    pub tags: Vec<String>,
}

/// Generated subtask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSubtask {
    pub title: String,
    pub description: String,
    pub estimated_time: Option<u32>, // minutes
    pub dependencies: Vec<usize>,    // indices of other subtasks this depends on
    pub order: usize,                 // suggested order
}

/// Intermediate structure for parsing subtasks with flexible dependencies
#[derive(Debug, Clone, Deserialize)]
struct IntermediateSubtask {
    pub title: String,
    pub description: String,
    pub estimated_time: Option<u32>,
    #[serde(deserialize_with = "deserialize_flexible_dependencies")]
    pub dependencies: Vec<DependencyRef>,
    pub order: usize,
}

/// Dependency reference that can be either a string (title) or usize (index)
#[derive(Debug, Clone)]
enum DependencyRef {
    Index(usize),
    Title(String),
}

/// Custom deserializer for dependencies that accepts either strings (task titles) or usize (indices)
fn deserialize_flexible_dependencies<'de, D>(deserializer: D) -> Result<Vec<DependencyRef>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct DependenciesVisitor;

    impl<'de> Visitor<'de> for DependenciesVisitor {
        type Value = Vec<DependencyRef>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a vector of strings (task titles) or numbers (indices)")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut deps = Vec::new();
            while let Some(value) = seq.next_element::<serde_json::Value>()? {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(idx) = n.as_u64() {
                            deps.push(DependencyRef::Index(idx as usize));
                        }
                    }
                    serde_json::Value::String(s) => {
                        deps.push(DependencyRef::Title(s));
                    }
                    _ => {}
                }
            }
            Ok(deps)
        }
    }

    deserializer.deserialize_seq(DependenciesVisitor)
}

/// Project suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSuggestion {
    pub name: String,
    pub confidence: f64,
    pub reason: String,
}

/// Task generator that uses AI to create tasks from story/description text (any format)
pub struct AITaskGenerator {
    ai_service: Arc<AIService>,
}

impl AITaskGenerator {
    pub fn new(ai_service: Arc<AIService>) -> Self {
        Self { ai_service }
    }

    /// Generate tasks from story/description text (any format)
    pub async fn generate_tasks_from_story(
        &self,
        story_text: &str,
        provider_type: Option<ProviderType>,
        history: Option<Vec<(String, String)>>, // (role, content) pairs
        context: Option<&TaskContext>,
        developer_note: Option<&str>,
        instruction: Option<&str>,
    ) -> Result<GeneratedTaskStructure, String> {
        // First, parse the story text to extract basic information
        let parsed = StoryParser::parse(story_text);

        // Build the prompt for AI generation
        let mut prompt = Self::build_generation_prompt(&parsed, context, developer_note, instruction);

        // If history is provided, prepend it to the prompt
        if let Some(ref hist) = history {
            let mut history_prompt = String::new();
            for (role, content) in hist {
                match role.as_str() {
                    "assistant" => {
                        history_prompt.push_str(&format!("Previous AI Response:\n{}\n\n", content));
                    }
                    "user" => {
                        history_prompt.push_str(&format!("User Instructions:\n{}\n\n", content));
                    }
                    _ => {}
                }
            }
            prompt = format!("{}\n\n{}", history_prompt, prompt);
        }

        // Generate with system message for better structure
        let system_message = Self::build_system_message();
        let options = GenerationOptions {
            temperature: Some(0.7),
            max_tokens: Some(4000),
            timeout_ms: Some(120000),
            model: None,
            extra_options: None,
        };

        let result = self
            .ai_service
            .generate_with_system(&system_message, &prompt, Some(options), provider_type)
            .await
            .map_err(|e| {
                format!(
                    "AI generation failed: {}. Please check your AI provider configuration.",
                    e
                )
            })?;

        // Parse the AI response
        let generated = Self::parse_ai_response(&result.content, &parsed)?;

        Ok(generated)
    }

    /// Build system message for AI
    fn build_system_message() -> String {
        r#"You are a task management assistant that converts story descriptions into well-structured, developer-friendly tasks and subtasks.

Your response MUST be valid JSON in this exact format:
{
  "main_task": {
    "title": "Short, action-oriented title (max 80 chars)",
    "description": "Concise description with clear steps",
    "priority": "low|medium|high",
    "type_": "Story|Bug|Feature|Task",
    "estimated_time": 120,
    "tags": ["backend", "api"]
  },
  "subtasks": [
    {
      "title": "Short subtask title",
      "description": "Brief description",
      "estimated_time": 30,
      "dependencies": [],
      "order": 1
    }
  ],
  "suggested_project": {
    "name": "Project Name",
    "confidence": 0.8,
    "reason": "Why this project"
  },
  "suggested_labels": ["backend", "api"],
  "confidence": 0.9,
  "model_used": "model-name"
}

Guidelines:
- Keep titles concise and action-oriented (use verbs)
- Descriptions should be clear but brief
- Break down work into logical, independent subtasks
- Estimate time realistically (in minutes)
- Suggest relevant tags and project links
- Use developer-friendly terminology
- Avoid jargon and marketing language"#.to_string()
    }

    /// Build prompt from parsed story with optional context, developer note, and instructions
    fn build_generation_prompt(
        parsed: &ParsedStory,
        context: Option<&TaskContext>,
        developer_note: Option<&str>,
        instruction: Option<&str>,
    ) -> String {
        let mut prompt = String::new();

        // Add context information if available
        if let Some(ref ctx) = context {
            // If creating a child task, include parent context
            if let Some(ref parent) = ctx.parent_task {
                prompt.push_str("=== PARENT TASK CONTEXT ===\n");
                prompt.push_str(&format!("Parent Task: {}\n", parent.title));
                if let Some(ref desc) = parent.description {
                    prompt.push_str(&format!("Parent Description: {}\n", desc));
                }
                if let Some(ref priority) = parent.priority {
                    prompt.push_str(&format!("Parent Priority: {}\n", priority));
                }
                if let Some(ref type_) = parent.type_ {
                    prompt.push_str(&format!("Parent Type: {}\n", type_));
                }
                if let Some(ref tags) = parent.tags {
                    if !tags.is_empty() {
                        prompt.push_str(&format!("Parent Tags: {}\n", tags.join(", ")));
                    }
                }
                prompt.push_str("\nYou are generating a SUBTASK for this parent task. ");
                prompt.push_str("The subtask should be a specific, actionable piece of work that contributes to the parent task.\n\n");
                
                // Include existing siblings for reference
                if let Some(ref siblings) = ctx.existing_siblings {
                    if !siblings.is_empty() {
                        prompt.push_str("Existing Sibling Subtasks (for reference, avoid duplication):\n");
                        for (i, sibling) in siblings.iter().enumerate() {
                            prompt.push_str(&format!("{}. {}", i + 1, sibling.title));
                            if let Some(ref desc) = sibling.description {
                                prompt.push_str(&format!(" - {}", desc));
                            }
                            prompt.push_str("\n");
                        }
                        prompt.push_str("\n");
                    }
                }
            }
            
            // If creating a parent task, include existing children context
            if let Some(ref children) = ctx.existing_children {
                if !children.is_empty() {
                    prompt.push_str("=== EXISTING CHILDREN CONTEXT ===\n");
                    prompt.push_str("This task already has the following subtasks (for reference, avoid duplication):\n");
                    for (i, child) in children.iter().enumerate() {
                        prompt.push_str(&format!("{}. {}", i + 1, child.title));
                        if let Some(ref desc) = child.description {
                            prompt.push_str(&format!(" - {}", desc));
                        }
                        if let Some(ref status) = child.status {
                            prompt.push_str(&format!(" [Status: {}]", status));
                        }
                        prompt.push_str("\n");
                    }
                    prompt.push_str("\n");
                }
            }
        }

        // Add developer note if provided
        if let Some(note) = developer_note {
            if !note.trim().is_empty() {
                prompt.push_str("=== DEVELOPER NOTE ===\n");
                prompt.push_str(&format!("{}\n\n", note));
                prompt.push_str("IMPORTANT: Include this developer note in the main task description to guide implementation.\n\n");
            }
        }

        // Add generation instructions if provided
        if let Some(inst) = instruction {
            if !inst.trim().is_empty() {
                prompt.push_str("=== GENERATION INSTRUCTIONS ===\n");
                prompt.push_str(&format!("{}\n\n", inst));
                prompt.push_str("Please follow these instructions when generating tasks.\n\n");
            }
        }

        prompt.push_str("Convert this story/description into developer-friendly tasks:\n\n");

        if let Some(ref title) = parsed.title {
            prompt.push_str(&format!("Title: {}\n", title));
        }

        if let Some(ref description) = parsed.description {
            prompt.push_str(&format!("Description: {}\n", description));
        }

        if !parsed.requirements.is_empty() {
            prompt.push_str("\nRequirements/Acceptance Criteria:\n");
            for (i, requirement) in parsed.requirements.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, requirement));
            }
        }

        if let Some(ref priority) = parsed.priority {
            prompt.push_str(&format!("\nPriority: {}\n", priority));
        }

        if let Some(ref story_type) = parsed.story_type {
            prompt.push_str(&format!("Type: {}\n", story_type));
        }

        if !parsed.labels.is_empty() {
            prompt.push_str(&format!("Labels: {}\n", parsed.labels.join(", ")));
        }

        if let Some(ref project) = parsed.project {
            prompt.push_str(&format!("Project: {}\n", project));
        }

        prompt.push_str("\nGenerate a structured task breakdown with subtasks. Focus on:");
        prompt.push_str("\n- Clear, actionable titles");
        prompt.push_str("\n- Logical task breakdown");
        prompt.push_str("\n- Realistic time estimates");
        prompt.push_str("\n- Developer-friendly language");
        prompt.push_str("\n- Preserve all important information");

        prompt
    }

    /// Parse AI response into GeneratedTaskStructure
    fn parse_ai_response(
        response: &str,
        original_parsed: &ParsedStory,
    ) -> Result<GeneratedTaskStructure, String> {
        // Try to extract JSON from response (might have markdown code blocks)
        let json_str = Self::extract_json_from_response(response);

        // Parse into intermediate structure first
        let intermediate: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse AI response as JSON: {}. Response: {}", e, response))?;

        // Convert intermediate structure to final structure, resolving dependencies
        let parsed = Self::convert_intermediate_to_final(intermediate)?;

        // Validate and enhance the parsed structure
        let enhanced = Self::enhance_generated_structure(parsed, original_parsed);

        Ok(enhanced)
    }

    /// Convert intermediate JSON structure to final GeneratedTaskStructure, resolving string dependencies
    fn convert_intermediate_to_final(value: serde_json::Value) -> Result<GeneratedTaskStructure, String> {
        // Extract subtasks first to build title-to-index mapping
        let subtasks_array = value.get("subtasks")
            .and_then(|v| v.as_array())
            .ok_or("Missing or invalid subtasks array")?;

        // Build title-to-index mapping
        let mut title_to_index = std::collections::HashMap::new();
        for (idx, subtask) in subtasks_array.iter().enumerate() {
            if let Some(title) = subtask.get("title").and_then(|v| v.as_str()) {
                title_to_index.insert(title.to_string(), idx);
            }
        }

        // Parse subtasks with flexible dependencies
        let mut resolved_subtasks = Vec::new();
        for subtask_value in subtasks_array {
            let intermediate: IntermediateSubtask = serde_json::from_value(subtask_value.clone())
                .map_err(|e| format!("Failed to parse subtask: {}", e))?;

            // Resolve dependencies
            let mut resolved_deps = Vec::new();
            for dep_ref in intermediate.dependencies {
                match dep_ref {
                    DependencyRef::Index(idx) => {
                        resolved_deps.push(idx);
                    }
                    DependencyRef::Title(title) => {
                        // Find index by title
                        if let Some(&idx) = title_to_index.get(&title) {
                            resolved_deps.push(idx);
                        } else {
                            // Try partial match (in case titles don't match exactly)
                            for (t, &idx) in &title_to_index {
                                if t.contains(&title) || title.contains(t) {
                                    resolved_deps.push(idx);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            resolved_subtasks.push(GeneratedSubtask {
                title: intermediate.title,
                description: intermediate.description,
                estimated_time: intermediate.estimated_time,
                dependencies: resolved_deps,
                order: intermediate.order,
            });
        }

        // Parse main task
        let main_task_value = value.get("main_task")
            .ok_or("Missing main_task")?;
        let main_task: GeneratedTask = serde_json::from_value(main_task_value.clone())
            .map_err(|e| format!("Failed to parse main_task: {}", e))?;

        // Parse optional fields
        let suggested_project = value.get("suggested_project")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        
        let suggested_labels = value.get("suggested_labels")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
            .unwrap_or_default();

        let confidence = value.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);

        let model_used = value.get("model_used")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(GeneratedTaskStructure {
            main_task,
            subtasks: resolved_subtasks,
            suggested_project,
            suggested_labels,
            confidence,
            model_used,
        })
    }

    /// Extract JSON from AI response (might be wrapped in markdown code blocks)
    fn extract_json_from_response(response: &str) -> String {
        // Try to find JSON in code blocks first
        if let Some(start) = response.find("```json") {
            if let Some(end) = response.rfind("```") {
                return response[start + 7..end].trim().to_string();
            }
        }

        // Try to find JSON in generic code blocks
        if let Some(start) = response.find("```") {
            if let Some(end) = response.rfind("```") {
                let content = response[start + 3..end].trim();
                if content.starts_with('{') {
                    return content.to_string();
                }
            }
        }

        // Try to find JSON object directly
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                return response[start..=end].to_string();
            }
        }

        // Fallback: return as-is (will fail in parsing, but that's okay)
        response.trim().to_string()
    }

    /// Enhance generated structure with original parsed data
    fn enhance_generated_structure(
        mut generated: GeneratedTaskStructure,
        original: &ParsedStory,
    ) -> GeneratedTaskStructure {
        // Use original title if AI didn't generate one or it's too generic
        if generated.main_task.title.is_empty()
            || generated.main_task.title.len() > 100
        {
            if let Some(ref title) = original.title {
                generated.main_task.title = title.clone();
            }
        }

        // Enhance description with original if needed
        if generated.main_task.description.is_empty() {
            if let Some(ref desc) = original.description {
                generated.main_task.description = desc.clone();
            }
        }

        // Use original priority if available and AI didn't set one
        if generated.main_task.priority == "medium" && original.priority.is_some() {
            if let Some(ref priority) = original.priority {
                generated.main_task.priority = priority.to_lowercase();
            }
        }

        // Use original type if available
        if generated.main_task.type_.is_empty() && original.story_type.is_some() {
            if let Some(ref story_type) = original.story_type {
                generated.main_task.type_ = story_type.clone();
            }
        }

        // Merge original labels with AI suggestions
        for label in &original.labels {
            if !generated.suggested_labels.contains(label) {
                generated.suggested_labels.push(label.clone());
            }
        }

        // Use original project if AI didn't suggest one
        if generated.suggested_project.is_none() && original.project.is_some() {
            if let Some(ref project) = original.project {
                generated.suggested_project = Some(ProjectSuggestion {
                    name: project.clone(),
                    confidence: 0.7,
                    reason: "Detected from story text".to_string(),
                });
            }
        }

        generated
    }
}

