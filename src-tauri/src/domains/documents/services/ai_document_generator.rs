use crate::domains::ai::providers::{GenerationOptions, ProviderType};
use crate::domains::ai::services::AIService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Generated document structure from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDocumentStructure {
    pub title: String,
    pub content: String,
    pub suggested_tags: Vec<String>,
    pub confidence: f64,
    pub model_used: String,
}

/// Document context for AI generation (similar to TaskContext)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    pub linked_task: Option<LinkedTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedTask {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

/// Document generator that uses AI to create documents from prompts/descriptions
pub struct AIDocumentGenerator {
    ai_service: Arc<AIService>,
}

impl AIDocumentGenerator {
    pub fn new(ai_service: Arc<AIService>) -> Self {
        Self { ai_service }
    }

    /// Generate document from prompt/description text
    pub async fn generate_document_from_prompt(
        &self,
        prompt: &str,
        provider_type: Option<ProviderType>,
        history: Option<Vec<(String, String)>>, // (role, content) pairs
        context: Option<&DocumentContext>,
        instruction: Option<&str>,
    ) -> Result<GeneratedDocumentStructure, String> {
        // Build the prompt for AI generation
        let mut user_prompt = Self::build_generation_prompt(prompt, context, instruction);

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
            user_prompt = format!("{}\n\n{}", history_prompt, user_prompt);
        }

        // Generate with system message for better structure
        let system_message = Self::build_system_message();
        let options = GenerationOptions {
            temperature: Some(0.7),
            max_tokens: Some(8000), // Documents can be longer than tasks
            timeout_ms: Some(120000),
            model: None,
            extra_options: None,
        };

        let result = self
            .ai_service
            .generate_with_system(&system_message, &user_prompt, Some(options), provider_type)
            .await
            .map_err(|e| {
                format!(
                    "AI generation failed: {}. Please check your AI provider configuration.",
                    e
                )
            })?;

        // Parse the AI response
        let generated = Self::parse_ai_response(&result.content, prompt)?;

        Ok(generated)
    }

    /// Build system message for AI
    fn build_system_message() -> String {
        r#"You are a document writing assistant that creates well-structured, professional documents in Markdown format.

Your response MUST be valid JSON in this exact format:
{
  "title": "Clear and descriptive document title",
  "content": "Full markdown content of the document. Use proper markdown formatting with headers, lists, code blocks, etc.",
  "suggested_tags": ["tag1", "tag2"],
  "confidence": 0.9,
  "model_used": "model-name"
}

Guidelines:
- Use clear, professional language
- Structure content with proper markdown headers (##, ###, etc.)
- Include relevant sections like Overview, Details, Examples, etc.
- Use code blocks for technical content when appropriate
- Make content comprehensive but concise
- Suggest relevant tags for categorization
- Ensure the content is well-formatted and readable"#.to_string()
    }

    /// Build prompt from user input with optional context and instructions
    fn build_generation_prompt(
        prompt: &str,
        context: Option<&DocumentContext>,
        instruction: Option<&str>,
    ) -> String {
        let mut full_prompt = String::new();

        // Add context information if available
        if let Some(ref ctx) = context {
            if let Some(ref linked_task) = ctx.linked_task {
                full_prompt.push_str("=== LINKED TASK CONTEXT ===\n");
                full_prompt.push_str(&format!("Task: {}\n", linked_task.title));
                if let Some(ref desc) = linked_task.description {
                    full_prompt.push_str(&format!("Task Description: {}\n", desc));
                }
                if let Some(ref status) = linked_task.status {
                    full_prompt.push_str(&format!("Task Status: {}\n", status));
                }
                full_prompt.push_str("\nThis document is linked to the above task. ");
                full_prompt.push_str("The document should provide relevant information, documentation, or notes related to this task.\n\n");
            }
        }

        // Add generation instructions if provided
        if let Some(inst) = instruction {
            if !inst.trim().is_empty() {
                full_prompt.push_str("=== GENERATION INSTRUCTIONS ===\n");
                full_prompt.push_str(&format!("{}\n\n", inst));
                full_prompt.push_str("Please follow these instructions when generating the document.\n\n");
            }
        }

        full_prompt.push_str("Create a well-structured document based on the following prompt:\n\n");
        full_prompt.push_str(prompt);
        full_prompt.push_str("\n\nGenerate a comprehensive document with proper markdown formatting.");

        full_prompt
    }

    /// Parse AI response into GeneratedDocumentStructure
    fn parse_ai_response(
        response: &str,
        original_prompt: &str,
    ) -> Result<GeneratedDocumentStructure, String> {
        // Try to extract JSON from response (might have markdown code blocks)
        let json_str = Self::extract_json_from_response(response);

        // Parse into structure
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse AI response as JSON: {}. Response: {}", e, response))?;

        let title = value.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Fallback: use first line of prompt as title
                original_prompt.lines().next().unwrap_or("Untitled Document").to_string()
            });

        let content = value.get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Fallback: use the response as content (might be plain text)
                response.to_string()
            });

        let suggested_tags = value.get("suggested_tags")
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

        Ok(GeneratedDocumentStructure {
            title,
            content,
            suggested_tags,
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
}

