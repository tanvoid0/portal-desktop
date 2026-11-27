//! Pure pieces of owning `agent-platformd`: where its data lives, its master
//! key, the environment it is spawned with, and the attach/spawn/foreign
//! decision. Ported from agent-platform's `desktop/crates/app/src/shell.rs`.
//! No Tauri here — the host (`src-tauri`) does the actual spawn and kill.
//!
//! The daemon reads everything from env and has no args. An unset master key
//! degrades it to an OPEN server, so [`daemon_env`] always sets one.

use crate::app_paths::DEV;
use rand::RngCore;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Port of a plain-http loopback base URL with an explicit port: the only
/// addresses portal starts a daemon on, or sends the install key to unasked.
pub fn local_port(base: &str) -> Option<u16> {
    let rest = base.trim().strip_prefix("http://")?.trim_end_matches('/');
    let (host, port) = rest.rsplit_once(':')?;
    if host != "127.0.0.1" && host != "localhost" {
        return None;
    }
    port.parse().ok().filter(|p| *p != 0)
}

/// Agent-platform's own `APP_DIR`: reusing it verbatim is what adopts the
/// user's existing database, key, workspaces and tokens.
const APP_DIR: &str = "com.tanvoid0.agentplatform";

#[cfg(windows)]
pub const DAEMON_EXE: &str = "agent-platformd.exe";
#[cfg(not(windows))]
pub const DAEMON_EXE: &str = "agent-platformd";

/// What `/health` and a keyed status request told us about the port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortProbe {
    Unreachable,
    /// Healthy and accepts our master key (or is an open pre-ADR-0019 daemon).
    AcceptsKey,
    /// Healthy but rejects our key: someone else's server (ADR 0020).
    RejectsKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Spawn,
    Attach,
    Foreign,
}

pub fn decide(probe: PortProbe) -> Action {
    match probe {
        PortProbe::Unreachable => Action::Spawn,
        PortProbe::AcceptsKey => Action::Attach,
        PortProbe::RejectsKey => Action::Foreign,
    }
}

/// `%APPDATA%/com.tanvoid0.agentplatform` in a release build; a `.dev` sibling in
/// a debug build so a dev portal never writes to the release AI database.
/// `AGENT_PLATFORM_APP_DIR` overrides both.
// ponytail: a fresh `.dev` dir, no first-launch copy from the release dir like
// `app_paths::seed_identifiers` does for portal's own DB. Add if devs want real data.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("AGENT_PLATFORM_APP_DIR") {
        return PathBuf::from(dir);
    }
    let name = if DEV {
        format!("{APP_DIR}.dev")
    } else {
        APP_DIR.to_string()
    };
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(name)
}

/// Replace a file in one step (write beside, fsync, rename): a half-written key
/// reads as a valid-looking wrong one and locks us out of our own data. Readers
/// never see a truncated file either.
pub(crate) fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    // Unique per call: two writers sharing a temp file could publish a half-written one.
    let tmp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// `<dir>/master.key`: reuse a non-empty one (keeps existing tokens valid),
/// else 32 random bytes as lowercase hex.
pub fn load_or_create_key(dir: &Path) -> std::io::Result<String> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("master.key");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let existing = existing.trim();
        if !existing.is_empty() {
            return Ok(existing.to_string());
        }
    }
    let mut raw = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let key: String = raw.iter().map(|b| format!("{b:02x}")).collect();
    write_atomic(&path, &key)?;
    Ok(key)
}

/// The full environment for the daemon. `database_url` is empty for the local
/// SQLite file: present-but-empty is deliberate, a developer `.env` only fills
/// gaps and must not point a desktop install at someone's Postgres.
pub fn daemon_env(
    data_dir: &Path,
    key: &str,
    port: u16,
    database_url: &str,
) -> Vec<(String, String)> {
    let p = |name: &str| data_dir.join(name).to_string_lossy().into_owned();
    [
        ("AGENT_PLATFORM_HOST", "127.0.0.1".to_string()),
        ("AGENT_PLATFORM_PORT", port.to_string()),
        ("AGENT_PLATFORM_MASTER_KEY", key.to_string()),
        ("AGENT_PLATFORM_ENV", "development".to_string()),
        ("AGENT_PLATFORM_DB_PATH", p("agent_platform.db")),
        ("AGENT_PLATFORM_WORKSPACE_ROOT", p("workspaces")),
        ("MODEL_OPS_DATA_DIR", p("model-ops")),
        ("MEDIA_DATA_DIR", p("media")),
        ("CONFIG_DIR", p("llm")),
        ("AGENT_PLATFORM_CLOUD_SESSION", p("cloud.session.json")),
        ("DATABASE_URL", database_url.to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// Dir the host registers once at startup: where the bundled sidecar sits.
static SIDECAR_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_sidecar_dir(dir: PathBuf) {
    let _ = SIDECAR_DIR.set(dir);
}

/// Binary to spawn: explicit env override, then the bundled sidecar, then a
/// locally built `agent-platformd` (dev, no network fetch needed).
pub fn daemon_exe() -> Option<PathBuf> {
    resolve_exe(
        std::env::var_os("AGENT_PLATFORM_DAEMON_EXE").map(PathBuf::from),
        SIDECAR_DIR.get().map(|d| d.join(DAEMON_EXE)),
        dev_build_exe(),
    )
}

fn resolve_exe(
    env: Option<PathBuf>,
    sidecar: Option<PathBuf>,
    dev: Option<PathBuf>,
) -> Option<PathBuf> {
    // An explicit override that points nowhere is an error to surface, not a
    // reason to silently run some other binary.
    if let Some(p) = env {
        return p.is_file().then_some(p);
    }
    [sidecar, dev].into_iter().flatten().find(|p| p.is_file())
}

/// `../../ai/agentic-ai/agent-platform/target/debug/agent-platformd` relative to
/// this repo's checkout: dev builds only, release never guesses sibling repos.
fn dev_build_exe() -> Option<PathBuf> {
    if !DEV {
        return None;
    }
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2)?;
    let root = repo
        .parent()?
        .parent()?
        .join("ai/agentic-ai/agent-platform/target");
    ["debug", "release"]
        .iter()
        .map(|p| root.join(p).join(DAEMON_EXE))
        .find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(env: &'a [(String, String)], k: &str) -> &'a str {
        &env.iter().find(|(n, _)| n == k).unwrap().1
    }

    #[test]
    fn env_points_everything_under_the_data_dir_and_always_sets_a_key() {
        let dir = Path::new("data");
        let env = daemon_env(dir, "k", 18410, "");
        assert_eq!(get(&env, "AGENT_PLATFORM_HOST"), "127.0.0.1");
        assert_eq!(get(&env, "AGENT_PLATFORM_PORT"), "18410");
        assert_eq!(get(&env, "AGENT_PLATFORM_MASTER_KEY"), "k");
        assert_eq!(get(&env, "DATABASE_URL"), "");
        for k in [
            "AGENT_PLATFORM_DB_PATH",
            "AGENT_PLATFORM_WORKSPACE_ROOT",
            "MODEL_OPS_DATA_DIR",
            "MEDIA_DATA_DIR",
            "CONFIG_DIR",
            "AGENT_PLATFORM_CLOUD_SESSION",
        ] {
            assert!(Path::new(get(&env, k)).starts_with(dir), "{k}");
        }
        assert!(get(&env, "AGENT_PLATFORM_CLOUD_SESSION").ends_with("cloud.session.json"));
    }

    #[test]
    fn key_is_created_once_then_reused() {
        let dir = tempfile::tempdir().unwrap();
        let a = load_or_create_key(dir.path()).unwrap();
        assert_eq!(a.len(), 64);
        assert!(a
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
        assert_eq!(load_or_create_key(dir.path()).unwrap(), a);

        // An existing key (the user's current install) is adopted, trimmed.
        std::fs::write(dir.path().join("master.key"), " existing\n").unwrap();
        assert_eq!(load_or_create_key(dir.path()).unwrap(), "existing");

        // Empty file is regenerated, not adopted.
        std::fs::write(dir.path().join("master.key"), "").unwrap();
        assert_eq!(load_or_create_key(dir.path()).unwrap().len(), 64);
    }

    #[test]
    fn only_loopback_http_with_a_port_is_local() {
        assert_eq!(local_port("http://127.0.0.1:18410"), Some(18410));
        assert_eq!(local_port("http://localhost:19000/"), Some(19000));
        for other in [
            "http://192.168.1.5:18410",
            "https://127.0.0.1:18410",
            "http://127.0.0.1",
            "http://127.0.0.1:0",
            "http://127.0.0.1:99999",
            "http://127.0.0.1:1@evil.test:80",
            "http://evil.test/127.0.0.1:18410",
        ] {
            assert_eq!(local_port(other), None, "{other}");
        }
    }

    #[test]
    fn attach_spawn_or_foreign() {
        assert_eq!(decide(PortProbe::Unreachable), Action::Spawn);
        assert_eq!(decide(PortProbe::AcceptsKey), Action::Attach);
        assert_eq!(decide(PortProbe::RejectsKey), Action::Foreign);
    }

    #[test]
    fn exe_resolution_order() {
        let dir = tempfile::tempdir().unwrap();
        let f = |n: &str| {
            let p = dir.path().join(n);
            std::fs::write(&p, "").unwrap();
            p
        };
        let (env, side, dev) = (f("env"), f("side"), f("dev"));
        let missing = dir.path().join("nope");
        assert_eq!(
            resolve_exe(Some(env.clone()), Some(side.clone()), Some(dev.clone())),
            Some(env)
        );
        assert_eq!(
            resolve_exe(None, Some(side.clone()), Some(dev.clone())),
            Some(side)
        );
        assert_eq!(
            resolve_exe(None, Some(missing.clone()), Some(dev.clone())),
            Some(dev)
        );
        // A bad override does not fall through to another binary.
        assert_eq!(
            resolve_exe(Some(missing.clone()), Some(f("s2")), None),
            None
        );
        assert_eq!(resolve_exe(None, Some(missing), None), None);
    }
}
