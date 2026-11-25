/**
 * Vault Service - setup, unlock, lock, password change and password reset for
 * the credential master key (MK).
 *
 * MK itself never changes. It is stored wrapped two ways in the single-row
 * `vault` table: under a key derived from the master password (PBKDF2) and
 * under a key derived from the recovery key (HKDF). The OS keychain also
 * keeps a raw copy, gated by a Windows Hello consent check upstream of
 * `unlock_with_keychain`. Because MK never changes, a password change or reset
 * never touches the already-encrypted `credentials` rows.
 */
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use data_encoding::BASE32_NOPAD;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::super::entities::vault::{
    ActiveModel as VaultActive, Entity as VaultEntity, Model as VaultModel,
};
use super::super::entities::Entity as CredentialEntity;
use super::super::CredentialError;
use super::encryption_service::{
    DecryptionRequest, EncryptionResult, EncryptionService, PBKDF2_ITERATIONS,
};
use super::vault_session::VaultSession;

const VAULT_ID: i32 = 1;
const KEYCHAIN_KEY_NAME: &str = "credential-master-key-v1";
/// Shortest master password setup, reset and change accept (the UI forms enforce the same 8).
const MIN_PASSWORD_LEN: usize = 8;
const RECOVERY_KEY_BYTES: usize = 20;
const RECOVERY_HKDF_INFO: &[u8] = b"portal-desktop-vault-recovery";

/// Marker file in the data dir, written when a sync restore swaps in a vault from another
/// device: this device's keychain key no longer matches the vault's master key. Cleared by
/// the next password/recovery unlock, which re-seeds the keychain with the restored key.
pub const KEYCHAIN_STALE_FLAG: &str = "vault_keychain_stale";

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
    data_dir: Option<PathBuf>,
}

impl VaultService {
    pub fn new(db: DatabaseConnection, session: Arc<VaultSession>) -> Self {
        Self {
            db,
            session,
            encryption: EncryptionService::new(),
            data_dir: None,
        }
    }

    /// Enables the post-restore stale-keychain check (see `KEYCHAIN_STALE_FLAG`).
    pub fn with_data_dir(mut self, data_dir: &Path) -> Self {
        self.data_dir = Some(data_dir.to_path_buf());
        self
    }

    fn stale_flag(&self) -> Option<PathBuf> {
        self.data_dir.as_ref().map(|d| d.join(KEYCHAIN_STALE_FLAG))
    }

    /// After a password/recovery unlock, replace a stale keychain key with the real one.
    fn reseed_keychain(&self, mk: &[u8; 32]) {
        let Some(flag) = self.stale_flag().filter(|f| f.exists()) else {
            return;
        };
        let stored = keyring::Entry::new(crate::app_paths::app_identifier(), KEYCHAIN_KEY_NAME)
            .and_then(|e| e.set_secret(mk));
        if stored.is_ok() {
            let _ = std::fs::remove_file(flag);
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
        let credential_count = CredentialEntity::find().all(&self.db).await?.len();
        let mk = Self::read_keychain_master_key(credential_count)?;
        self.setup_with_master_key(mk, password).await
    }

    pub async fn setup_with_master_key(
        &self,
        mk: [u8; 32],
        password: &str,
    ) -> Result<String, CredentialError> {
        if password.len() < MIN_PASSWORD_LEN {
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

        let mk = self.unwrap_mk_with_password(&row, password)?;
        self.reseed_keychain(&mk);
        self.session.unlock(mk);
        Ok(())
    }

    /// Unwraps MK with the master password, using the salt and iteration count stored in
    /// `row`. Shared by unlock and password change so both verify a password the same way:
    /// a wrong one fails the AEAD tag check and surfaces as `InvalidMasterPassword`.
    fn unwrap_mk_with_password(
        &self,
        row: &VaultModel,
        password: &str,
    ) -> Result<[u8; 32], CredentialError> {
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

        decode_mk(&mk_b64)
    }

    /// Caller must already have Windows Hello consent before calling this —
    /// it only reads the keychain and unlocks the session. Async: blocking a
    /// tokio worker on the DB reads (`futures::executor::block_on`) could hang.
    pub async fn unlock_with_keychain(&self) -> Result<(), CredentialError> {
        if self.stale_flag().is_some_and(|f| f.exists()) {
            return Err(CredentialError::EncryptionFailed(
                "this vault was restored from another device; unlock with your vault password once"
                    .to_string(),
            ));
        }
        let exists = VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .is_some();
        if !exists {
            return Err(CredentialError::VaultNotSetUp);
        }
        let credential_count = CredentialEntity::find().all(&self.db).await?.len();
        self.session
            .unlock(Self::read_keychain_master_key(credential_count)?);
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
        if new_password.len() < MIN_PASSWORD_LEN {
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

        self.rewrap_under_password(row, mk, new_password).await
    }

    /// Rotates the master password: verifies `old_password` exactly like
    /// `unlock_with_password` (a wrong one is `InvalidMasterPassword` and nothing is written),
    /// then re-wraps the SAME MK under `new_password`. MK is unchanged, so the `credentials`
    /// rows are not re-encrypted: unwrapping MK with the old password and wrapping it with
    /// the new one is the "decrypt with old, encrypt with new" step. `recovery_*` is
    /// untouched, so the recovery key keeps working. Leaves the session unlocked.
    pub async fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), CredentialError> {
        if new_password.len() < MIN_PASSWORD_LEN || new_password == old_password {
            return Err(CredentialError::InvalidMasterPassword);
        }
        let row = VaultEntity::find_by_id(VAULT_ID)
            .one(&self.db)
            .await?
            .ok_or(CredentialError::VaultNotSetUp)?;

        let mk = self.unwrap_mk_with_password(&row, old_password)?;
        self.rewrap_under_password(row, mk, new_password).await
    }

    /// Wraps MK under `new_password` with a fresh salt and the current iteration count,
    /// writes the new password fields in one update (`recovery_*` carried over as is), then
    /// re-seeds the keychain and unlocks the session. Shared by reset and change.
    async fn rewrap_under_password(
        &self,
        row: VaultModel,
        mk: [u8; 32],
        new_password: &str,
    ) -> Result<(), CredentialError> {
        let new_salt = self.encryption.generate_salt()?;
        let new_pw_key = self
            .encryption
            .derive_key(new_password, &new_salt, PBKDF2_ITERATIONS)?;
        let new_wrapped = self
            .encryption
            .encrypt(&general_purpose::STANDARD.encode(mk), &new_pw_key)?;

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

        self.reseed_keychain(&mk);
        self.session.unlock(mk);
        Ok(())
    }

    /// Read the raw master key from the OS keychain, minting one on first
    /// run. Relocated from the old `CredentialService::get_master_key` —
    /// `setup` is now its only caller, since MK never changes after setup.
    ///
    /// `credential_count` guards against silently orphaning existing encrypted rows: it is
    /// only safe to mint a brand new random key when there is nothing encrypted under the
    /// old one to lose (see QA-049).
    fn read_keychain_master_key(credential_count: usize) -> Result<[u8; 32], CredentialError> {
        const KEY_NAME: &str = KEYCHAIN_KEY_NAME;

        // A dev build keeps its own entry, so it can never overwrite the release's key.
        let entry =
            keyring::Entry::new(crate::app_paths::app_identifier(), KEY_NAME).map_err(|e| {
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
                // decryptable.
                if let Some(key) = Self::legacy_master_key(KEY_NAME) {
                    entry.set_secret(&key).map_err(|e| {
                        CredentialError::EncryptionFailed(format!(
                            "Failed to store master key in keychain: {}",
                            e
                        ))
                    })?;
                    return Ok(key);
                }

                // No legacy key either. Minting a fresh one is only safe when nothing is
                // already encrypted under whatever key used to live here — otherwise this
                // permanently orphans every existing credential with no warning.
                if credential_count > 0 {
                    return Err(CredentialError::MasterKeyMissing(credential_count));
                }

                use rand::RngCore;
                let mut key = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut key);
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

    /// Master key stored under a seed identifier, newest first: the release's for a
    /// dev build (its database is a copy of the release's, encrypted under that key),
    /// then the pre-rename ones. Drop a legacy identifier once no install remains on it.
    fn legacy_master_key(key_name: &str) -> Option<[u8; 32]> {
        crate::app_paths::seed_identifiers().find_map(|identifier| {
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

    async fn vault_row(db: &DatabaseConnection) -> VaultModel {
        VaultEntity::find_by_id(VAULT_ID)
            .one(db)
            .await
            .expect("query vault")
            .expect("vault row")
    }

    #[tokio::test]
    async fn password_change_rewraps_same_mk_and_keeps_credentials_and_recovery_working() {
        let db = test_db().await;
        let session = Arc::new(VaultSession::default());
        let vault = VaultService::new(db.clone(), session.clone());

        let recovery = vault
            .setup_with_master_key([7u8; 32], "password-one")
            .await
            .expect("setup");
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
        let before = vault_row(&db).await;

        // Locked on purpose: the old password alone proves the right to rotate, and unlocks.
        vault.lock();
        vault
            .change_password("password-one", "password-two")
            .await
            .expect("change password");
        assert!(session.is_unlocked());

        // Fresh salt and wrap, current iteration count; recovery fields and created_at untouched.
        let after = vault_row(&db).await;
        assert_ne!(after.pw_salt, before.pw_salt);
        assert_ne!(after.pw_wrapped_mk, before.pw_wrapped_mk);
        assert_eq!(after.pw_iterations, PBKDF2_ITERATIONS as i32);
        assert_eq!(after.recovery_salt, before.recovery_salt);
        assert_eq!(after.recovery_wrapped_mk, before.recovery_wrapped_mk);
        assert_eq!(after.created_at, before.created_at);
        assert_ne!(after.updated_at, before.updated_at);

        vault.lock();
        assert!(matches!(
            vault.unlock_with_password("password-one").await,
            Err(CredentialError::InvalidMasterPassword)
        ));
        vault
            .unlock_with_password("password-two")
            .await
            .expect("unlock with new password");

        // Same MK, so the row encrypted before the change still decrypts.
        let decrypted = credentials
            .decrypt_credential(&created.id)
            .await
            .expect("decrypt after change");
        assert_eq!(decrypted, "super-secret");

        // The recovery key still resets the password, and the credential still decrypts.
        vault
            .reset_password(&recovery, "password-three")
            .await
            .expect("reset with the original recovery key");
        vault.lock();
        vault
            .unlock_with_password("password-three")
            .await
            .expect("unlock after reset");
        let decrypted = credentials
            .decrypt_credential(&created.id)
            .await
            .expect("decrypt after reset");
        assert_eq!(decrypted, "super-secret");
    }

    #[tokio::test]
    async fn password_change_with_wrong_old_password_leaves_the_vault_untouched() {
        let db = test_db().await;
        let session = Arc::new(VaultSession::default());
        let vault = VaultService::new(db.clone(), session.clone());
        vault
            .setup_with_master_key([7u8; 32], "password-one")
            .await
            .expect("setup");
        let before = vault_row(&db).await;

        vault.lock();
        assert!(matches!(
            vault
                .change_password("not-the-password", "password-two")
                .await,
            Err(CredentialError::InvalidMasterPassword)
        ));
        assert_eq!(vault_row(&db).await, before);
        assert!(!session.is_unlocked());

        vault
            .unlock_with_password("password-one")
            .await
            .expect("old password still works");
    }

    #[tokio::test]
    async fn password_change_rejects_a_short_or_unchanged_new_password() {
        let db = test_db().await;
        let vault = VaultService::new(db.clone(), Arc::new(VaultSession::default()));
        vault
            .setup_with_master_key([7u8; 32], "password-one")
            .await
            .expect("setup");
        let before = vault_row(&db).await;

        for new_password in ["short", "password-one"] {
            assert!(matches!(
                vault.change_password("password-one", new_password).await,
                Err(CredentialError::InvalidMasterPassword)
            ));
        }
        assert_eq!(vault_row(&db).await, before);
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
