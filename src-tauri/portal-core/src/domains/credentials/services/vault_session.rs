/**
 * Vault Session - in-memory holder for the unlocked master key (MK).
 *
 * Nothing here ever touches disk: an explicit lock or a restart means the key
 * has to come back through a password, the recovery key, or the OS keychain.
 * There is deliberately no idle timeout — the user unlocks once per run and the
 * app keeps working; "Lock" in the UI is the way to drop the key early.
 */
use std::sync::Mutex;

use super::super::CredentialError;

#[derive(Debug, Default)]
pub struct VaultSession {
    inner: Mutex<Option<[u8; 32]>>,
}

impl VaultSession {
    pub fn unlock(&self, mk: [u8; 32]) {
        *self.inner.lock().unwrap() = Some(mk);
    }

    /// Best-effort: overwrite the key bytes before dropping them.
    /// ponytail: no `zeroize` dependency for this — doesn't stop the
    /// compiler eliding the write or a swapped/paged-out copy surviving;
    /// add `zeroize` if that threat model ever matters here.
    pub fn lock(&self) {
        if let Some(mut mk) = self.inner.lock().unwrap().take() {
            mk.fill(0);
        }
    }

    pub fn key(&self) -> Result<[u8; 32], CredentialError> {
        (*self.inner.lock().unwrap()).ok_or(CredentialError::VaultLocked)
    }

    /// Same as `key`; kept so drift-comparison callers read as "not a real
    /// credential use".
    pub fn peek_key(&self) -> Result<[u8; 32], CredentialError> {
        self.key()
    }

    pub fn is_unlocked(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }
}
