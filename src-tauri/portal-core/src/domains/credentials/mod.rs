/**
 * Credentials Domain Module
 */
pub mod entities;
pub mod services;

#[derive(Debug)]
pub enum CredentialError {
    CredentialNotFound(String),
    VaultNotFound(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidMasterPassword,
    InvalidRecoveryKey,
    VaultAlreadySetUp,
    VaultNotSetUp,
    VaultLocked,
    SessionExpired,
    /// The OS keychain's master key entry is gone (keychain reset, bundle identifier
    /// change, entry deleted by hand) while encrypted credential rows still exist. Minting a
    /// fresh key here would silently and permanently orphan every one of them, so this
    /// refuses instead — see QA-049.
    MasterKeyMissing(usize),
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
            CredentialError::InvalidRecoveryKey => write!(f, "Invalid recovery key"),
            CredentialError::VaultAlreadySetUp => write!(f, "Vault is already set up"),
            CredentialError::VaultNotSetUp => write!(f, "Vault is not set up"),
            CredentialError::VaultLocked => write!(f, "Vault is locked"),
            CredentialError::SessionExpired => write!(f, "Session expired"),
            CredentialError::MasterKeyMissing(count) => write!(
                f,
                "master key missing — {} credential{} can't be decrypted; restore the key or reset credentials",
                count,
                if *count == 1 { "" } else { "s" }
            ),
            CredentialError::DatabaseError(err) => write!(f, "Database error: {}", err),
            CredentialError::IOError(err) => write!(f, "IO error: {}", err),
            CredentialError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            CredentialError::DeserializationError(err) => {
                write!(f, "Deserialization error: {}", err)
            }
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
