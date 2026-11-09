/**
 * Vault Session - in-memory holder for the unlocked master key (MK).
 *
 * Nothing here ever touches disk: a lock, a restart, or 15 idle minutes
 * since the last credential use all mean the key has to come back through
 * a password, the recovery key, or the OS keychain.
 */
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::super::CredentialError;

const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Default)]
pub struct VaultSession {
    inner: Mutex<Option<([u8; 32], Instant)>>,
}

impl VaultSession {
    pub fn unlock(&self, mk: [u8; 32]) {
        *self.inner.lock().unwrap() = Some((mk, Instant::now()));
    }

    /// Best-effort: overwrite the key bytes before dropping them.
    /// ponytail: no `zeroize` dependency for this — doesn't stop the
    /// compiler eliding the write or a swapped/paged-out copy surviving;
    /// add `zeroize` if that threat model ever matters here.
    pub fn lock(&self) {
        if let Some((mut mk, _)) = self.inner.lock().unwrap().take() {
            mk.fill(0);
        }
    }

    /// Returns the master key if unlocked and not idle-expired, and slides
    /// the idle timer forward. Locks (and reports locked) if the session
    /// had expired.
    pub fn key(&self) -> Result<[u8; 32], CredentialError> {
        let mut guard = self.inner.lock().unwrap();
        match *guard {
            Some((mk, last_used)) if last_used.elapsed() < IDLE_TIMEOUT => {
                *guard = Some((mk, Instant::now()));
                Ok(mk)
            }
            Some(_) => {
                *guard = None;
                Err(CredentialError::VaultLocked)
            }
            None => Err(CredentialError::VaultLocked),
        }
    }

    /// Status check only — does not slide the idle timer. Still locks the
    /// session if it had idle-expired, same as `key`.
    pub fn is_unlocked(&self) -> bool {
        let mut guard = self.inner.lock().unwrap();
        match *guard {
            Some((_, last_used)) if last_used.elapsed() < IDLE_TIMEOUT => true,
            Some(_) => {
                *guard = None;
                false
            }
            None => false,
        }
    }
}
