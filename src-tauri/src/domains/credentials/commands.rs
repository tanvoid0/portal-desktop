use crate::database::DatabaseManager;
use crate::domains::credentials::hello;
use crate::domains::credentials::services::vault_service::{VaultService, VaultState};
use crate::domains::credentials::services::{CredentialService, LockPolicy, VaultSession};
use std::sync::Arc;
/**
 * Credentials Tauri Commands
 */
use tauri::{Manager, State};

/// Create a new credential
#[portal_rpc_macro::rpc]
pub async fn create_credential(
    name: String,
    credential_type: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    value: String,
    fields: Option<std::collections::HashMap<String, String>>,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    let request =
        crate::domains::credentials::services::credential_service::CredentialCreateRequest {
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
        Err(e) => Err(e.to_string()),
    }
}

/// Get all credentials
#[portal_rpc_macro::rpc]
pub async fn get_credentials(
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<Vec<serde_json::Value>, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    match service.get_credentials().await {
        Ok(credentials) => {
            let result: Vec<serde_json::Value> = credentials
                .into_iter()
                .map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null))
                .collect();
            Ok(result)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Get credential by ID
#[portal_rpc_macro::rpc]
pub async fn get_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    match service.get_credential(&id).await {
        Ok(credential) => Ok(serde_json::to_value(credential).unwrap_or(serde_json::Value::Null)),
        Err(e) => Err(e.to_string()),
    }
}

/// Update credential
#[portal_rpc_macro::rpc]
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
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<serde_json::Value, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    let request =
        crate::domains::credentials::services::credential_service::CredentialUpdateRequest {
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
        Err(e) => Err(e.to_string()),
    }
}

/// Delete credential
#[portal_rpc_macro::rpc]
pub async fn delete_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<(), String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    match service.delete_credential(&id).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Decrypt credential value
#[portal_rpc_macro::rpc]
pub async fn decrypt_credential(
    id: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<String, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    match service.decrypt_credential(&id).await {
        Ok(value) => Ok(value),
        Err(e) => Err(e.to_string()),
    }
}

/// Status of the vault commands return — kept IPC-only (`local_only`): the
/// daemon has no window to anchor a Hello prompt to, and nothing LAN-side
/// should see lock state or be able to drive unlock/reset.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultStatusResponse {
    pub state: String,
    pub hello_available: bool,
    pub has_credentials: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultSetupResponse {
    pub recovery_key: String,
}

/// Vault status: setup/lock state, Windows Hello availability, whether any
/// credentials exist yet.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_status(
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<VaultStatusResponse, String> {
    let service = VaultService::new(db.get_connection_clone(), session.inner().clone());
    let status = service.status().await.map_err(|e| e.to_string())?;
    Ok(VaultStatusResponse {
        state: match status.state {
            VaultState::NeedsSetup => "needs_setup",
            VaultState::Locked => "locked",
            VaultState::Unlocked => "unlocked",
        }
        .to_string(),
        hello_available: hello::available().await,
        has_credentials: status.has_credentials,
    })
}

/// First-time vault setup. Returns the recovery key exactly once — there is
/// deliberately no command to fetch it again later.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_setup(
    password: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<VaultSetupResponse, String> {
    let service = VaultService::new(db.get_connection_clone(), session.inner().clone());
    let recovery_key = service.setup(&password).await.map_err(|e| e.to_string())?;
    Ok(VaultSetupResponse { recovery_key })
}

/// Unlock with the master password.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_unlock_with_password(
    password: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
    prompter: State<'_, Arc<crate::domains::credentials::prompter::TauriVaultPrompter>>,
) -> Result<(), String> {
    let service = VaultService::new(db.get_connection_clone(), session.inner().clone())
        .with_data_dir(db.data_dir());
    service
        .unlock_with_password(&password)
        .await
        .map_err(|e| e.to_string())?;
    prompter.settle(true); // release waiters even if the unlock event was lost
    Ok(())
}

/// Unlock via the OS keychain, gated by a Windows Hello consent prompt
/// anchored to the main window. Button-triggered only — never call this on
/// dialog mount.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_unlock_with_hello(
    app: tauri::AppHandle,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
    prompter: State<'_, Arc<crate::domains::credentials::prompter::TauriVaultPrompter>>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    #[cfg(windows)]
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    #[cfg(not(windows))]
    let hwnd = {
        let _ = window;
        0isize
    };

    let verified = hello::verify(hwnd, "Unlock Portal Desktop").await?;
    if !verified {
        return Err("Windows Hello verification was not completed".to_string());
    }

    let service = VaultService::new(db.get_connection_clone(), session.inner().clone())
        .with_data_dir(db.data_dir());
    service
        .unlock_with_keychain()
        .await
        .map_err(|e| e.to_string())?;
    prompter.settle(true);
    Ok(())
}

/// Lock the vault immediately. Also stops the next launch from unlocking itself.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_lock(session: State<'_, Arc<VaultSession>>) -> Result<(), String> {
    session.lock();
    Ok(())
}

/// When the vault locks itself again.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_get_lock_policy(
    session: State<'_, Arc<VaultSession>>,
) -> Result<LockPolicy, String> {
    Ok(session.policy())
}

/// Needs an unlocked vault: loosening the policy is as sensitive as using a credential.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_set_lock_policy(
    policy: LockPolicy,
    session: State<'_, Arc<VaultSession>>,
) -> Result<(), String> {
    session.require_key().await.map_err(|e| e.to_string())?;
    session.set_policy(policy).map_err(|e| e.to_string())
}

/// Frontend's answer to `vault://unlock-requested`: `true` once unlocked,
/// `false` if the user dismissed the prompt. Releases every command waiting
/// in `VaultSession::require_key`.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_unlock_settled(
    ok: bool,
    prompter: State<'_, Arc<crate::domains::credentials::prompter::TauriVaultPrompter>>,
) -> Result<(), String> {
    prompter.settle(ok);
    Ok(())
}

/// Reset the master password using the recovery key. The master key itself
/// is unchanged, so existing credentials stay decryptable.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_reset_password(
    recovery_key: String,
    new_password: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
    prompter: State<'_, Arc<crate::domains::credentials::prompter::TauriVaultPrompter>>,
) -> Result<(), String> {
    let service = VaultService::new(db.get_connection_clone(), session.inner().clone())
        .with_data_dir(db.data_dir());
    service
        .reset_password(&recovery_key, &new_password)
        .await
        .map_err(|e| e.to_string())?;
    prompter.settle(true);
    Ok(())
}

/// Change the master password: the current one is verified, then the master key is re-wrapped
/// under the new one. The master key itself is unchanged, so credentials are not re-encrypted
/// and the recovery key keeps working. Leaves the vault unlocked.
#[portal_rpc_macro::rpc(local_only)]
pub async fn vault_change_password(
    old_password: String,
    new_password: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
    prompter: State<'_, Arc<crate::domains::credentials::prompter::TauriVaultPrompter>>,
) -> Result<(), String> {
    let service = VaultService::new(db.get_connection_clone(), session.inner().clone())
        .with_data_dir(db.data_dir());
    service
        .change_password(&old_password, &new_password)
        .await
        .map_err(|e| e.to_string())?;
    prompter.settle(true);
    Ok(())
}

/// Search credentials
#[portal_rpc_macro::rpc]
pub async fn search_credentials(
    query: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<Vec<serde_json::Value>, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.inner().clone());

    match service.search_credentials(&query).await {
        Ok(credentials) => {
            let result: Vec<serde_json::Value> = credentials
                .into_iter()
                .map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null))
                .collect();
            Ok(result)
        }
        Err(e) => Err(e.to_string()),
    }
}
