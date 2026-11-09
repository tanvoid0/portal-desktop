use super::super::CredentialError;
use super::encryption_service::{DecryptionRequest, EncryptionResult, EncryptionService};
use super::vault_session::VaultSession;
use crate::domains::credentials::entities::Column;
use crate::domains::credentials::entities::{
    ActiveModel as CredentialActive, Entity as CredentialEntity, Model as CredentialModel,
};
use chrono::Utc;
/**
 * Credential Service - Business logic for credential management
 */
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CredentialService {
    db: DatabaseConnection,
    encryption: EncryptionService,
    session: Arc<VaultSession>,
}

impl CredentialService {
    pub fn new(db: DatabaseConnection, session: Arc<VaultSession>) -> Self {
        Self {
            db,
            encryption: EncryptionService::new(),
            session,
        }
    }

    /// Create a new credential
    pub async fn create_credential(
        &self,
        request: CredentialCreateRequest,
    ) -> Result<CredentialModel, CredentialError> {
        let id = uuid::Uuid::new_v4().to_string();
        let id_for_verify = id.clone();
        let now = Utc::now().naive_utc();

        // Encrypt the credential value
        let master_key = self.session.key()?;
        let encryption_result = self.encryption.encrypt(&request.value, &master_key)?;

        // Encrypt additional fields
        let mut encrypted_fields = std::collections::HashMap::new();
        if let Some(fields) = request.fields {
            for (key, value) in fields {
                let field_result = self.encryption.encrypt(&value, &master_key)?;
                encrypted_fields.insert(key, serde_json::to_string(&field_result)?);
            }
        }

        let credential = CredentialActive {
            id: Set(id),
            name: Set(request.name),
            credential_type: Set(request.credential_type.to_string()),
            status: Set("active".to_string()),
            description: Set(request.description),
            tags: Set(serde_json::to_string(&request.tags.unwrap_or_default())?),
            encrypted_value: Set(serde_json::to_string(&encryption_result)?),
            encrypted_fields: Set(serde_json::to_string(&encrypted_fields)?),
            metadata: Set(serde_json::to_string(
                &request.metadata.unwrap_or_default(),
            )?),
            created_at: Set(now),
            updated_at: Set(now),
            last_used: Set(None),
            expires_at: Set(request.expires_at.map(|d| d.naive_utc())),
        };

        // SQLite string primary keys can fail to return the inserted row even when
        // the insert succeeds. Verify by ID when SeaORM reports RecordNotFound.
        match credential.insert(&self.db).await {
            Ok(result) => Ok(result),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("RecordNotFound")
                    || error_str.contains("Failed to find inserted item")
                {
                    CredentialEntity::find_by_id(&id_for_verify)
                        .one(&self.db)
                        .await?
                        .ok_or_else(|| CredentialError::CredentialNotFound(id_for_verify.clone()))
                } else {
                    Err(CredentialError::DatabaseError(e))
                }
            }
        }
    }

    /// Get all credentials
    pub async fn get_credentials(&self) -> Result<Vec<CredentialModel>, CredentialError> {
        let credentials = CredentialEntity::find().all(&self.db).await?;
        Ok(credentials)
    }

    /// Get credential by ID
    pub async fn get_credential(&self, id: &str) -> Result<CredentialModel, CredentialError> {
        let credential = CredentialEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| CredentialError::CredentialNotFound(id.to_string()))?;
        Ok(credential)
    }

    /// Update credential
    pub async fn update_credential(
        &self,
        id: &str,
        request: CredentialUpdateRequest,
    ) -> Result<CredentialModel, CredentialError> {
        let mut credential = self.get_credential(id).await?;
        let now = Utc::now().naive_utc();

        // Update fields
        if let Some(name) = request.name {
            credential.name = name;
        }
        if let Some(description) = request.description {
            credential.description = Some(description);
        }
        if let Some(tags) = request.tags {
            credential.tags = serde_json::to_string(&tags)?;
        }
        if let Some(status) = request.status {
            credential.status = status.to_string();
        }
        if let Some(expires_at) = request.expires_at {
            credential.expires_at = Some(expires_at.naive_utc());
        }

        // Update encrypted value if provided
        if let Some(value) = request.value {
            let master_key = self.session.key()?;
            let encryption_result = self.encryption.encrypt(&value, &master_key)?;
            credential.encrypted_value = serde_json::to_string(&encryption_result)?;
        }

        // Update encrypted fields if provided
        if let Some(fields) = request.fields {
            let master_key = self.session.key()?;
            let mut encrypted_fields = std::collections::HashMap::new();
            for (key, value) in fields {
                let field_result = self.encryption.encrypt(&value, &master_key)?;
                encrypted_fields.insert(key, serde_json::to_string(&field_result)?);
            }
            credential.encrypted_fields = serde_json::to_string(&encrypted_fields)?;
        }

        // Update metadata if provided
        if let Some(metadata) = request.metadata {
            credential.metadata = serde_json::to_string(&metadata)?;
        }

        credential.updated_at = now;

        // Save to database
        let active_model = CredentialActive {
            id: Set(credential.id.clone()),
            name: Set(credential.name.clone()),
            credential_type: Set(credential.credential_type.clone()),
            status: Set(credential.status.clone()),
            description: Set(credential.description.clone()),
            tags: Set(credential.tags.clone()),
            encrypted_value: Set(credential.encrypted_value.clone()),
            encrypted_fields: Set(credential.encrypted_fields.clone()),
            metadata: Set(credential.metadata.clone()),
            created_at: Set(credential.created_at),
            updated_at: Set(credential.updated_at),
            last_used: Set(credential.last_used),
            expires_at: Set(credential.expires_at),
        };

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    /// Delete credential
    pub async fn delete_credential(&self, id: &str) -> Result<(), CredentialError> {
        CredentialEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Decrypt credential value
    pub async fn decrypt_credential(&self, id: &str) -> Result<String, CredentialError> {
        let credential = self.get_credential(id).await?;
        // Ensure EncryptionResult is deserializable (ensure #[derive(serde::Deserialize)] on EncryptionResult)
        let encryption_data: EncryptionResult =
            serde_json::from_str(&credential.encrypted_value)
                .map_err(|e| CredentialError::DeserializationError(e.to_string()))?;

        let master_key = self.session.key()?;
        let request = DecryptionRequest {
            encrypted: encryption_data.encrypted.clone(),
            iv: encryption_data.iv.clone(),
            tag: encryption_data.tag,
            algorithm: encryption_data.algorithm,
            key: master_key,
        };

        let decrypted = self.encryption.decrypt(request)?;

        // Update last used timestamp
        let now = Utc::now().naive_utc();
        let active_model = CredentialActive {
            id: Set(credential.id),
            name: Set(credential.name),
            credential_type: Set(credential.credential_type),
            status: Set(credential.status),
            description: Set(credential.description),
            tags: Set(credential.tags),
            encrypted_value: Set(credential.encrypted_value),
            encrypted_fields: Set(credential.encrypted_fields),
            metadata: Set(credential.metadata),
            created_at: Set(credential.created_at),
            updated_at: Set(credential.updated_at),
            last_used: Set(Some(now)),
            expires_at: Set(credential.expires_at),
        };
        active_model.update(&self.db).await?;

        Ok(decrypted)
    }

    /// Decrypt without throwing when the vault is locked. A future caller
    /// (project-env drift checking) needs a non-throwing peek rather than an
    /// error on every check while locked.
    pub async fn peek_value(&self, id: &str) -> Result<Option<String>, CredentialError> {
        match self.decrypt_credential(id).await {
            Ok(value) => Ok(Some(value)),
            Err(CredentialError::VaultLocked) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Search credentials
    pub async fn search_credentials(
        &self,
        query: &str,
    ) -> Result<Vec<CredentialModel>, CredentialError> {
        let credentials = CredentialEntity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(Column::Name.contains(query))
                    .add(Column::Description.contains(query)),
            )
            .all(&self.db)
            .await?;
        Ok(credentials)
    }
}

#[derive(Debug, Clone)]
pub struct CredentialCreateRequest {
    pub name: String,
    pub credential_type: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub value: String,
    pub fields: Option<std::collections::HashMap<String, String>>,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct CredentialUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub value: Option<String>,
    pub fields: Option<std::collections::HashMap<String, String>>,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub status: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
