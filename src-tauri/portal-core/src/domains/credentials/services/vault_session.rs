/**
 * Vault Session - in-memory holder for the unlocked master key (MK).
 *
 * Neither the key nor the password is ever written by this module. What persists
 * is the user's `LockPolicy` and a marker saying "unlocked when last seen, in OS
 * session X"; if the policy allows it at startup, the shell re-reads the key from
 * the OS keychain (`VaultService::unlock_with_keychain`). Any lock removes the
 * marker. The idle timeout is enforced by the shell, which can see OS input.
 */
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

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

const POLICY_FILE: &str = "vault_lock_policy.json";

/// How long an unlock outlives the app.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeepUnlocked {
    /// Closing the app locks the vault.
    #[default]
    AppClose,
    /// Survives app restarts until the machine restarts or the user signs out.
    SystemRestart,
    /// Survives everything until the user locks it.
    UntilLocked,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockPolicy {
    pub keep_unlocked: KeepUnlocked,
    /// Lock after this many idle minutes while the app runs; 0 = never.
    pub idle_minutes: u32,
}

/// Identifies the running OS session, so `SystemRestart` can tell it ended.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsSession {
    pub id: String,
    pub boot_time: u64,
}

impl OsSession {
    /// Boot time is derived from the clock on some platforms, so allow it to wobble.
    fn same_as(&self, other: &OsSession) -> bool {
        self.id == other.id && self.boot_time.abs_diff(other.boot_time) <= 120
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Stored {
    #[serde(default)]
    policy: LockPolicy,
    #[serde(default)]
    unlocked_in: Option<OsSession>,
}

#[derive(Default)]
pub struct VaultSession {
    inner: Mutex<Option<[u8; 32]>>,
    prompter: OnceLock<Arc<dyn VaultPrompter>>,
    persistence: OnceLock<(PathBuf, OsSession)>,
    policy: Mutex<LockPolicy>,
    last_used: Mutex<Option<Instant>>,
}

impl std::fmt::Debug for VaultSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultSession").field("unlocked", &self.is_unlocked()).finish()
    }
}

impl VaultSession {
    /// Loads the saved policy. Call once at startup, before any unlock.
    pub fn enable_persistence(&self, data_dir: &Path, os_session: OsSession) {
        let path = data_dir.join(POLICY_FILE);
        *self.policy.lock().unwrap() = read_stored(&path).policy;
        let _ = self.persistence.set((path, os_session));
    }

    pub fn policy(&self) -> LockPolicy {
        *self.policy.lock().unwrap()
    }

    pub fn set_policy(&self, policy: LockPolicy) -> std::io::Result<()> {
        *self.policy.lock().unwrap() = policy;
        self.save(self.is_unlocked())
    }

    /// Whether the saved policy says this launch should come up unlocked.
    pub fn should_restore(&self) -> bool {
        let Some((path, now)) = self.persistence.get() else {
            return false;
        };
        let Some(then) = read_stored(path).unlocked_in else {
            return false;
        };
        match self.policy().keep_unlocked {
            KeepUnlocked::AppClose => false,
            KeepUnlocked::SystemRestart => then.same_as(now),
            KeepUnlocked::UntilLocked => true,
        }
    }

    /// Time since the key was last unlocked or read.
    pub fn since_last_use(&self) -> Duration {
        self.last_used.lock().unwrap().map(|t| t.elapsed()).unwrap_or_default()
    }

    fn save(&self, unlocked: bool) -> std::io::Result<()> {
        let Some((path, os_session)) = self.persistence.get() else {
            return Ok(());
        };
        let stored = Stored {
            policy: self.policy(),
            unlocked_in: unlocked.then(|| os_session.clone()),
        };
        std::fs::write(path, serde_json::to_vec(&stored)?)
    }

    pub fn unlock(&self, mk: [u8; 32]) {
        *self.inner.lock().unwrap() = Some(mk);
        *self.last_used.lock().unwrap() = Some(Instant::now());
        // Failing to write only means the next launch asks again.
        let _ = self.save(true);
    }

    /// Best-effort: overwrite the key bytes before dropping them.
    /// ponytail: no `zeroize` dependency for this — doesn't stop the
    /// compiler eliding the write or a swapped/paged-out copy surviving;
    /// add `zeroize` if that threat model ever matters here.
    pub fn lock(&self) {
        if let Some(mut mk) = self.inner.lock().unwrap().take() {
            mk.fill(0);
        }
        // A marker that survives a lock would unlock the next launch: if it
        // can't be rewritten, drop the whole file (policy falls back to default).
        if self.save(false).is_err() {
            if let Some((path, _)) = self.persistence.get() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    pub fn key(&self) -> Result<[u8; 32], CredentialError> {
        let key = self.peek_key()?;
        *self.last_used.lock().unwrap() = Some(Instant::now());
        Ok(key)
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
        (*self.inner.lock().unwrap()).ok_or(CredentialError::VaultLocked)
    }

    pub fn is_unlocked(&self) -> bool {
        self.inner.lock().unwrap().is_some()
    }
}

/// Missing or unreadable → default policy and no marker, i.e. locked.
fn read_stored(path: &Path) -> Stored {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
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

    #[test]
    fn restore_follows_policy_os_session_and_explicit_lock() {
        let dir = tempfile::tempdir().unwrap();
        let boot = |id: &str, boot_time| OsSession { id: id.into(), boot_time };
        let launch = |os: OsSession| {
            let s = VaultSession::default();
            s.enable_persistence(dir.path(), os);
            s
        };
        let policy = |keep_unlocked| LockPolicy { keep_unlocked, idle_minutes: 5 };

        let s = launch(boot("a", 1000));
        s.unlock([1; 32]);
        assert!(!launch(boot("a", 1000)).should_restore()); // default: lock on close

        s.set_policy(policy(KeepUnlocked::SystemRestart)).unwrap();
        assert_eq!(launch(boot("a", 1000)).policy(), policy(KeepUnlocked::SystemRestart));
        assert!(launch(boot("a", 1001)).should_restore()); // clock wobble
        assert!(!launch(boot("a", 9000)).should_restore()); // rebooted
        assert!(!launch(boot("b", 1000)).should_restore()); // signed out and in

        s.set_policy(policy(KeepUnlocked::UntilLocked)).unwrap();
        assert!(launch(boot("b", 9000)).should_restore());

        s.lock();
        assert!(!launch(boot("a", 1000)).should_restore());
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
