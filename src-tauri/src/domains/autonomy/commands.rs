use tauri::command;
use serde_json::Value;
use crate::database::DatabaseManager;
use crate::domains::autonomy::services::AutonomyService;
use crate::domains::autonomy::services::autonomy_service::AutonomousActionRequest;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

// Global autonomy service instances per user/session
// In a real app, you'd manage this properly with user sessions
use std::sync::OnceLock;

static AUTONOMY_SERVICES: OnceLock<Arc<Mutex<HashMap<String, AutonomyService>>>> = OnceLock::new();

fn get_services_map() -> Arc<Mutex<HashMap<String, AutonomyService>>> {
    AUTONOMY_SERVICES.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

// Note: We can't clone AutonomyService easily, so we'll manage it through the mutex directly
// Functions will get mutable access when needed

#[command]
pub async fn evaluate_autonomous_action(
    action_type: String,
    action_data: Value,
    context: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Value, String> {
    
    let request = AutonomousActionRequest {
        action_type,
        action_data,
        context,
        user_id: None,
    };
    
    // Ensure service exists and evaluate action
    // We need to handle the mutex properly - acquire lock, get mutable reference, then release before await
    let db_conn = db_manager.get_connection_clone();
    let result = {
        let service_map = get_services_map();
        let mut services = service_map.lock().await;
        // Ensure service exists
        if !services.contains_key("default") {
            services.insert("default".to_string(), AutonomyService::new());
        }
        
        // Get autonomy level and enabled state before releasing lock
        let service = services.get("default").ok_or("Failed to get autonomy service")?;
        let autonomy_level = service.get_autonomy_level();
        let autonomy_enabled = service.is_enabled();
        
        // Release lock before await
        drop(services);
        
        // Create a temporary service instance with current settings for this operation
        let mut temp_service = AutonomyService::new();
        temp_service.set_autonomy_level(autonomy_level);
        temp_service.set_enabled(autonomy_enabled);
        temp_service.evaluate_action(&db_conn, request).await?
    };

    Ok(serde_json::json!({
        "action_id": result.action_id,
        "executed": result.executed,
        "requires_approval": result.requires_approval,
        "classification": {
            "safety_level": format!("{:?}", result.classification.safety_level),
            "confidence": result.classification.confidence,
            "reason": result.classification.reason,
        },
        "message": result.message,
    }))
}

#[command]
pub async fn record_autonomous_action_outcome(
    action_id: String,
    action_type: String,
    context: String,
    success: bool,
    feedback: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db_conn = db_manager.get_connection_clone();
    
    let mut temp_service = AutonomyService::new();
    temp_service.record_action_outcome(&db_conn, &action_id, &action_type, &context, success, feedback).await
}

#[command]
pub async fn get_autonomy_level(
) -> Result<String, String> {
    let service_map = get_services_map();
    let services = service_map.lock().await;
    let service = services.get("default")
        .ok_or("Autonomy service not found")?;
    Ok(format!("{:?}", service.get_autonomy_level()))
}

#[command]
pub async fn set_autonomy_level(
    level: String,
) -> Result<(), String> {
    let level = match level.to_lowercase().as_str() {
        "observation" => crate::domains::autonomy::services::autonomy_service::AutonomyLevel::Observation,
        "conservative" => crate::domains::autonomy::services::autonomy_service::AutonomyLevel::Conservative,
        "balanced" => crate::domains::autonomy::services::autonomy_service::AutonomyLevel::Balanced,
        "aggressive" => crate::domains::autonomy::services::autonomy_service::AutonomyLevel::Aggressive,
        _ => return Err(format!("Invalid autonomy level: {}", level)),
    };

    let service_map = get_services_map();
    let mut services = service_map.lock().await;
    let service = services.get_mut("default")
        .ok_or("Autonomy service not found")?;
    
    service.set_autonomy_level(level);
    Ok(())
}

#[command]
pub async fn get_autonomy_enabled(
) -> Result<bool, String> {
    let service_map = get_services_map();
    let services = service_map.lock().await;
    let service = services.get("default")
        .ok_or("Autonomy service not found")?;
    Ok(service.is_enabled())
}

#[command]
pub async fn set_autonomy_enabled(
    enabled: bool,
) -> Result<(), String> {
    let service_map = get_services_map();
    let mut services = service_map.lock().await;
    let service = services.get_mut("default")
        .ok_or("Autonomy service not found")?;
    
    service.set_enabled(enabled);
    Ok(())
}

#[command]
pub async fn get_approval_stats(
) -> Result<Value, String> {
    let service_map = get_services_map();
    let services = service_map.lock().await;
    let service = services.get("default")
        .ok_or("Autonomy service not found")?;
    let stats = service.get_approval_stats();
    
    let mut result = serde_json::Map::new();
    for (key, (approved, total)) in stats {
        result.insert(key, serde_json::json!({
            "approved": approved,
            "total": total,
            "approval_rate": if total > 0 { approved as f64 / total as f64 } else { 0.0 },
        }));
    }
    
    Ok(Value::Object(result))
}
