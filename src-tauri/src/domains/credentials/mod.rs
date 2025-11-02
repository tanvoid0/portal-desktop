/**
 * Credentials Domain Module
 */

pub mod commands;
pub mod entities;
pub mod services;


#[derive(Debug)]
pub enum CredentialError {
    CredentialNotFound(String),
    VaultNotFound(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidMasterPassword,
    VaultLocked,
    SessionExpired,
    DatabaseError(sea_orm::DbErr),
    IOError(std::io::Error),
    SerializationError(serde_json::Error),
    DeserializationError(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialError::CredentialNotFound(id) => write!(f, "Credential not found: {}", id),
            CredentialError::VaultNotFound(id) => write!(f, "Vault not found: {}", id),
            CredentialError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            CredentialError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            CredentialError::InvalidMasterPassword => write!(f, "Invalid master password"),
            CredentialError::VaultLocked => write!(f, "Vault is locked"),
            CredentialError::SessionExpired => write!(f, "Session expired"),
            CredentialError::DatabaseError(err) => write!(f, "Database error: {}", err),
            CredentialError::IOError(err) => write!(f, "IO error: {}", err),
            CredentialError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            CredentialError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl From<sea_orm::DbErr> for CredentialError {
    fn from(err: sea_orm::DbErr) -> Self {
        CredentialError::DatabaseError(err)
    }
}

impl From<std::io::Error> for CredentialError {
    fn from(err: std::io::Error) -> Self {
        CredentialError::IOError(err)
    }
}

impl From<serde_json::Error> for CredentialError {
    fn from(err: serde_json::Error) -> Self {
        CredentialError::SerializationError(err)
    }
}

impl From<CredentialError> for String {
    fn from(error: CredentialError) -> Self {
        error.to_string()
    }
}
