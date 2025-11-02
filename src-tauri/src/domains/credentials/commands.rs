/**
 * Credentials Tauri Commands
 */

use tauri::State;
use crate::database::DatabaseManager;
use crate::domains::credentials::services::CredentialService;
use std::sync::Arc;

/// Create a new credential
#[tauri::command]
pub async fn create_credential(
    name: String,
    credential_type: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    value: String,
    fields: Option<std::collections::HashMap<String, String>>,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone());

    let request = crate::domains::credentials::services::credential_service::CredentialCreateRequest {
        name,
        credential_type,
        description,
        tags,
        value,
        fields,
        metadata,
        expires_at,
    };

    match service.create_credential(request).await {
        Ok(credential) => Ok(serde_json::to_value(credential).unwrap_or(serde_json::Value::Null)),
        Err(e) => Err(e.to_string())
    }
}

/// Get all credentials
#[tauri::command]
pub async fn get_credentials(
    db: State<'_, Arc<DatabaseManager>>
) -> Result<Vec<serde_json::Value>, String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    match service.get_credentials().await {
        Ok(credentials) => {
            let result: Vec<serde_json::Value> = credentials
                .into_iter()
                .map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null))
                .collect();
            Ok(result)
        }
        Err(e) => Err(e.to_string())
    }
}

/// Get credential by ID
#[tauri::command]
pub async fn get_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    match service.get_credential(&id).await {
        Ok(credential) => Ok(serde_json::to_value(credential).unwrap_or(serde_json::Value::Null)),
        Err(e) => Err(e.to_string())
    }
}

/// Update credential
#[tauri::command]
pub async fn update_credential(
    id: String,
    name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    value: Option<String>,
    fields: Option<std::collections::HashMap<String, String>>,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    status: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    let request = crate::domains::credentials::services::credential_service::CredentialUpdateRequest {
        name,
        description,
        tags,
        value,
        fields,
        metadata,
        status,
        expires_at,
    };

    match service.update_credential(&id, request).await {
        Ok(credential) => Ok(serde_json::to_value(credential).unwrap_or(serde_json::Value::Null)),
        Err(e) => Err(e.to_string())
    }
}

/// Delete credential
#[tauri::command]
pub async fn delete_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<(), String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    match service.delete_credential(&id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

/// Decrypt credential value
#[tauri::command]
pub async fn decrypt_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<String, String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    match service.decrypt_credential(&id).await {
        Ok(value) => Ok(value),
        Err(e) => Err(e.to_string())
    }
}

/// Search credentials
#[tauri::command]
pub async fn search_credentials(
    query: String,
    db: State<'_, Arc<DatabaseManager>>
) -> Result<Vec<serde_json::Value>, String> {
    let service = CredentialService::new(db.get_connection_clone());
    
    match service.search_credentials(&query).await {
        Ok(credentials) => {
            let result: Vec<serde_json::Value> = credentials
                .into_iter()
                .map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null))
                .collect();
            Ok(result)
        }
        Err(e) => Err(e.to_string())
    }
}
