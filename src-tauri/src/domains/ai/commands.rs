use crate::database::DatabaseManager;
use crate::domains::ai::catalog::{CatalogQuery, PlatformCatalog};
use crate::domains::ai::chat;
use crate::domains::ai::chat_title::{
    fallback_title_from_message, generate_smart_title, is_placeholder_title,
    should_apply_generated_title, smart_titles_enabled, PLACEHOLDER_CHAT,
};
use crate::domains::ai::conversation::{
    Conversation, ConversationMessage, ConversationWithMessages,
};
use crate::domains::ai::logging::{AILog, LogFilters};
use crate::domains::ai::message::ChatMessage;
use crate::domains::ai::platform_config::PlatformConfig;
use crate::domains::ai::providers::{
    AIError, AgentPlatformProvider, ConfigurationStatus, GenerationOptions, GenerationResult,
    ProviderConfig, ProviderType,
};
use crate::domains::ai::services::{AIService, AISettingsService};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use tauri::{Emitter, State};
// Use the centralized logger from utils
use crate::domains::ai::entities::ai_conversation::Column as ConversationColumn;
use crate::domains::ai::entities::ai_conversation_message::Column as ConversationMessageColumn;
use crate::domains::ai::entities::{
    AILogColumn, AILogEntity, ConversationActiveModel, ConversationEntity,
    ConversationMessageActiveModel, ConversationMessageEntity, TrainingDataEntity,
};

// Import logger macros
use crate::{log_debug, log_error, log_info};

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

/// Fetch agent-platform provider/model catalog (`GET /v1/catalog`).
#[tauri::command]
pub async fn get_ai_platform_catalog(
    providers: Option<Vec<String>>,
    live: Option<bool>,
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<PlatformCatalog, String> {
    let config = settings_service
        .get_provider_config(ProviderType::AgentPlatform)
        .map_err(|e| e.to_string())?;
    let platform = AgentPlatformProvider::new(config);
    let query = CatalogQuery { providers, live };
    platform
        .fetch_catalog(query)
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
        .map_err(|e| match e {
            AIError::ConfigurationIncomplete(status) => {
                format!(
                    "Configuration incomplete. Missing fields: {}. Warnings: {}",
                    status.missing_fields.join(", "),
                    status.warnings.join(", ")
                )
            }
            _ => e.to_string(),
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
        .map_err(|e| match e {
            AIError::ConfigurationIncomplete(status) => {
                format!(
                    "Configuration incomplete. Missing fields: {}. Warnings: {}",
                    status.missing_fields.join(", "),
                    status.warnings.join(", ")
                )
            }
            _ => e.to_string(),
        })
}

/// Send a message to AI (chat) - non-streaming
#[tauri::command]
pub async fn ai_send_message(
    message: String,
    history: Vec<ChatMessage>,
    provider: Option<ProviderType>,
    conversation_id: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    model: Option<String>,
    llm_provider: Option<String>,
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
        llm_provider,
    };
    chat::send_message(request, ai_service).await
}

/// Send a message to AI (chat) with streaming support
#[tauri::command]
pub async fn ai_send_message_stream(
    message: String,
    history: Vec<ChatMessage>,
    provider: Option<ProviderType>,
    conversation_id: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    model: Option<String>,
    llm_provider: Option<String>,
    stream_id: String, // Unique ID for this stream
    app_handle: tauri::AppHandle,
    ai_service: State<'_, Arc<AIService>>,
    db_manager: State<'_, Arc<DatabaseManager>>,
    settings_service: State<'_, Arc<AISettingsService>>,
) -> Result<String, String> {
    let is_first_user = history.iter().filter(|m| m.role == "user").count() == 0;
    let fallback_title = if is_first_user {
        Some(fallback_title_from_message(&message, PLACEHOLDER_CHAT))
    } else {
        None
    };

    if let (Some(conv_id), Some(fb)) = (&conversation_id, &fallback_title) {
        let db = db_manager.get_connection();
        if let Ok(Some(conv_model)) = ConversationEntity::find_by_id(conv_id).one(db).await {
            if is_placeholder_title(&conv_model.title) {
                let mut conversation: ConversationActiveModel = conv_model.into();
                conversation.title = Set(fb.clone());
                conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());
                let _ = conversation.update(db).await;
                let _ = app_handle.emit(
                    &format!("ai-stream-title-{stream_id}"),
                    serde_json::json!({ "conversation_id": conv_id, "title": fb }),
                );

                if smart_titles_enabled() {
                    let app = app_handle.clone();
                    let sid = stream_id.clone();
                    let cid = conv_id.clone();
                    let fb_clone = fb.clone();
                    let msg = message.clone();
                    let cfg = PlatformConfig::resolve(settings_service.inner());
                    let db_arc = db_manager.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        let client = Client::new();
                        let Ok(smart) = generate_smart_title(&client, &cfg, &msg).await else {
                            return;
                        };
                        if smart == fb_clone {
                            return;
                        }
                        let db = db_arc.get_connection();
                        let Ok(Some(conv_model)) =
                            ConversationEntity::find_by_id(&cid).one(db).await
                        else {
                            return;
                        };
                        if !should_apply_generated_title(&conv_model.title, &fb_clone) {
                            return;
                        }
                        let mut conversation: ConversationActiveModel = conv_model.into();
                        conversation.title = Set(smart.clone());
                        conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());
                        if conversation.update(db).await.is_ok() {
                            let _ = app.emit(
                                &format!("ai-stream-title-{sid}"),
                                serde_json::json!({ "conversation_id": cid, "title": smart }),
                            );
                        }
                    });
                }
            }
        }
    }

    let options = GenerationOptions {
        temperature,
        max_tokens,
        timeout_ms: None,
        model,
        llm_provider,
        extra_options: None,
    };

    let mut messages = history;
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: message,
    });

    let provider_type = provider;

    let mut full_response = String::new();
    let app_handle_clone = app_handle.clone();
    let stream_id_clone = stream_id.clone();
    let result = ai_service
        .generate_chat_stream(
            &messages,
            Some(options),
            provider_type,
            Box::new(move |chunk: String| {
                full_response.push_str(&chunk);
                app_handle_clone
                    .emit(&format!("ai-stream-chunk-{}", stream_id_clone), &chunk)
                    .map_err(|e| AIError::GenericError(format!("Failed to emit event: {}", e)))?;
                Ok(())
            }),
        )
        .await;

    let final_title = if let Some(conv_id) = &conversation_id {
        let db = db_manager.get_connection();
        ConversationEntity::find_by_id(conv_id)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|m| m.title)
    } else {
        None
    };

    match result {
        Ok(gen_result) => {
            let complete_payload = if let Some(title) = final_title {
                serde_json::json!({ "content": gen_result.content, "title": title })
            } else {
                serde_json::json!({ "content": gen_result.content })
            };
            app_handle
                .emit(
                    &format!("ai-stream-complete-{}", stream_id),
                    &complete_payload,
                )
                .map_err(|e| format!("Failed to emit completion event: {}", e))?;
            Ok(gen_result.content)
        }
        Err(e) => Err(format!("AI generation error: {}", e)),
    }
}

/// Create a new conversation
#[tauri::command]
pub async fn ai_create_conversation(
    title: String,
    provider: ProviderType,
    model: Option<String>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Conversation, String> {
    log_info!(
        "AI",
        "Creating new conversation: title='{}', provider={:?}",
        title,
        provider
    );

    let conversation = Conversation::new(title.clone(), format!("{:?}", provider), model);
    log_debug!("AI", "Generated conversation ID: {}", conversation.id);

    let db = db_manager.get_connection();

    let active_model = ConversationActiveModel {
        id: Set(conversation.id.clone()),
        title: Set(conversation.title.clone()),
        provider: Set(conversation.provider.clone()),
        model: Set(conversation.model.clone()),
        created_at: Set(conversation.created_at.clone()),
        updated_at: Set(conversation.updated_at.clone()),
    };

    // Insert the conversation
    // Note: For SQLite with string primary keys, insert() tries to return the inserted record
    // which can fail with "RecordNotFound" even though the insert succeeded.
    // We handle this by checking if the error is RecordNotFound and verifying the insert succeeded.
    log_info!("AI", "Inserting conversation into database...");
    match active_model.insert(db).await {
        Ok(inserted_model) => {
            // Insert succeeded and record was returned
            log_info!(
                "AI",
                "Successfully created conversation with ID: {}",
                inserted_model.id
            );
            Ok(Conversation::from(inserted_model))
        }
        Err(e) => {
            let error_str = e.to_string();
            // If error is RecordNotFound, the insert likely succeeded but SeaORM
            // couldn't retrieve it (SQLite limitation). Verify it exists.
            if error_str.contains("RecordNotFound")
                || error_str.contains("Failed to find inserted item")
            {
                log_debug!(
                    "AI",
                    "Insert returned RecordNotFound, verifying conversation exists with ID: {}",
                    conversation.id
                );
                // Verify the insert actually succeeded
                let inserted_model = ConversationEntity::find_by_id(&conversation.id)
                    .one(db)
                    .await
                    .map_err(|e| {
                        log_error!("AI", "Failed to verify conversation insert: {}", e);
                        format!("Failed to verify created conversation: {}", e)
                    })?
                    .ok_or_else(|| {
                        let err_msg = format!(
                            "Failed to find inserted conversation with id: {} after insert",
                            conversation.id
                        );
                        log_error!("AI", "{}", err_msg);
                        err_msg
                    })?;

                log_info!(
                    "AI",
                    "Successfully created conversation with ID: {} (verified after RecordNotFound)",
                    inserted_model.id
                );
                Ok(Conversation::from(inserted_model))
            } else {
                // Real error, return it
                log_error!("AI", "Failed to insert conversation: {}", e);
                Err(format!("Failed to create conversation: {}", e))
            }
        }
    }
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
    // Note: For SQLite with string primary keys, insert() tries to return the inserted record
    // which can fail with "RecordNotFound" even though the insert succeeded.
    // We handle this by checking if the error is RecordNotFound and verifying the insert succeeded.
    for msg in messages {
        let active_model = ConversationMessageActiveModel {
            id: Set(msg.id.clone()),
            conversation_id: Set(msg.conversation_id.clone()),
            role: Set(msg.role.clone()),
            content: Set(msg.content.clone()),
            timestamp: Set(msg.timestamp.clone()),
            sequence: Set(msg.sequence),
        };

        // Attempt insert - may fail with RecordNotFound even if insert succeeded
        match active_model.insert(db).await {
            Ok(_) => {
                // Insert succeeded and record was returned
            }
            Err(e) => {
                let error_str = e.to_string();
                // If error is RecordNotFound, the insert likely succeeded but SeaORM
                // couldn't retrieve it (SQLite limitation). Verify it exists.
                if error_str.contains("RecordNotFound")
                    || error_str.contains("Failed to find inserted item")
                {
                    // Verify the insert actually succeeded
                    let exists = ConversationMessageEntity::find_by_id(&msg.id)
                        .one(db)
                        .await
                        .map_err(|e| format!("Failed to verify message insert: {}", e))?
                        .is_some();

                    if !exists {
                        return Err(format!(
                            "Failed to save message: insert failed - record not found after insert"
                        ));
                    }
                    // Insert succeeded, continue to next message
                } else {
                    // Real error, return it
                    return Err(format!("Failed to save message: {}", e));
                }
            }
        }
    }

    // Update conversation updated_at
    let mut conversation: ConversationActiveModel = ConversationEntity::find_by_id(&id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to find conversation: {}", e))?
        .ok_or_else(|| "Conversation not found".to_string())?
        .into();

    conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());
    conversation
        .update(db)
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
        messages: messages_models
            .into_iter()
            .map(ConversationMessage::from)
            .collect(),
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

    // Calculate message count for each conversation
    let mut result = Vec::new();
    for conv_model in conversations {
        let message_count: u64 = ConversationMessageEntity::find()
            .filter(ConversationMessageColumn::ConversationId.eq(&conv_model.id))
            .count(db)
            .await
            .map_err(|e| format!("Failed to count messages: {}", e))?;

        let mut conversation: Conversation = Conversation::from(conv_model);
        conversation.message_count = Some(message_count as i32);
        result.push(conversation);
    }

    Ok(result)
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

    conversation
        .update(db)
        .await
        .map_err(|e| format!("Failed to update conversation title: {}", e))?;

    Ok(())
}

/// Update the model used for a conversation (allows mid-thread switching).
#[tauri::command]
pub async fn ai_update_conversation_model(
    id: String,
    model: Option<String>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();

    let mut conversation: ConversationActiveModel = ConversationEntity::find_by_id(&id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to find conversation: {}", e))?
        .ok_or_else(|| "Conversation not found".to_string())?
        .into();

    conversation.model = Set(model);
    conversation.updated_at = Set(chrono::Utc::now().to_rfc3339());

    conversation
        .update(db)
        .await
        .map_err(|e| format!("Failed to update conversation model: {}", e))?;

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

    let mut db_query = AILogEntity::find().filter(
        AILogColumn::RequestData
            .like(&search_pattern)
            .or(AILogColumn::ResponseData.like(&search_pattern))
            .or(AILogColumn::ErrorMessage.like(&search_pattern)),
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

    Ok(training_data
        .into_iter()
        .map(|td| {
            serde_json::json!({
                "id": td.id,
                "name": td.name,
                "type": td.type_,
                "content": td.content,
                "metadata": td.metadata,
                "created_at": td.created_at,
                "updated_at": td.updated_at,
            })
        })
        .collect())
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
