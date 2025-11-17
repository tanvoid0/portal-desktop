use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::database::DatabaseManager;
use crate::domains::documents::services::document_service::DocumentService;
use crate::domains::documents::services::ai_document_generator::{AIDocumentGenerator, GeneratedDocumentStructure, DocumentContext};
use crate::domains::ai::providers::ProviderType;
use crate::domains::ai::services::AIService;
use crate::domains::documents::repositories::document_repository::{CreateDocumentRequest, UpdateDocumentRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub is_archived: bool,
    pub content_draft: Option<String>,
    pub is_draft: bool,
    pub tags: Option<String>, // JSON array of strings
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_edited_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<crate::entities::document::Model> for DocumentResponse {
    fn from(model: crate::entities::document::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            content: model.content,
            is_archived: model.is_archived,
            content_draft: model.content_draft,
            is_draft: model.is_draft,
            tags: model.tags,
            created_at: model.created_at.map(|dt| dt.into()),
            updated_at: model.updated_at.map(|dt| dt.into()),
            last_edited_at: model.last_edited_at.map(|dt| dt.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentCommand {
    pub title: String,
    pub content: String,
    pub is_archived: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentCommand {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_archived: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentCommand {
    pub prompt: String,
    pub provider_type: Option<ProviderType>,
    pub history: Option<Vec<ConversationMessage>>,
    pub context: Option<DocumentContext>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[tauri::command]
pub async fn create_document(
    db_manager: State<'_, DatabaseManager>,
    command: CreateDocumentCommand,
) -> Result<DocumentResponse, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    let request = CreateDocumentRequest {
        title: command.title,
        content: command.content,
        is_archived: command.is_archived,
        tags: command.tags,
    };

    service
        .create_document(request)
        .await
        .map(|doc| doc.into())
        .map_err(|e| format!("Failed to create document: {}", e))
}

#[tauri::command]
pub async fn get_document(
    db_manager: State<'_, DatabaseManager>,
    id: i32,
) -> Result<Option<DocumentResponse>, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .get_document(id)
        .await
        .map(|opt| opt.map(|doc| doc.into()))
        .map_err(|e| format!("Failed to get document: {}", e))
}

#[tauri::command]
pub async fn get_documents(
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<DocumentResponse>, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .get_documents()
        .await
        .map(|docs| docs.into_iter().map(|doc| doc.into()).collect())
        .map_err(|e| format!("Failed to get documents: {}", e))
}

#[tauri::command]
pub async fn update_document(
    db_manager: State<'_, DatabaseManager>,
    id: i32,
    command: UpdateDocumentCommand,
) -> Result<DocumentResponse, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    let request = UpdateDocumentRequest {
        title: command.title,
        content: command.content,
        is_archived: command.is_archived,
        tags: command.tags,
    };

    service
        .update_document(id, request)
        .await
        .map(|doc| doc.into())
        .map_err(|e| format!("Failed to update document: {}", e))
}

#[tauri::command]
pub async fn update_document_draft(
    db_manager: State<'_, DatabaseManager>,
    id: i32,
    content_draft: String,
) -> Result<DocumentResponse, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .update_draft(id, content_draft)
        .await
        .map(|doc| doc.into())
        .map_err(|e| format!("Failed to update document draft: {}", e))
}

#[tauri::command]
pub async fn save_document(
    db_manager: State<'_, DatabaseManager>,
    id: i32,
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
    is_archived: Option<bool>,
) -> Result<DocumentResponse, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .save_document(id, title, content, tags, is_archived)
        .await
        .map(|doc| doc.into())
        .map_err(|e| format!("Failed to save document: {}", e))
}

#[tauri::command]
pub async fn delete_document(
    db_manager: State<'_, DatabaseManager>,
    id: i32,
) -> Result<(), String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .delete_document(id)
        .await
        .map_err(|e| format!("Failed to delete document: {}", e))
}

#[tauri::command]
pub async fn search_documents(
    db_manager: State<'_, DatabaseManager>,
    query: String,
) -> Result<Vec<DocumentResponse>, String> {
    let conn = db_manager.get_connection_clone();
    let service = DocumentService::new(conn);
    
    service
        .search_documents(&query)
        .await
        .map(|docs| docs.into_iter().map(|doc| doc.into()).collect())
        .map_err(|e| format!("Failed to search documents: {}", e))
}

#[tauri::command]
pub async fn generate_document_with_ai(
    ai_service: State<'_, Arc<AIService>>,
    command: GenerateDocumentCommand,
) -> Result<GeneratedDocumentStructure, String> {
    // Validation
    if command.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }

    if command.prompt.len() > 20000 {
        return Err("Prompt is too long (max 20000 characters)".to_string());
    }

    // Create AI document generator
    let generator = AIDocumentGenerator::new(ai_service.inner().clone());

    // Convert history to (role, content) pairs
    let history = command.history.map(|hist| {
        hist.into_iter()
            .map(|msg| (msg.role, msg.content))
            .collect()
    });

    // Generate document
    generator
        .generate_document_from_prompt(
            &command.prompt,
            command.provider_type,
            history,
            command.context.as_ref(),
            command.instruction.as_deref(),
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to generate document: {}", e);
            format!("Failed to generate document: {}", e)
        })
}

