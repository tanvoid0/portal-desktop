/**
 * Vault Session - in-memory holder for the unlocked master key (MK).
 *
 * Nothing here ever touches disk: an explicit lock or a restart means the key
 * has to come back through a password, the recovery key, or the OS keychain.
 * There is deliberately no idle timeout — the user unlocks once per run and the
 * app keeps working; "Lock" in the UI is the way to drop the key early.
 */
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

use super::super::CredentialError;

/// Asks the user to unlock the vault. Resolves `true` once they did, `false`
/// if they dismissed the prompt (or nobody could answer it). Implemented by
/// the app shell; core stays UI-agnostic.
pub trait VaultPrompter: Send + Sync {
    fn prompt(&self) -> Pin<Box<dyn Future<Output = bool> + Send>>;
}

tokio::task_local! {
    /// Set by the daemon's /rpc handler around remote-trust dispatches; makes `require_key`
    /// skip the prompt. Lost across `tokio::spawn`: re-enter the scope inside any spawned task that can reach a
    /// vault-gated call (see `CoderService::spawn_run`).
    pub static REMOTE_CALLER: ();
}

#[derive(Default)]
pub struct VaultSession {
    inner: Mutex<Option<[u8; 32]>>,
    prompter: OnceLock<Arc<dyn VaultPrompter>>,
}

impl std::fmt::Debug for VaultSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultSession").field("unlocked", &self.is_unlocked()).finish()
    }
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

    /// Wire the unlock prompt. Set once at startup; without one, `require_key`
    /// behaves exactly like `key`.
    pub fn set_prompter(&self, prompter: Arc<dyn VaultPrompter>) {
        let _ = self.prompter.set(prompter);
    }

    /// The gate for any real credential use. Locked → ask the user to unlock,
    /// then continue; dismissed (or no prompter) → `VaultLocked`, so the caller
    /// fails cleanly. Use this, not `key()`, wherever a command needs the key —
    /// it is what makes a new vault-touching command prompt with no extra wiring.
    pub async fn require_key(&self) -> Result<[u8; 32], CredentialError> {
        if let Ok(key) = self.key() {
            return Ok(key);
        }
        // A remote (LAN) caller must not pop an unlock dialog on a desk nobody may be at:
        // fail fast like `key()`.
        if REMOTE_CALLER.try_with(|_| ()).is_ok() {
            return Err(CredentialError::VaultLocked);
        }
        if let Some(p) = self.prompter.get() {
            // Whatever the prompt said (even a timeout after a lost event), the
            // key is the truth: unlocked meanwhile → continue.
            p.prompt().await;
            return self.key();
        }
        Err(CredentialError::VaultLocked)
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Stub(Option<Arc<VaultSession>>);
    impl VaultPrompter for Stub {
        fn prompt(&self) -> Pin<Box<dyn Future<Output = bool> + Send>> {
            let unlock = self.0.clone();
            Box::pin(async move {
                match unlock {
                    Some(s) => {
                        s.unlock([7; 32]);
                        true
                    }
                    None => false,
                }
            })
        }
    }

    #[tokio::test]
    async fn require_key_prompts_then_continues_or_fails() {
        let none = VaultSession::default();
        assert!(none.require_key().await.is_err()); // no prompter

        let declined = VaultSession::default();
        declined.set_prompter(Arc::new(Stub(None)));
        assert!(declined.require_key().await.is_err()); // user cancelled

        let s = Arc::new(VaultSession::default());
        s.set_prompter(Arc::new(Stub(Some(s.clone()))));
        assert_eq!(s.require_key().await.unwrap(), [7; 32]); // unlocked → continues

        // Prompt times out (false) but the vault got unlocked meanwhile → still continues.
        struct UnlockThenTimeout(Arc<VaultSession>);
        impl VaultPrompter for UnlockThenTimeout {
            fn prompt(&self) -> Pin<Box<dyn Future<Output = bool> + Send>> {
                self.0.unlock([9; 32]);
                Box::pin(async { false })
            }
        }
        let t = Arc::new(VaultSession::default());
        t.set_prompter(Arc::new(UnlockThenTimeout(t.clone())));
        assert_eq!(t.require_key().await.unwrap(), [9; 32]);

        // Remote caller: no prompt, fails fast while locked.
        let r = Arc::new(VaultSession::default());
        r.set_prompter(Arc::new(Stub(Some(r.clone()))));
        let res = REMOTE_CALLER.scope((), r.require_key()).await;
        assert!(matches!(res, Err(CredentialError::VaultLocked)));
        assert!(r.key().is_err()); // Stub would have unlocked had it been asked
    }
}
