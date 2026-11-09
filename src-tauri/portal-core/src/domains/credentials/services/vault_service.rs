/**
 * Vault Service - setup, unlock, lock and password reset for the credential
 * master key (MK).
 *
 * MK itself never changes. It is stored wrapped two ways in the single-row
 * `vault` table: under a key derived from the master password (PBKDF2) and
 * under a key derived from the recovery key (HKDF). The OS keychain also
 * keeps a raw copy, gated by a Windows Hello consent check upstream of
 * `unlock_with_keychain`. Because MK never changes, a password reset never
 * touches the already-encrypted `credentials` rows.
 */
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use data_encoding::BASE32_NOPAD;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

use super::super::entities::vault::{ActiveModel as VaultActive, Entity as VaultEntity};
use super::super::entities::Entity as CredentialEntity;
use super::super::CredentialError;
use super::encryption_service::{
    DecryptionRequest, EncryptionResult, EncryptionService, PBKDF2_ITERATIONS,
};
use super::vault_session::VaultSession;

const VAULT_ID: i32 = 1;
const RECOVERY_KEY_BYTES: usize = 20;
const RECOVERY_HKDF_INFO: &[u8] = b"portal-desktop-vault-recovery";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultState {
    NeedsSetup,
    Locked,
    Unlocked,
}

#[derive(Debug, Clone)]
pub struct VaultStatus {
    pub state: VaultState,
    pub has_credentials: bool,
}

#[derive(Clone)]
pub struct VaultService {
    db: DatabaseConnection,
    session: Arc<VaultSession>,
    encryption: EncryptionService,
}

impl VaultService {
    pub fn new(db: DatabaseConnection, session: Arc<VaultSession>) -> Self {
        Self {
            db,
            session,
            encryption: EncryptionService::new(),
        }
    }

    pub async fn status(&self) -> Result<VaultStatus, CredentialError> {
        let exists = VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .is_some();
        let has_credentials = CredentialEntity::find().one(&self.db).await?.is_some();
        let state = if !exists {
            VaultState::NeedsSetup
        } else if self.session.is_unlocked() {
            VaultState::Unlocked
        } else {
            VaultState::Locked
        };
        Ok(VaultStatus {
            state,
            has_credentials,
        })
    }

    /// Reads MK from the OS keychain (today's only source of it) and runs
    /// `setup_with_master_key`. The one real caller; tests use the latter
    /// directly so they never touch the OS keychain.
    pub async fn setup(&self, password: &str) -> Result<String, CredentialError> {
        let mk = Self::read_keychain_master_key()?;
        self.setup_with_master_key(mk, password).await
    }

    pub async fn setup_with_master_key(
        &self,
        mk: [u8; 32],
        password: &str,
    ) -> Result<String, CredentialError> {
        if password.len() < 8 {
            return Err(CredentialError::InvalidMasterPassword);
        }
        if VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(CredentialError::VaultAlreadySetUp);
        }

        let recovery_bytes = self.encryption.generate_bytes(RECOVERY_KEY_BYTES)?;
        let recovery_key = encode_recovery_key(&recovery_bytes);
        let mk_b64 = general_purpose::STANDARD.encode(mk);

        let pw_salt = self.encryption.generate_salt()?;
        let pw_key = self
            .encryption
            .derive_key(password, &pw_salt, PBKDF2_ITERATIONS)?;
        let pw_wrapped = self.encryption.encrypt(&mk_b64, &pw_key)?;

        let recovery_salt = self.encryption.generate_salt()?;
        let recovery_wrap_key =
            self.encryption
                .hkdf_key(&recovery_bytes, &recovery_salt, RECOVERY_HKDF_INFO)?;
        let recovery_wrapped = self.encryption.encrypt(&mk_b64, &recovery_wrap_key)?;

        let now = Utc::now().to_rfc3339();
        let active = VaultActive {
            id: Set(VAULT_ID),
            pw_salt: Set(general_purpose::STANDARD.encode(pw_salt)),
            pw_iterations: Set(PBKDF2_ITERATIONS as i32),
            pw_wrapped_mk: Set(serde_json::to_string(&pw_wrapped)?),
            recovery_salt: Set(general_purpose::STANDARD.encode(recovery_salt)),
            recovery_wrapped_mk: Set(serde_json::to_string(&recovery_wrapped)?),
            created_at: Set(now.clone()),
            updated_at: Set(now),
        };
        active.insert(&self.db).await?;

        self.session.unlock(mk);
        Ok(recovery_key)
    }

    pub async fn unlock_with_password(&self, password: &str) -> Result<(), CredentialError> {
        let row = VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .ok_or(CredentialError::VaultNotSetUp)?;

        let salt = general_purpose::STANDARD
            .decode(&row.pw_salt)
            .map_err(|e| CredentialError::DeserializationError(e.to_string()))?;
        let pw_key = self
            .encryption
            .derive_key(password, &salt, row.pw_iterations as u32)?;

        let wrapped: EncryptionResult = serde_json::from_str(&row.pw_wrapped_mk)?;
        let mk_b64 = self
            .encryption
            .decrypt(DecryptionRequest {
                encrypted: wrapped.encrypted,
                iv: wrapped.iv,
                tag: wrapped.tag,
                algorithm: wrapped.algorithm,
                key: pw_key,
            })
            .map_err(|_| CredentialError::InvalidMasterPassword)?;

        self.session.unlock(decode_mk(&mk_b64)?);
        Ok(())
    }

    /// Caller must already have Windows Hello consent before calling this —
    /// it only reads the keychain and unlocks the session. Sync on purpose:
    /// portal-core never depends on an async runtime from the host (tauri),
    /// so the one DB read here blocks on `futures::executor::block_on`.
    pub fn unlock_with_keychain(&self) -> Result<(), CredentialError> {
        let exists =
            futures::executor::block_on(VaultEntity::find_by_id(VAULT_ID).one(&self.db))?.is_some();
        if !exists {
            return Err(CredentialError::VaultNotSetUp);
        }
        self.session.unlock(Self::read_keychain_master_key()?);
        Ok(())
    }

    pub fn lock(&self) {
        self.session.lock();
    }

    /// Unwraps MK with the recovery key, then re-wraps it under a fresh
    /// password salt. `recovery_wrapped_mk` is never touched, so the same
    /// recovery key keeps working after this.
    pub async fn reset_password(
        &self,
        recovery_key: &str,
        new_password: &str,
    ) -> Result<(), CredentialError> {
        if new_password.len() < 8 {
            return Err(CredentialError::InvalidMasterPassword);
        }
        let row = VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .ok_or(CredentialError::VaultNotSetUp)?;

        let recovery_bytes = decode_recovery_key(recovery_key)?;
        let recovery_salt = general_purpose::STANDARD
            .decode(&row.recovery_salt)
            .map_err(|e| CredentialError::DeserializationError(e.to_string()))?;
        let recovery_wrap_key =
            self.encryption
                .hkdf_key(&recovery_bytes, &recovery_salt, RECOVERY_HKDF_INFO)?;

        let wrapped: EncryptionResult = serde_json::from_str(&row.recovery_wrapped_mk)?;
        let mk_b64 = self
            .encryption
            .decrypt(DecryptionRequest {
                encrypted: wrapped.encrypted,
                iv: wrapped.iv,
                tag: wrapped.tag,
                algorithm: wrapped.algorithm,
                key: recovery_wrap_key,
            })
            .map_err(|_| CredentialError::InvalidRecoveryKey)?;
        let mk = decode_mk(&mk_b64)?;

        let new_salt = self.encryption.generate_salt()?;
        let new_pw_key = self
            .encryption
            .derive_key(new_password, &new_salt, PBKDF2_ITERATIONS)?;
        let new_wrapped = self.encryption.encrypt(&mk_b64, &new_pw_key)?;

        let active = VaultActive {
            id: Set(row.id),
            pw_salt: Set(general_purpose::STANDARD.encode(new_salt)),
            pw_iterations: Set(PBKDF2_ITERATIONS as i32),
            pw_wrapped_mk: Set(serde_json::to_string(&new_wrapped)?),
            recovery_salt: Set(row.recovery_salt),
            recovery_wrapped_mk: Set(row.recovery_wrapped_mk),
            created_at: Set(row.created_at),
            updated_at: Set(Utc::now().to_rfc3339()),
        };
        active.update(&self.db).await?;

        self.session.unlock(mk);
        Ok(())
    }

    /// Read the raw master key from the OS keychain, minting one on first
    /// run. Relocated from the old `CredentialService::get_master_key` —
    /// `setup` is now its only caller, since MK never changes after setup.
    fn read_keychain_master_key() -> Result<[u8; 32], CredentialError> {
        const SERVICE: &str = crate::app_paths::APP_IDENTIFIER;
        const KEY_NAME: &str = "credential-master-key-v1";

        let entry = keyring::Entry::new(SERVICE, KEY_NAME).map_err(|e| {
            CredentialError::EncryptionFailed(format!("Keychain access failed: {}", e))
        })?;

        match entry.get_secret() {
            Ok(bytes) if bytes.len() == 32 => {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(key)
            }
            Ok(_) => Err(CredentialError::EncryptionFailed(
                "Stored master key has unexpected length".to_string(),
            )),
            Err(keyring::Error::NoEntry) => {
                // No entry under the current service name: either a genuine first run,
                // or an install that predates the bundle-identifier rename. Carry the
                // old key across when there is one, so existing credentials stay
                // decryptable; otherwise mint a fresh random key.
                let key = Self::legacy_master_key(KEY_NAME).unwrap_or_else(|| {
                    use rand::RngCore;
                    let mut key = [0u8; 32];
                    rand::rngs::OsRng.fill_bytes(&mut key);
                    key
                });
                entry.set_secret(&key).map_err(|e| {
                    CredentialError::EncryptionFailed(format!(
                        "Failed to store master key in keychain: {}",
                        e
                    ))
                })?;
                Ok(key)
            }
            Err(e) => Err(CredentialError::EncryptionFailed(format!(
                "Failed to read master key from keychain: {}",
                e
            ))),
        }
    }

    /// Master key stored under a pre-rename bundle identifier, newest first.
    /// Drop an identifier once no install remains on it.
    fn legacy_master_key(key_name: &str) -> Option<[u8; 32]> {
        crate::app_paths::LEGACY_APP_IDENTIFIERS
            .iter()
            .find_map(|identifier| {
                keyring::Entry::new(identifier, key_name)
                    .ok()?
                    .get_secret()
                    .ok()?
                    .try_into()
                    .ok()
            })
    }
}

/// 20 random bytes, base32-encoded (no padding — 160 bits divides evenly
/// into 32 symbols) and grouped into 8 blocks of 4 for readability.
fn encode_recovery_key(bytes: &[u8]) -> String {
    BASE32_NOPAD
        .encode(bytes)
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

/// Strips everything but alphanumerics and uppercases before decoding, so
/// pasted whitespace or dashes don't matter.
fn decode_recovery_key(input: &str) -> Result<Vec<u8>, CredentialError> {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    BASE32_NOPAD
        .decode(cleaned.as_bytes())
        .map_err(|_| CredentialError::InvalidRecoveryKey)
}

fn decode_mk(mk_b64: &str) -> Result<[u8; 32], CredentialError> {
    let bytes = general_purpose::STANDARD
        .decode(mk_b64)
        .map_err(|e| CredentialError::DeserializationError(e.to_string()))?;
    bytes.try_into().map_err(|_| {
        CredentialError::DeserializationError("master key has unexpected length".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::credentials::services::credential_service::{
        CredentialCreateRequest, CredentialService,
    };
    use crate::migrations::runner::run_migrations;
    use sea_orm::Database;

    async fn test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.expect("connect");
        run_migrations(&conn).await.expect("migrate");
        conn
    }

    #[tokio::test]
    async fn password_reset_rewraps_mk_and_keeps_existing_credentials_decryptable() {
        let db = test_db().await;
        let session = Arc::new(VaultSession::default());
        let vault = VaultService::new(db.clone(), session.clone());

        let mk = [7u8; 32];
        let recovery = vault
            .setup_with_master_key(mk, "password-one")
            .await
            .expect("setup");
        assert_eq!(recovery.split('-').count(), 8);

        // Encrypted under MK before the reset.
        let credentials = CredentialService::new(db.clone(), session.clone());
        let created = credentials
            .create_credential(CredentialCreateRequest {
                name: "test".into(),
                credential_type: "password".into(),
                description: None,
                tags: None,
                value: "super-secret".into(),
                fields: None,
                metadata: None,
                expires_at: None,
            })
            .await
            .expect("create credential");

        vault.lock();
        assert!(matches!(
            vault.unlock_with_password("wrong-password").await,
            Err(CredentialError::InvalidMasterPassword)
        ));
        vault
            .unlock_with_password("password-one")
            .await
            .expect("unlock with original password");

        vault
            .reset_password(&recovery, "password-two")
            .await
            .expect("reset password");

        assert!(matches!(
            vault.unlock_with_password("password-one").await,
            Err(CredentialError::InvalidMasterPassword)
        ));
        vault
            .unlock_with_password("password-two")
            .await
            .expect("unlock with new password");

        // Still decrypts after the reset — same MK.
        let decrypted = credentials
            .decrypt_credential(&created.id)
            .await
            .expect("decrypt after reset");
        assert_eq!(decrypted, "super-secret");
    }

    #[test]
    fn hkdf_and_pbkdf2_wrap_unwrap_round_trip() {
        let encryption = EncryptionService::new();
        let mk = [9u8; 32];
        let mk_b64 = general_purpose::STANDARD.encode(mk);

        let pw_salt = encryption.generate_salt().expect("salt");
        let pw_key = encryption
            .derive_key("a-password", &pw_salt, 10_000)
            .expect("derive");
        let pw_wrapped = encryption.encrypt(&mk_b64, &pw_key).expect("encrypt");
        let unwrapped = encryption
            .decrypt(DecryptionRequest {
                encrypted: pw_wrapped.encrypted,
                iv: pw_wrapped.iv,
                tag: pw_wrapped.tag,
                algorithm: pw_wrapped.algorithm,
                key: pw_key,
            })
            .expect("decrypt");
        assert_eq!(decode_mk(&unwrapped).expect("decode"), mk);

        let recovery_bytes = encryption.generate_bytes(20).expect("recovery bytes");
        let recovery_salt = encryption.generate_salt().expect("salt");
        let wrap_key = encryption
            .hkdf_key(&recovery_bytes, &recovery_salt, RECOVERY_HKDF_INFO)
            .expect("hkdf");
        let recovery_wrapped = encryption.encrypt(&mk_b64, &wrap_key).expect("encrypt");
        let unwrapped = encryption
            .decrypt(DecryptionRequest {
                encrypted: recovery_wrapped.encrypted,
                iv: recovery_wrapped.iv,
                tag: recovery_wrapped.tag,
                algorithm: recovery_wrapped.algorithm,
                key: wrap_key,
            })
            .expect("decrypt");
        assert_eq!(decode_mk(&unwrapped).expect("decode"), mk);
    }

    #[test]
    fn recovery_key_round_trips_through_formatting_and_pasted_whitespace() {
        let bytes: Vec<u8> = (0..20).collect();
        let formatted = encode_recovery_key(&bytes);
        assert_eq!(formatted.matches('-').count(), 7);

        let pasted = format!("  {}  ", formatted.replace('-', " - "));
        assert_eq!(decode_recovery_key(&pasted).expect("decode"), bytes);
    }
}
