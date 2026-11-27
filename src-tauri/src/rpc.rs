//! The command registry behind `POST /rpc/{command}`.
//!
//! Entries are submitted by `#[rpc]` at the definition site and collected by
//! `inventory` at link time, so there is no central list to keep in step — a
//! command is reachable from every client the day it is written, or not at all.
//! See `docs/development/REMOTE_ARCHITECTURE.md`.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

use serde_json::Value;
use tauri::AppHandle;

type Call = for<'a> fn(
    &'a AppHandle,
    &'a Caller,
    Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>>;

/// Who is making this call, and what it's allowed to do.
///
/// `caps: None` is local/IPC — unrestricted, the same convention `events()`,
/// `pump()`, and `control.rs::input_allowed` already use for a loopback peer.
/// `Some(caps)` is a remote peer, scoped to the caps its pairing granted.
///
/// Deliberately not `Deserialize`: that would collide with the blanket
/// `impl<D: Deserialize> CommandArg for D` and let a remote client hand the
/// daemon its own `Caller` as a JSON argument. Instead [`Caller`] gets a
/// hand-written `CommandArg` impl below that always returns `local()` — this
/// is what makes every local Tauri IPC call unrestricted, by construction
/// rather than by a check that could be skipped.
#[derive(Clone)]
pub struct Caller {
    caps: Option<Vec<String>>,
    /// The paired device behind a remote call; `None` for local and `Anon`.
    device: Option<String>,
    /// Set by the first `require()` that fails, so the daemon can read back
    /// which capability a command's own `DispatchError::Failed` was actually
    /// about (vs. some unrelated error) — `Arc` so the clone a `#[rpc]`
    /// handler is handed still reports to the daemon's own copy.
    denied: Arc<OnceLock<&'static str>>,
}

impl Caller {
    /// The machine itself — loopback HTTP or Tauri IPC. Unrestricted. No
    /// `Default`: unrestricted must always be spelled out at the call site,
    /// never fallen into silently.
    pub(crate) fn local() -> Self {
        Self {
            caps: None,
            device: None,
            denied: Arc::new(OnceLock::new()),
        }
    }

    /// A remote peer, scoped to `caps`; `device` is its pairing id, if any.
    pub(crate) fn remote(device: Option<String>, caps: Vec<String>) -> Self {
        Self {
            caps: Some(caps),
            device,
            denied: Arc::new(OnceLock::new()),
        }
    }

    #[allow(dead_code)] // first callers: S2a command envelopes
    pub fn device(&self) -> Option<&str> {
        self.device.as_deref()
    }

    /// Who to credit in a log: `local`, `device:<id>`, or `remote` (no id).
    #[allow(dead_code)] // first callers: S2a command envelopes
    pub fn actor(&self) -> String {
        match (&self.caps, &self.device) {
            (None, _) => "local".into(),
            (Some(_), Some(id)) => format!("device:{id}"),
            (Some(_), None) => "remote".into(),
        }
    }

    /// Ok when local, or when `cap` is one of this caller's granted caps.
    /// Otherwise records `cap` as denied (see [`Caller::denied`]) and
    /// returns the same refusal message the static per-command cap check
    /// already uses.
    pub fn require(&self, cap: &'static str) -> Result<(), String> {
        match &self.caps {
            None => Ok(()),
            Some(caps) if caps.iter().any(|c| c == cap) => Ok(()),
            Some(_) => {
                let _ = self.denied.set(cap);
                Err("this device is not permitted to call this command".to_string())
            }
        }
    }

    /// The capability the most recent failed `require()` denied, if any.
    pub(crate) fn denied(&self) -> Option<&'static str> {
        self.denied.get().copied()
    }
}

/// Local IPC calls never carry a `Caller` argument from the frontend — this
/// is what makes them unrestricted rather than relying on every caller to
/// remember to construct one.
impl<'de, R: tauri::Runtime> tauri::ipc::CommandArg<'de, R> for Caller {
    fn from_command(
        _command: tauri::ipc::CommandItem<'de, R>,
    ) -> Result<Self, tauri::ipc::InvokeError> {
        Ok(Caller::local())
    }
}

/// One command, as the daemon sees it.
pub struct Entry {
    pub name: &'static str,
    pub call: Call,
    /// Capability a remote (LAN) peer must hold to reach this command. `None`
    /// means loopback/IPC-only: registered and reachable from this machine, but
    /// never served to a remote peer, since nothing it holds can match `None`.
    /// Set by `#[rpc(cap = "...")]`; absent when the attribute carries no `cap`.
    pub cap: Option<&'static str>,
}

inventory::collect!(Entry);

/// Capabilities granted to a device when the approval dialog is left untouched.
///
/// `terminal.exec` is deliberately absent — letting a phone run commands on this
/// PC is opt-in per device (the dialog renders it unchecked). Credentials carry
/// no capability at all, so they can never be granted to anyone.
///
/// `sysinfo` is the read-only host readout (`host_stats`, `disk_usage`, and
/// `system_details`, which names the busiest processes): it writes nothing and starts nothing, so it rides the default
/// grant. Move it to [`SENSITIVE_CAPS`] to make it opt-in.
pub const DEFAULT_CAPS: &[&str] = &[
    "coder",
    "projects",
    "tasks",
    "dashboard",
    "terminal",
    "agents",
    "sysinfo",
];

/// Capabilities rendered unchecked in the approval dialog: granting one lets a
/// remote device do something that reaches outside the app's own data.
///
/// `agents.exec` starts an agent-CLI subprocess and answers the tool-call
/// permission prompts it raises (arbitrary shell/file-write included) — same
/// shape of risk as `terminal.exec`, so it gets the same opt-in treatment: a
/// paired device can still be granted it, just never by leaving the approval
/// dialog untouched.
///
/// `agents.history` reads back a past run's full persisted transcript (tool
/// inputs/outputs, file contents, command output) — not RCE-equivalent like
/// `agents.exec`, but still a wholesale grant of everything ever typed or
/// produced in an agent run, so it opts in rather than riding the default
/// grant the way the live `agents` cap (which run/model info, not history)
/// does.
///
/// `github.write` acts on GitHub as the user — creating and editing issues,
/// and dispatching a workflow run, which can deploy depending on what that
/// workflow does. Read-only GitHub access (`github`) is not sensitive; this is.
///
/// `n8n.run` triggers an external n8n workflow. n8n workflows are arbitrary —
/// a node in one can run shell commands or hit any URL — so running one is as
/// sensitive as `terminal.exec`, even though it happens on another box.
///
/// `coder.exec` writes files and runs processes through the coder: a direct
/// file write under any caller-named workspace root (a `.bat` in the Startup
/// folder), shell-open of a path, git commit/push (hooks run), and driving
/// the agent loop — which in auto-accept mode or under an allow rule runs
/// tools without a prompt. Same risk as `terminal.exec`; the plain `coder` cap
/// keeps the read side. See `domains/coder/commands.rs`.
pub const SENSITIVE_CAPS: &[&str] = &[
    "terminal.exec",
    "coder.exec",
    "agents.exec",
    "agents.history",
    "github.write",
    "n8n.run",
];

/// The one cap that guards every provider's thread transcript reads
/// (`coder_thread_list` / `_snapshot` / `_events_since`) and the `coder://thread-*`
/// frames (`daemon::event_cap`). The commands carry `#[rpc(cap = "coder")]` (the macro
/// takes only a literal) and then `caller.require(THREAD_READ_CAP)`. Moving thread reads
/// to a new cap means this line, that cap's entry in `SENSITIVE_CAPS` if it is opt-in,
/// and the three `#[rpc(cap = "coder")]` literals in `src/domains/threads/commands.rs`
/// (a cap no command carries can never be granted, and the exact cap-set test in this
/// file says so). The daemon follows this constant on its own.
/// Owner decision 2026-11-27: the default-granted `coder`, so every paired
/// device reads every provider's transcripts, CLI ones included
/// (`docs/development/CODER_THREAD_LOG_DESIGN.md` §4).
pub const THREAD_READ_CAP: &str = "coder";

/// Why a call could not be made, as opposed to a call that ran and failed.
#[derive(Debug)]
pub enum DispatchError {
    /// No command by that name — either it does not exist or it is
    /// `#[rpc(local_only)]`.
    Unknown,
    /// The command ran and returned an error.
    Failed(String),
}

pub fn is_registered(name: &str) -> bool {
    lookup(name).is_some()
}

/// Every registered name, sorted. For diagnostics and the drift test.
pub fn names() -> Vec<&'static str> {
    let mut names: Vec<_> = inventory::iter::<Entry>
        .into_iter()
        .map(|e| e.name)
        .collect();
    names.sort_unstable();
    names
}

/// The capability a remote peer needs to call `name`, or `None` when the command
/// is loopback/IPC-only (or not registered at all). The daemon reads this for a
/// remote peer: `None` here means "refuse", since a peer's granted set can never
/// contain `None`.
pub fn capability(name: &str) -> Option<&'static str> {
    lookup(name).and_then(|entry| entry.cap)
}

/// Every distinct capability a device can be granted, collected from the
/// registry. Sorted and deduped. Backs the exact cap-list guard test and
/// `remote_list_capabilities` — the approval checkbox UI's list.
pub fn capabilities() -> Vec<&'static str> {
    let mut caps: Vec<_> = inventory::iter::<Entry>
        .into_iter()
        .filter_map(|entry| entry.cap)
        .collect();
    caps.sort_unstable();
    caps.dedup();
    caps
}

fn lookup(name: &str) -> Option<&'static Entry> {
    inventory::iter::<Entry>
        .into_iter()
        .find(|entry| entry.name == name)
}

/// Run a command by name with a JSON argument object, on behalf of `caller`.
pub async fn dispatch(
    app: &AppHandle,
    caller: &Caller,
    name: &str,
    args: Value,
) -> Result<Value, DispatchError> {
    let entry = lookup(name).ok_or(DispatchError::Unknown)?;
    (entry.call)(app, caller, args)
        .await
        .map_err(DispatchError::Failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `inventory` collects through link-section magic, and the failure mode is
    /// an empty registry rather than a build error — which would look exactly
    /// like "the daemon serves nothing" at runtime. Fail here instead.
    #[test]
    fn the_registry_is_not_empty() {
        let names = names();
        assert!(
            names.contains(&"coder_list_threads"),
            "registry did not collect #[rpc] commands; got {names:?}"
        );

        // A floor rather than an exact count, so adding a command does not fail
        // this — but reverting the conversion of whole domains would. Every
        // command below the floor is one no remote client can reach.
        assert!(
            names.len() > 300,
            "only {} commands registered — a domain has probably lost its \
             `#[portal_rpc_macro::rpc]` attributes",
            names.len()
        );
    }

    #[test]
    fn names_are_unique() {
        let names = names();
        let mut unique = names.clone();
        unique.dedup();
        assert_eq!(
            names.len(),
            unique.len(),
            "two commands registered under one name; /rpc would reach whichever \
             the linker happened to order first"
        );
    }

    #[test]
    fn unregistered_names_are_distinguishable_from_failures() {
        assert!(!is_registered("no_such_command"));
        assert!(is_registered("coder_list_threads"));
    }

    /// The credential vault must never be reachable over the LAN. It is still a
    /// registered command (loopback and IPC reach it), but it carries no
    /// capability, so `capability` answers `None` and the daemon refuses it to
    /// any remote peer. No annotation anywhere can grant it.
    #[test]
    fn credentials_are_never_remotable() {
        assert!(
            is_registered("decrypt_credential"),
            "decrypt_credential should still be a registered (loopback) command"
        );
        assert!(
            capability("decrypt_credential").is_none(),
            "decrypt_credential must carry no capability — it is ungrantable"
        );
    }

    /// "Sync all" pushes to remotes and uploads a snapshot, and `push_ok` is
    /// the push allowlist: none of it may be reachable by a LAN device.
    #[test]
    fn sync_all_is_local_only() {
        for name in [
            "sync_all_projects",
            "cancel_sync_all",
            "set_project_push_ok",
            "sync_repo_facts",
            "sync_apply_resolution",
            "drive_sync_drives",
            "drive_sync_status",
            "drive_sync_apply",
        ] {
            // local_only commands are not in the registry at all.
            assert!(!is_registered(name), "{name}");
            assert_eq!(capability(name), None, "{name}");
        }
    }

    /// Running commands on the host is opt-in per device, so `terminal.exec` is
    /// sensitive and never in the untouched-dialog default grant.
    #[test]
    fn terminal_exec_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"terminal.exec"),
            "terminal.exec must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"terminal.exec"),
            "terminal.exec must be listed as sensitive so the dialog unchecks it"
        );
    }

    /// Starting an agent run spawns a process and answers its tool-call
    /// permission prompts — RCE-equivalent, so it gets the same opt-in
    /// treatment as `terminal.exec` rather than the default grant.
    #[test]
    fn agents_exec_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"agents.exec"),
            "agents.exec must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"agents.exec"),
            "agents.exec must be listed as sensitive so the dialog unchecks it"
        );
    }

    /// The coder's write/run/approve side reaches outside the app's own data
    /// (any file on disk, git hooks, an auto-accepting agent loop), so it is
    /// opt-in. Every command that can do that carries `coder.exec`, not the
    /// default `coder` — listed here so moving one back is a test failure.
    #[test]
    fn coder_exec_is_sensitive_not_default() {
        assert!(!DEFAULT_CAPS.contains(&"coder.exec"));
        assert!(SENSITIVE_CAPS.contains(&"coder.exec"));
        for name in [
            "coder_send",
            "coder_remove_queued",
            "coder_retry",
            "coder_edit_message",
            "coder_approve",
            "coder_set_mode",
            "coder_set_permission_mode",
            "coder_add_rule",
            "coder_remove_rule",
            "coder_accept_change",
            "coder_reject_change",
            "coder_set_hunk",
            "coder_modify_change",
            "coder_write_file",
            "coder_open_in_explorer",
            "coder_git_commit",
            "coder_git_push",
            "coder_submit_command_result",
            "coder_submit_terminal_list",
            "coder_multitask_spawn",
            "coder_multitask_cleanup",
        ] {
            assert_eq!(capability(name), Some("coder.exec"), "{name}");
        }
    }

    /// A project's start/build/test commands run as typed when the user clicks
    /// Run/Build/Test, in its `path`, so writing one is running a command:
    /// `terminal.exec`, not the default `projects`. Creating always sets
    /// `path`; updating is gated per field, so a default device can still
    /// rename (to a shell-inert name), describe or re-status a project.
    #[test]
    fn project_run_fields_need_terminal_exec() {
        use crate::domains::projects::commands::require_exec_for_run_fields as guard;
        assert_eq!(capability("create_project"), Some("terminal.exec"));
        assert_eq!(capability("update_project"), Some("projects"));

        let default_remote =
            || Caller::remote(None, DEFAULT_CAPS.iter().map(|c| c.to_string()).collect());
        let exec_remote = Caller::remote(None, vec!["projects".into(), "terminal.exec".into()]);
        // path, build_command, start_command, test_command, output_directory
        for i in 0..5 {
            let mut f: [Option<String>; 5] = Default::default();
            f[i] = Some("calc.exe".into());
            let caller = default_remote();
            assert!(
                guard(&caller, &f[0], &f[1], &f[2], &f[3], &f[4], &None).is_err(),
                "field {i}"
            );
            assert_eq!(caller.denied(), Some("terminal.exec"), "field {i}");
            assert!(guard(&Caller::local(), &f[0], &f[1], &f[2], &f[3], &f[4], &None).is_ok());
            assert!(guard(&exec_remote, &f[0], &f[1], &f[2], &f[3], &f[4], &None).is_ok());
        }
        let none = None;
        assert!(guard(&default_remote(), &none, &none, &none, &none, &none, &none).is_ok());

        // `${PROJECT_NAME}` is spliced raw into `cmd /C` / `sh -c`.
        let rename = |n: &str| Some(n.to_string());
        for ok in ["My App", "portal_desktop-2.0", "v1.2 beta"] {
            assert!(
                guard(
                    &default_remote(),
                    &none,
                    &none,
                    &none,
                    &none,
                    &none,
                    &rename(ok)
                )
                .is_ok(),
                "{ok:?}"
            );
        }
        for bad in [
            "x & calc.exe & rem",
            "x | y",
            "a;b",
            "$(id)",
            "`id`",
            "%PATH%",
            "a
b",
            "a	b",
            "\"q",
            "x > out",
            "-rf",
            "x --upload-pack calc",
            "Café",
            "\u{2BA}-x\u{2BA}",
            "",
            ".",
            "out ..",
            "a  b",
        ] {
            let caller = default_remote();
            assert!(
                guard(&caller, &none, &none, &none, &none, &none, &rename(bad)).is_err(),
                "{bad:?}"
            );
            assert_eq!(caller.denied(), Some("terminal.exec"), "{bad:?}");
            assert!(guard(
                &Caller::local(),
                &none,
                &none,
                &none,
                &none,
                &none,
                &rename(bad)
            )
            .is_ok());
            assert!(guard(
                &exec_remote,
                &none,
                &none,
                &none,
                &none,
                &none,
                &rename(bad)
            )
            .is_ok());
        }
    }

    /// Reading back a past run's full transcript is a wholesale data grant
    /// (every tool input/output, file content, command output that run ever
    /// touched), so it opts in rather than riding the default `agents` grant.
    #[test]
    fn agents_history_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"agents.history"),
            "agents.history must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"agents.history"),
            "agents.history must be listed as sensitive so the dialog unchecks it"
        );
    }

    /// Dispatching a workflow (or editing an issue) acts on GitHub as the
    /// user and can deploy, so it opts in rather than riding the default
    /// `github` (read-only) grant.
    #[test]
    fn github_write_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"github.write"),
            "github.write must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"github.write"),
            "github.write must be listed as sensitive so the dialog unchecks it"
        );
    }

    /// Running an n8n workflow can execute arbitrary commands via its nodes,
    /// so it gets the same opt-in treatment as `terminal.exec`.
    #[test]
    fn n8n_run_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"n8n.run"),
            "n8n.run must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"n8n.run"),
            "n8n.run must be listed as sensitive so the dialog unchecks it"
        );
    }

    /// `capabilities()` is what the exact cap-list guard (added once domains are
    /// annotated) and `remote_list_capabilities` build on, so it must come back
    /// sorted and free of duplicates whatever the registry holds.
    #[test]
    fn capabilities_are_sorted_and_deduped() {
        let caps = capabilities();
        let mut sorted = caps.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            caps, sorted,
            "capabilities() must be sorted and deduplicated"
        );
    }

    /// The exact set of capabilities a device can be granted, collected from
    /// the registry. Adding or removing a `cap = "..."` anywhere changes this,
    /// so this guard forces the remote surface to be a deliberate decision
    /// rather than a drift. The dangerous project commands
    /// (execute_command_in_directory, select_directory,
    /// open_project_in_explorer) and the whole credential vault are deliberately
    /// absent — loopback only.
    #[test]
    fn the_cap_list_is_exactly_the_annotated_set() {
        assert_eq!(
            capabilities(),
            vec![
                "actions",
                "agents",
                "agents.exec",
                "agents.history",
                "coder",
                "coder.exec",
                "dashboard",
                "github",
                "github.write",
                "n8n",
                "n8n.run",
                "pipelines",
                "projects",
                "sysinfo",
                "tasks",
                "terminal",
                "terminal.exec"
            ],
            "remote capability surface changed — update this guard only if intended"
        );

        // One command per cap, plus proof the fenced-off ones stay fenced.
        assert_eq!(capability("coder_list_threads"), Some("coder"));
        assert_eq!(capability("coder_write_file"), Some("coder.exec"));
        assert_eq!(capability("get_dashboard_overview"), Some("dashboard"));
        assert_eq!(capability("get_all_projects"), Some("projects"));
        assert_eq!(capability("create_project"), Some("terminal.exec"));
        assert_eq!(capability("create_task"), Some("tasks"));
        assert_eq!(capability("load_command_history"), Some("terminal"));
        assert_eq!(capability("create_terminal_process"), Some("terminal.exec"));
        assert_eq!(capability("agents_list_backends"), Some("agents"));
        assert_eq!(capability("agents_start_run"), Some("agents.exec"));
        assert_eq!(capability("agents_list_sessions"), Some("agents.history"));
        assert_eq!(capability("get_pipelines"), Some("pipelines"));
        assert_eq!(capability("execute_pipeline"), Some("terminal.exec"));
        assert_eq!(capability("github_list_repositories"), Some("github"));
        assert_eq!(capability("github_create_issue"), Some("github.write"));
        assert_eq!(capability("check_n8n_health"), Some("n8n"));
        assert_eq!(capability("trigger_n8n_workflow"), Some("n8n.run"));
        assert_eq!(capability("get_script_execution"), Some("actions"));
        assert_eq!(capability("get_recent_runs"), Some("actions"));
        assert_eq!(capability("execute_script"), Some("terminal.exec"));
        assert_eq!(capability("host_stats"), Some("sysinfo"));
        assert_eq!(capability("disk_usage"), Some("sysinfo"));
        assert_eq!(capability("system_details"), Some("sysinfo"));
        assert!(capability("execute_command_in_directory").is_none());
        assert!(capability("create_pipeline").is_none());
        assert!(capability("github_start_device_flow").is_none());
        assert!(capability("github_clone_repository").is_none());
        assert!(capability("register_embedded_workflow").is_none());
        assert!(capability("release_ship").is_none());
    }

    #[test]
    fn local_caller_is_unrestricted() {
        assert!(Caller::local().require("agents.exec").is_ok());
    }

    #[test]
    fn remote_caller_is_refused_a_cap_it_lacks_and_records_which_one() {
        let caller = Caller::remote(None, vec!["coder".to_string()]);
        let err = caller.require("agents.exec");
        assert!(err.is_err());
        assert_eq!(caller.denied(), Some("agents.exec"));
    }

    #[test]
    fn remote_caller_holding_the_cap_is_allowed() {
        let caller = Caller::remote(None, vec!["coder".to_string(), "agents.exec".to_string()]);
        assert!(caller.require("agents.exec").is_ok());
        assert_eq!(caller.denied(), None);
    }

    #[test]
    fn denied_is_visible_through_a_clone() {
        let caller = Caller::remote(None, vec!["coder".to_string()]);
        let clone = caller.clone();
        assert!(clone.require("agents.exec").is_err());
        assert_eq!(caller.denied(), Some("agents.exec"));
    }
}
