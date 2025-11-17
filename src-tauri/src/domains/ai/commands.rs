use tauri::State;
use std::sync::Arc;
use crate::domains::ai::providers::{
    AIError, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
};
use crate::domains::ai::services::{AIService, AISettingsService};
use crate::domains::ai::chat;
use crate::domains::ai::conversation::{Conversation, ConversationMessage, ConversationWithMessages};
use crate::domains::ai::logging::{AILog, LogFilters};
use crate::database::DatabaseManager;
use sea_orm::{EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect, QueryOrder};
use crate::domains::ai::entities::{
    ConversationEntity, ConversationActiveModel,
    ConversationMessageEntity, ConversationMessageActiveModel,
    AILogEntity, AILogActiveModel, AILogColumn,
    TrainingDataEntity, TrainingDataActiveModel,
};
use crate::domains::ai::entities::ai_conversation::Column as ConversationColumn;
use crate::domains::ai::entities::ai_conversation_message::Column as ConversationMessageColumn;

/// Get configuration status for a provider
#[tauri::command]
pub async fn get_ai_provider_config_status(
    provider_type: ProviderType,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<ConfigurationStatus, String> {
    ai_service
        .check_provider_configuration(Some(provider_type))
        .await
        .map_err(|e| e.to_string())
}

/// Get all configured AI providers
#[tauri::command]
pub async fn get_ai_providers(
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<Vec<ProviderConfig>, String> {
    settings_service.get_all_providers()
}

/// Get default AI provider
#[tauri::command]
pub async fn get_default_ai_provider(
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<Option<ProviderType>, String> {
    settings_service.get_default_provider()
}

/// Set default AI provider
#[tauri::command]
pub async fn set_default_ai_provider(
    provider_type: ProviderType,
    settings_service: State<'_, Arc<AISettingsService>>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<(), String> {
    settings_service
        .set_default_provider(provider_type.clone())
        .map_err(|e| e.to_string())?;
    
    ai_service
        .set_default_provider(provider_type)
        .await
        .map_err(|e| e.to_string())
}

/// Save provider configuration
#[tauri::command]
pub async fn save_ai_provider_config(
    config: ProviderConfig,
    settings_service: State<'_, Arc<AISettingsService>>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<(), String> {
    settings_service
        .save_provider_config(config.clone())
        .map_err(|e| e.to_string())?;
    
    ai_service
        .update_provider_config(config)
        .await
        .map_err(|e| e.to_string())
}

/// Get provider configuration
#[tauri::command]
pub async fn get_ai_provider_config(
    provider_type: ProviderType,
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<ProviderConfig, String> {
    settings_service.get_provider_config(provider_type)
}

/// Test provider connection
#[tauri::command]
pub async fn test_ai_provider(
    provider_type: ProviderType,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<(), String> {
    ai_service
        .test_provider(provider_type)
        .await
        .map_err(|e| e.to_string())
}

/// Get available models for a provider (installed models)
#[tauri::command]
pub async fn get_ai_provider_models(
    provider_type: Option<ProviderType>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<Vec<String>, String> {
    ai_service
        .get_available_models(provider_type)
        .await
        .map_err(|e| e.to_string())
}

/// Get available Ollama models (downloadable models from library)
/// This returns models available to download, which changes over time
#[tauri::command]
pub async fn get_ai_available_ollama_models(
    ai_service: State<'_, Arc<AIService>>,
) -> Result<std::collections::HashMap<String, Vec<std::collections::HashMap<String, String>>>, String> {
    ai_service
        .get_available_ollama_models()
        .await
        .map_err(|e| e.to_string())
}

/// Generate text using AI
#[tauri::command]
pub async fn generate_ai_text(
    prompt: String,
    options: Option<GenerationOptions>,
    provider_type: Option<ProviderType>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<GenerationResult, String> {
    ai_service
        .generate(&prompt, options, provider_type)
        .await
        .map_err(|e| {
            match e {
                AIError::ConfigurationIncomplete(status) => {
                    format!(
                        "Configuration incomplete. Missing fields: {}. Warnings: {}",
                        status.missing_fields.join(", "),
                        status.warnings.join(", ")
                    )
                }
                _ => e.to_string(),
            }
        })
}

/// Generate text with system message
#[tauri::command]
pub async fn generate_ai_text_with_system(
    system_message: String,
    user_message: String,
    options: Option<GenerationOptions>,
    provider_type: Option<ProviderType>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<GenerationResult, String> {
    ai_service
        .generate_with_system(&system_message, &user_message, options, provider_type)
        .await
        .map_err(|e| {
            match e {
                AIError::ConfigurationIncomplete(status) => {
                    format!(
                        "Configuration incomplete. Missing fields: {}. Warnings: {}",
                        status.missing_fields.join(", "),
                        status.warnings.join(", ")
                    )
                }
                _ => e.to_string(),
            }
        })
}

/// Send a message to AI (chat)
#[tauri::command]
pub async fn ai_send_message(
    message: String,
    history: Vec<chat::ChatMessage>,
    provider: Option<ProviderType>,
    conversation_id: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    model: Option<String>,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<String, String> {
    let request = chat::SendMessageRequest {
        message,
        history,
        provider,
        conversation_id,
        temperature,
        max_tokens,
        model,
    };
    chat::send_message(request, ai_service).await
}

/// Create a new conversation
#[tauri::command]
pub async fn ai_create_conversation(
    title: String,
    provider: ProviderType,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Conversation, String> {
    let conversation = Conversation::new(title, format!("{:?}", provider));
    let db = db_manager.get_connection();
    
    let active_model = ConversationActiveModel {
        id: Set(conversation.id.clone()),
        title: Set(conversation.title.clone()),
        provider: Set(conversation.provider.clone()),
        created_at: Set(conversation.created_at.clone()),
        updated_at: Set(conversation.updated_at.clone()),
    };
    
    // Insert the conversation
    active_model.insert(db)
        .await
        .map_err(|e| format!("Failed to create conversation: {}", e))?;
    
    // For SQLite, we need to query the record after insertion since it doesn't support RETURNING
    let inserted_model = ConversationEntity::find_by_id(&conversation.id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to retrieve created conversation: {}", e))?
        .ok_or_else(|| format!("Failed to find inserted conversation with id: {}", conversation.id))?;
    
    Ok(Conversation::from(inserted_model))
}

/// Save conversation messages
#[tauri::command]
pub async fn ai_save_conversation(
    id: String,
    messages: Vec<ConversationMessage>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    
    // Delete existing messages for this conversation
    ConversationMessageEntity::delete_many()
        .filter(ConversationMessageColumn::ConversationId.eq(&id))
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete existing messages: {}", e))?;
    
    // Insert new messages
    for msg in messages {
        let active_model = ConversationMessageActiveModel {
            id: Set(msg.id),
            conversation_id: Set(msg.conversation_id),
            role: Set(msg.role),
            content: Set(msg.content),
            timestamp: Set(msg.timestamp),
            sequence: Set(msg.sequence),
        };
        
        active_model.insert(db)
            .await
            .map_err(|e| format!("Failed to save message: {}", e))?;
    }
    
    // Update conversation updated_at
    let mut conversation: ConversationActiveModel = ConversationEntity::find_by_id(&id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to find conversation: {}", e))?
        .ok_or_else(|| "Conversation not found".to_string())?
        .into();
    
    conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());
    conversation.update(db)
        .await
        .map_err(|e| format!("Failed to update conversation: {}", e))?;
    
    Ok(())
}

/// Load conversation by ID
#[tauri::command]
pub async fn ai_load_conversation(
    id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<ConversationWithMessages, String> {
    let db = db_manager.get_connection();
    
    let conversation_model = ConversationEntity::find_by_id(&id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to load conversation: {}", e))?
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    let messages_models = ConversationMessageEntity::find()
        .filter(ConversationMessageColumn::ConversationId.eq(&id))
        .order_by_asc(ConversationMessageColumn::Sequence)
        .all(db)
        .await
        .map_err(|e| format!("Failed to load messages: {}", e))?;
    
    Ok(ConversationWithMessages {
        conversation: Conversation::from(conversation_model),
        messages: messages_models.into_iter().map(ConversationMessage::from).collect(),
    })
}

/// List all conversations
#[tauri::command]
pub async fn ai_list_conversations(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Conversation>, String> {
    let db = db_manager.get_connection();
    
    let conversations = ConversationEntity::find()
        .order_by_desc(ConversationColumn::UpdatedAt)
        .all(db)
        .await
        .map_err(|e| format!("Failed to list conversations: {}", e))?;
    
    Ok(conversations.into_iter().map(Conversation::from).collect())
}

/// Delete conversation
#[tauri::command]
pub async fn ai_delete_conversation(
    id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    
    // Messages will be deleted automatically due to CASCADE foreign key
    ConversationEntity::delete_by_id(&id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete conversation: {}", e))?;
    
    Ok(())
}

/// Update conversation title
#[tauri::command]
pub async fn ai_update_conversation_title(
    id: String,
    title: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    
    let mut conversation: ConversationActiveModel = ConversationEntity::find_by_id(&id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to find conversation: {}", e))?
        .ok_or_else(|| "Conversation not found".to_string())?
        .into();
    
    conversation.title = Set(title);
    conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());
    
    conversation.update(db)
        .await
        .map_err(|e| format!("Failed to update conversation title: {}", e))?;
    
    Ok(())
}

/// Get AI logs with filters
#[tauri::command]
pub async fn ai_get_logs(
    filters: LogFilters,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<AILog>, String> {
    let db = db_manager.get_connection();
    
    let mut query = AILogEntity::find();
    
    if let Some(provider) = &filters.provider {
        query = query.filter(AILogColumn::Provider.eq(provider));
    }
    if let Some(log_type) = &filters.log_type {
        query = query.filter(AILogColumn::LogType.eq(log_type));
    }
    if let Some(date_from) = &filters.date_from {
        query = query.filter(AILogColumn::Timestamp.gte(date_from));
    }
    if let Some(date_to) = &filters.date_to {
        query = query.filter(AILogColumn::Timestamp.lte(date_to));
    }
    
    let logs = query
        .order_by_desc(AILogColumn::Timestamp)
        .limit(1000)
        .all(db)
        .await
        .map_err(|e| format!("Failed to get logs: {}", e))?;
    
    Ok(logs.into_iter().map(AILog::from).collect())
}

/// Search AI logs
#[tauri::command]
pub async fn ai_search_logs(
    query: String,
    filters: LogFilters,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<AILog>, String> {
    let db = db_manager.get_connection();
    let search_pattern = format!("%{}%", query);
    
    let mut db_query = AILogEntity::find()
        .filter(
            AILogColumn::RequestData.like(&search_pattern)
                .or(AILogColumn::ResponseData.like(&search_pattern))
                .or(AILogColumn::ErrorMessage.like(&search_pattern))
        );
    
    if let Some(provider) = &filters.provider {
        db_query = db_query.filter(AILogColumn::Provider.eq(provider));
    }
    if let Some(log_type) = &filters.log_type {
        db_query = db_query.filter(AILogColumn::LogType.eq(log_type));
    }
    if let Some(date_from) = &filters.date_from {
        db_query = db_query.filter(AILogColumn::Timestamp.gte(date_from));
    }
    if let Some(date_to) = &filters.date_to {
        db_query = db_query.filter(AILogColumn::Timestamp.lte(date_to));
    }
    
    let logs = db_query
        .order_by_desc(AILogColumn::Timestamp)
        .limit(1000)
        .all(db)
        .await
        .map_err(|e| format!("Failed to search logs: {}", e))?;
    
    Ok(logs.into_iter().map(AILog::from).collect())
}

/// Export logs to file
#[tauri::command]
pub async fn ai_export_logs(
    filters: LogFilters,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<String, String> {
    let logs = ai_get_logs(filters, db_manager).await?;
    let json = serde_json::to_string_pretty(&logs)
        .map_err(|e| format!("Failed to serialize logs: {}", e))?;
    
    // For now, return JSON string - proper implementation would write to file
    Ok(json)
}

/// List training data
#[tauri::command]
pub async fn ai_list_training_data(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    let db = db_manager.get_connection();
    
    let training_data = TrainingDataEntity::find()
        .all(db)
        .await
        .map_err(|e| format!("Failed to list training data: {}", e))?;
    
    Ok(training_data.into_iter().map(|td| serde_json::json!({
        "id": td.id,
        "name": td.name,
        "type": td.type_,
        "content": td.content,
        "metadata": td.metadata,
        "created_at": td.created_at,
        "updated_at": td.updated_at,
    })).collect())
}

/// Delete training data
#[tauri::command]
pub async fn ai_delete_training_data(
    id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    
    TrainingDataEntity::delete_by_id(&id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete training data: {}", e))?;
    
    Ok(())
}

