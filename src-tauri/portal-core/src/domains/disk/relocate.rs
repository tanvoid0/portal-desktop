//! Move a tool's big data directory to another drive and repoint the tool at it.
//!
//! Table-driven: adding a tool is one `TOOLS` row (env var + default location +
//! the processes that hold the files open). `plan` is read-only and tells you
//! exactly what `apply` would do, so a human or an AI agent can review it first.
//! `apply` refuses while the tool is running — it never kills processes.
//!
//! Repointing = setting a *user* environment variable. Only processes started
//! after the change see it, so the plan says to restart the tool.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domains::disk::dev_cleaners::known_caches::fast_dir_size;

struct Tool {
    id: &'static str,
    label: &'static str,
    /// User env var the tool reads to find its data.
    env_var: &'static str,
    /// Current default location, relative to the home dir.
    default_rel: &'static str,
    /// Appended to the destination to form the env var value. The var points at
    /// the data dir itself for most tools, so this is usually empty.
    value_suffix: &'static str,
    /// Executable names (no `.exe`, lowercase) that must not be running.
    busy: &'static [&'static str],
}

const TOOLS: &[Tool] = &[
    Tool { id: "ollama", label: "Ollama models", env_var: "OLLAMA_MODELS", default_rel: ".ollama\\models", value_suffix: "", busy: &["ollama", "ollama app"] },
    Tool { id: "gradle", label: "Gradle home", env_var: "GRADLE_USER_HOME", default_rel: ".gradle", value_suffix: "", busy: &["java", "gradle"] },
    Tool { id: "android_avd", label: "Android emulators", env_var: "ANDROID_AVD_HOME", default_rel: ".android\\avd", value_suffix: "", busy: &["emulator", "qemu-system-x86_64", "qemu-system-aarch64", "adb"] },
    Tool { id: "npm_cache", label: "npm cache", env_var: "npm_config_cache", default_rel: "AppData\\Local\\npm-cache", value_suffix: "", busy: &["node", "npm"] },
    Tool { id: "cargo_home", label: "Cargo home", env_var: "CARGO_HOME", default_rel: ".cargo", value_suffix: "", busy: &["cargo", "rustc", "rust-analyzer"] },
    Tool { id: "huggingface", label: "Hugging Face cache", env_var: "HF_HOME", default_rel: ".cache\\huggingface", value_suffix: "", busy: &["python", "python3"] },
];

#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct RelocatableTool {
    pub id: String,
    pub label: String,
    pub env_var: String,
    pub source: String,
    pub exists: bool,
    /// What the user env var currently points at (already relocated if set).
    pub current_env: Option<String>,
}

#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct RelocationPlan {
    pub tool_id: String,
    pub source: String,
    pub dest: String,
    pub env_var: String,
    pub env_value: String,
    #[ts(type = "number")]
    pub source_bytes: u64,
    /// Running processes that hold the data open. `apply` refuses while non-empty.
    pub running: Vec<String>,
    /// Why `apply` would refuse, if anything. Empty = ready.
    pub blockers: Vec<String>,
    pub steps: Vec<String>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct RelocationResult {
    pub dest: String,
    pub env_var: String,
    pub env_value: String,
    pub note: String,
}

#[derive(Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/disk/generated/")]
pub struct RelocationRequest {
    pub tool_id: String,
    /// Destination parent, e.g. `I:\ollama`. The data lands in `<target_root>\<default dir name>`.
    pub target_root: String,
}

fn home() -> Result<PathBuf, String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .map_err(|_| "no home directory".to_string())
}

fn tool(id: &str) -> Result<&'static Tool, String> {
    TOOLS.iter().find(|t| t.id == id).ok_or_else(|| format!("unknown tool: {id}"))
}

pub fn list() -> Result<Vec<RelocatableTool>, String> {
    let home = home()?;
    Ok(TOOLS
        .iter()
        .map(|t| {
            let source = home.join(t.default_rel);
            RelocatableTool {
                id: t.id.into(),
                label: t.label.into(),
                env_var: t.env_var.into(),
                exists: source.is_dir(),
                source: source.to_string_lossy().into_owned(),
                current_env: user_env(t.env_var),
            }
        })
        .collect())
}

/// Pure plan construction (no I/O except what the caller passes in) so it is testable.
fn build_plan(
    t: &Tool,
    home: &Path,
    target_root: &Path,
    running: Vec<String>,
    source_bytes: u64,
    current_env: Option<String>,
) -> RelocationPlan {
    let source = home.join(t.default_rel);
    let name = source.file_name().unwrap_or_default();
    let dest = target_root.join(name);
    // `join("")` would append a trailing separator, so only join a real suffix.
    let env_value = if t.value_suffix.is_empty() {
        dest.to_string_lossy().into_owned()
    } else {
        dest.join(t.value_suffix).to_string_lossy().into_owned()
    };

    let mut blockers = Vec::new();
    if let Some(cur) = current_env.filter(|c| !c.is_empty()) {
        blockers.push(format!("{} already points at {cur}; already relocated?", t.env_var));
    }
    if !source.is_dir() {
        blockers.push(format!("source {} does not exist", source.display()));
    }
    if dest.starts_with(&source) {
        blockers.push("destination is inside the source".into());
    }
    if dest.exists() && dest.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
        blockers.push(format!("destination {} is not empty", dest.display()));
    }
    if !running.is_empty() {
        blockers.push(format!("close first: {}", running.join(", ")));
    }

    let steps = vec![
        format!("Move {} -> {} (robocopy /MOVE, per-file verified)", source.display(), dest.display()),
        format!("Set user env var {}={}", t.env_var, env_value),
        format!("Restart {} so it reads the new location", t.label),
    ];

    RelocationPlan {
        tool_id: t.id.into(),
        source: source.to_string_lossy().into_owned(),
        dest: dest.to_string_lossy().into_owned(),
        env_var: t.env_var.into(),
        env_value,
        source_bytes,
        running,
        blockers,
        steps,
    }
}

/// Read-only: what would moving `tool_id` under `target_root` do?
pub fn plan(req: &RelocationRequest) -> Result<RelocationPlan, String> {
    let t = tool(&req.tool_id)?;
    let home = home()?;
    let bytes = fast_dir_size(&home.join(t.default_rel));
    Ok(build_plan(
        t,
        &home,
        Path::new(&req.target_root),
        running_processes(t.busy),
        bytes,
        user_env(t.env_var),
    ))
}

/// Moves the data and sets the env var. Blocking; call from `spawn_blocking`.
pub fn apply(req: &RelocationRequest) -> Result<RelocationResult, String> {
    let p = plan(req)?;
    if !p.blockers.is_empty() {
        return Err(p.blockers.join("; "));
    }
    #[cfg(not(windows))]
    return Err("relocation is only implemented on Windows".into());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        std::fs::create_dir_all(&p.dest).map_err(|e| e.to_string())?;
        // robocopy exit codes 0-7 are success (bit flags); >= 8 is failure.
        let moved = std::process::Command::new("robocopy")
            .arg(&p.source)
            .arg(&p.dest)
            .args(["/E", "/MOVE", "/COPY:DAT", "/R:1", "/W:1", "/NFL", "/NDL", "/NP", "/NJH", "/NJS"])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| e.to_string())?;
        if moved.code().is_none_or(|c| c >= 8) {
            return Err(format!(
                "move failed (robocopy exit {:?}); nothing repointed, partial data may be in {}",
                moved.code(),
                p.dest
            ));
        }
        // Only repoint once the data is really there.
        let status = std::process::Command::new("setx")
            .args([&p.env_var, &p.env_value])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("data moved to {} but setting {} failed", p.dest, p.env_var));
        }
        Ok(RelocationResult {
            dest: p.dest,
            env_var: p.env_var,
            env_value: p.env_value,
            note: "Restart the tool (and any terminal/IDE that launches it) to pick up the new location".into(),
        })
    }
}

fn user_env(name: &str) -> Option<String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // `reg query` reads the persisted user value, not this process's stale copy.
        let out = std::process::Command::new("reg")
            .args(["query", "HKCU\\Environment", "/v", name])
            .creation_flags(0x0800_0000)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text.lines().find(|l| l.trim_start().starts_with(name))?;
        let mut parts = line.splitn(3, "    ").map(str::trim).filter(|s| !s.is_empty());
        parts.nth(2).map(str::to_string)
    }
    #[cfg(not(windows))]
    {
        std::env::var(name).ok()
    }
}

fn running_processes(busy: &[&str]) -> Vec<String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let mut hits: Vec<String> = sys
        .processes()
        .values()
        .map(|p| p.name().to_lowercase().trim_end_matches(".exe").to_string())
        .filter(|n| busy.contains(&n.as_str()))
        .collect();
    hits.sort();
    hits.dedup();
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradle() -> &'static Tool {
        tool("gradle").unwrap()
    }

    #[test]
    fn plan_computes_dest_and_env_value() {
        let home = std::env::temp_dir().join("portal-reloc-home");
        std::fs::create_dir_all(home.join(".gradle")).unwrap();
        let p = build_plan(gradle(), &home, Path::new("I:\\build"), vec![], 5, None);
        assert!(p.dest.ends_with(".gradle"));
        assert!(p.dest.starts_with("I:"));
        assert_eq!(p.env_var, "GRADLE_USER_HOME");
        assert_eq!(p.env_value, p.dest);
        assert!(p.blockers.is_empty(), "{:?}", p.blockers);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn plan_blocks_when_running_missing_or_already_repointed() {
        let home = std::env::temp_dir().join("portal-reloc-home-missing");
        let p = build_plan(
            gradle(),
            &home,
            Path::new("I:\\build"),
            vec!["java".into()],
            0,
            Some("I:\\build\\.gradle".into()),
        );
        assert_eq!(p.blockers.len(), 3, "{:?}", p.blockers);
    }

    #[test]
    fn plan_blocks_dest_inside_source() {
        let home = std::env::temp_dir().join("portal-reloc-home-inside");
        std::fs::create_dir_all(home.join(".gradle")).unwrap();
        let p = build_plan(gradle(), &home, &home.join(".gradle"), vec![], 0, None);
        assert!(p.blockers.iter().any(|b| b.contains("inside")), "{:?}", p.blockers);
        let _ = std::fs::remove_dir_all(&home);
    }
}
