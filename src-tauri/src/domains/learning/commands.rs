use tauri::command;
use serde_json::{Value, json};
use crate::database::DatabaseManager;
use crate::domains::learning::services::{LearningService, MLIntensity, MemoryManager};
use std::sync::Arc;

#[command]
pub async fn record_learning_event(
    event_type: String,
    event_data: Value,
    outcome: Option<String>,
    context: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<i32, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.record_event(&db, event_type, event_data, outcome, context).await
}

#[command]
pub async fn learn_pattern(
    pattern_type: String,
    pattern_data: Value,
    context: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<i32, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.learn_pattern(&db, pattern_type, pattern_data, context).await
}

#[command]
pub async fn record_pattern_outcome(
    pattern_id: i32,
    success: bool,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.record_pattern_outcome(&db, pattern_id, success).await
}

#[command]
pub async fn get_suggestions(
    pattern_type: String,
    context: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Value>, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.get_suggestions(&db, &pattern_type, context.as_deref()).await
}

#[command]
pub async fn learn_preference(
    preference_type: String,
    context: Option<String>,
    preference_value: Value,
    learned_from: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<i32, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.learn_preference(&db, preference_type, context, preference_value, learned_from).await
}

#[command]
pub async fn get_preference(
    preference_type: String,
    context: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<Value>, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    service.get_preference(&db, &preference_type, context.as_deref()).await
}

#[command]
pub async fn get_ml_intensity(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<String, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    
    // Try to get saved intensity from preferences
    match service.get_preference(&db, "ml_intensity", Some("global")).await {
        Ok(Some(value)) => {
            if let Some(intensity_str) = value.get("intensity").and_then(|v| v.as_str()) {
                return Ok(intensity_str.to_string());
            }
        }
        Ok(None) => {}
        Err(_) => {} // Log error but continue to default
    }
    
    // Return default
    Ok(MLIntensity::default().to_string().to_string())
}

#[command]
pub async fn set_ml_intensity(
    intensity: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    
    let ml_intensity = MLIntensity::from_str(&intensity);
    
    // Save to preferences
    service.learn_preference(
        &db,
        "ml_intensity".to_string(),
        Some("global".to_string()),
        serde_json::json!({ "intensity": ml_intensity.to_string() }),
        Some("user_setting".to_string()),
    )
    .await
    .map_err(|e| format!("Failed to save ML intensity: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_ml_enabled(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<bool, String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    
    // Get enabled state from preferences
    let preference = service.get_preference(&db, "ml_enabled", Some("global"))
        .await
        .map_err(|e| format!("Failed to get ML enabled state: {}", e))?;
    
    if let Some(value) = preference {
        if let Some(enabled) = value.get("enabled").and_then(|v| v.as_bool()) {
            return Ok(enabled);
        }
    }
    
    // Default to enabled
    Ok(true)
}

#[command]
pub async fn set_ml_enabled(
    enabled: bool,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    let service = LearningService::with_default();
    
    // Save enabled state to preferences
    service.learn_preference(
        &db,
        "ml_enabled".to_string(),
        Some("global".to_string()),
        serde_json::json!({ "enabled": enabled }),
        Some("user_setting".to_string()),
    )
    .await
    .map_err(|e| format!("Failed to save ML enabled state: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_all_patterns(
    limit: Option<u64>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Value>, String> {
    use crate::domains::learning::repositories::LearnedPatternRepository;
    
    let db = db_manager.get_connection();
    let patterns = LearnedPatternRepository::get_all(&db)
        .await
        .map_err(|e| format!("Failed to get patterns: {}", e))?;
    
    let patterns = if let Some(lim) = limit {
        patterns.into_iter().take(lim as usize).collect()
    } else {
        patterns
    };
    
    let mut result = Vec::new();
    for pattern in patterns {
        let pattern_data: Value = serde_json::from_str(&pattern.pattern_data)
            .unwrap_or_else(|_| json!(null));
        
        result.push(json!({
            "id": pattern.id,
            "pattern_type": pattern.pattern_type,
            "pattern_data": pattern_data,
            "context": pattern.context,
            "frequency": pattern.frequency,
            "last_used": pattern.last_used.map(|d| d.to_string()),
            "success_rate": pattern.success_rate,
            "is_important": pattern.is_important,
            "created_at": pattern.created_at.map(|d| d.to_string()),
        }));
    }
    
    Ok(result)
}

#[command]
pub async fn get_recent_events(
    limit: u64,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Value>, String> {
    use crate::domains::learning::repositories::LearningEventRepository;
    
    let db = db_manager.get_connection();
    let events = LearningEventRepository::get_recent(&db, limit)
        .await
        .map_err(|e| format!("Failed to get events: {}", e))?;
    
    let mut result = Vec::new();
    for event in events {
        let event_data: Value = serde_json::from_str(&event.event_data)
            .unwrap_or_else(|_| json!(null));
        
        result.push(json!({
            "id": event.id,
            "event_type": event.event_type,
            "event_data": event_data,
            "outcome": event.outcome,
            "context": event.context,
            "created_at": event.created_at.map(|d| d.to_string()),
        }));
    }
    
    Ok(result)
}

#[command]
pub async fn get_all_preferences(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Value>, String> {
    use crate::domains::learning::repositories::UserPreferenceRepository;
    
    let db = db_manager.get_connection();
    let preferences = UserPreferenceRepository::get_all(&db)
        .await
        .map_err(|e| format!("Failed to get preferences: {}", e))?;
    
    let mut result = Vec::new();
    for pref in preferences {
        let preference_value: Value = serde_json::from_str(&pref.preference_value)
            .unwrap_or_else(|_| json!(null));
        
        result.push(json!({
            "id": pref.id,
            "preference_type": pref.preference_type,
            "context": pref.context,
            "preference_value": preference_value,
            "confidence": pref.confidence,
            "learned_from": pref.learned_from,
            "is_important": pref.is_important,
            "created_at": pref.created_at.map(|d| d.to_string()),
            "updated_at": pref.updated_at.map(|d| d.to_string()),
        }));
    }
    
    Ok(result)
}

#[command]
pub async fn cleanup_learning_data(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Value, String> {
    let db = db_manager.get_connection();
    let memory_manager = MemoryManager::new();
    
    let stats = memory_manager.cleanup(&db).await
        .map_err(|e| format!("Failed to cleanup learning data: {}", e))?;
    
    Ok(serde_json::json!({
        "events_deleted": stats.events_deleted,
        "patterns_deleted": stats.patterns_deleted,
        "patterns_consolidated": stats.patterns_consolidated,
    }))
}

#[command]
pub async fn get_memory_stats(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Value, String> {
    let db = db_manager.get_connection();
    let memory_manager = MemoryManager::new();
    
    let stats = memory_manager.get_stats(&db).await
        .map_err(|e| format!("Failed to get memory stats: {}", e))?;
    
    Ok(serde_json::json!({
        "total_events": stats.total_events,
        "total_patterns": stats.total_patterns,
        "total_preferences": stats.total_preferences,
        "events_retention_days": stats.events_retention_days,
        "patterns_retention_days": stats.patterns_retention_days,
        "max_events": stats.max_events,
        "max_patterns_per_type": stats.max_patterns_per_type,
    }))
}

#[command]
pub async fn get_cleanup_preview(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Value, String> {
    let db = db_manager.get_connection();
    let memory_manager = MemoryManager::new();
    
    let preview = memory_manager.get_cleanup_preview(&db).await
        .map_err(|e| format!("Failed to get cleanup preview: {}", e))?;
    
    Ok(serde_json::json!({
        "events_to_delete": preview.events_to_delete,
        "patterns_to_delete": preview.patterns_to_delete,
        "events_over_limit": preview.events_over_limit,
    }))
}

#[command]
pub async fn mark_pattern_important(
    pattern_id: i32,
    is_important: bool,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    use crate::domains::learning::repositories::LearnedPatternRepository;
    
    let db = db_manager.get_connection();
    LearnedPatternRepository::mark_important(&db, pattern_id, is_important)
        .await
        .map_err(|e| format!("Failed to mark pattern as important: {}", e))?;
    
    Ok(())
}

