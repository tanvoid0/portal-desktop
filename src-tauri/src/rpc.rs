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
            denied: Arc::new(OnceLock::new()),
        }
    }

    /// A remote peer, scoped to `caps`.
    pub(crate) fn remote(caps: Vec<String>) -> Self {
        Self {
            caps: Some(caps),
            denied: Arc::new(OnceLock::new()),
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
pub const DEFAULT_CAPS: &[&str] = &[
    "coder",
    "projects",
    "tasks",
    "dashboard",
    "terminal",
    "agents",
];

/// Capabilities rendered unchecked in the approval dialog: granting one lets a
/// remote device do something that reaches outside the app's own data.
///
/// `agents.exec` starts an agent-CLI subprocess and answers the tool-call
/// permission prompts it raises (arbitrary shell/file-write included) — same
/// shape of risk as `terminal.exec`, so it gets the same opt-in treatment: a
/// paired device can still be granted it, just never by leaving the approval
/// dialog untouched. Also covers the coder domain's `claude-cli` engine
/// (`CLAUDE_CLI_PROVIDER`) for the same reason: a coder thread running it is
/// an agent-CLI subprocess too, so `coder/commands.rs::require_engine` gates
/// the seven commands that drive one behind this cap on top of the plain
/// `coder` one.
///
/// `remote.input` moves the mouse and types on the keyboard — a paired phone
/// that holds it can do anything a person sitting at this PC could, so it
/// gets the same opt-in treatment rather than riding along with pairing.
///
/// `agents.history` reads back a past run's full persisted transcript (tool
/// inputs/outputs, file contents, command output) — not RCE-equivalent like
/// `agents.exec`, but still a wholesale grant of everything ever typed or
/// produced in an agent run, so it opts in rather than riding the default
/// grant the way the live `agents` cap (which run/model info, not history)
/// does.
pub const SENSITIVE_CAPS: &[&str] = &[
    "terminal.exec",
    "agents.exec",
    "agents.history",
    REMOTE_INPUT_CAP,
];

/// Gates `/control`'s mouse/keyboard frames (`daemon::control::handle_client_frame`),
/// not an `/rpc` command — there is no command behind it to carry a
/// `#[rpc(cap = ...)]` attribute, so unlike every other entry in
/// [`capabilities`] this one is listed by hand rather than collected from the
/// registry. Kept as a named constant (vs. a bare string literal in three
/// places) so the approval UI, the sensitivity list and the gate itself can't
/// drift apart on a typo.
pub const REMOTE_INPUT_CAP: &str = "remote.input";

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

/// Every distinct capability a device can be granted: the registry's, plus
/// the handful (so far just [`REMOTE_INPUT_CAP`]) that gate a `/control`
/// frame type instead of an `/rpc` command and so have nothing in the
/// registry to collect. Sorted and deduped. Backs the exact cap-list guard
/// test and `remote_list_capabilities` — the approval checkbox UI's list.
pub fn capabilities() -> Vec<&'static str> {
    let mut caps: Vec<_> = inventory::iter::<Entry>
        .into_iter()
        .filter_map(|entry| entry.cap)
        .chain(std::iter::once(REMOTE_INPUT_CAP))
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

    /// A paired phone gets to move the mouse and type on this PC, so
    /// `remote.input` gets the same opt-in treatment as `terminal.exec` — and
    /// since it has no `/rpc` command behind it, this is also the only thing
    /// that would catch `capabilities()` forgetting to list it (the registry
    /// has nothing to collect it from).
    #[test]
    fn remote_input_is_sensitive_not_default() {
        assert!(
            !DEFAULT_CAPS.contains(&"remote.input"),
            "remote.input must not be granted by default"
        );
        assert!(
            SENSITIVE_CAPS.contains(&"remote.input"),
            "remote.input must be listed as sensitive so the dialog unchecks it"
        );
        assert!(
            capabilities().contains(&REMOTE_INPUT_CAP),
            "remote.input must still reach remote_list_capabilities despite \
             having no registry entry, or the approval UI can never offer it"
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

    /// The exact set of capabilities a device can be granted: the registry's,
    /// plus `remote.input` (see [`capabilities`]). Adding or removing a
    /// `cap = "..."` anywhere, or changing the hand-listed frame-only set,
    /// changes this, so this guard forces the remote surface to be a
    /// deliberate decision rather than a drift. The dangerous project
    /// commands (execute_command_in_directory, select_directory,
    /// open_project_in_explorer) and the whole credential vault are deliberately
    /// absent — loopback only.
    #[test]
    fn the_cap_list_is_exactly_the_annotated_set() {
        assert_eq!(
            capabilities(),
            vec![
                "agents",
                "agents.exec",
                "agents.history",
                "coder",
                "dashboard",
                "projects",
                "remote.input",
                "tasks",
                "terminal",
                "terminal.exec"
            ],
            "remote capability surface changed — update this guard only if intended"
        );

        // One command per cap, plus proof the fenced-off ones stay fenced.
        assert_eq!(capability("coder_list_threads"), Some("coder"));
        assert_eq!(capability("get_dashboard_overview"), Some("dashboard"));
        assert_eq!(capability("get_all_projects"), Some("projects"));
        assert_eq!(capability("create_task"), Some("tasks"));
        assert_eq!(capability("load_command_history"), Some("terminal"));
        assert_eq!(capability("create_terminal_process"), Some("terminal.exec"));
        assert_eq!(capability("agents_list_backends"), Some("agents"));
        assert_eq!(capability("agents_start_run"), Some("agents.exec"));
        assert_eq!(capability("agents_list_sessions"), Some("agents.history"));
        assert!(capability("execute_command_in_directory").is_none());
    }

    #[test]
    fn local_caller_is_unrestricted() {
        assert!(Caller::local().require("agents.exec").is_ok());
    }

    #[test]
    fn remote_caller_is_refused_a_cap_it_lacks_and_records_which_one() {
        let caller = Caller::remote(vec!["coder".to_string()]);
        let err = caller.require("agents.exec");
        assert!(err.is_err());
        assert_eq!(caller.denied(), Some("agents.exec"));
    }

    #[test]
    fn remote_caller_holding_the_cap_is_allowed() {
        let caller = Caller::remote(vec!["coder".to_string(), "agents.exec".to_string()]);
        assert!(caller.require("agents.exec").is_ok());
        assert_eq!(caller.denied(), None);
    }

    #[test]
    fn denied_is_visible_through_a_clone() {
        let caller = Caller::remote(vec!["coder".to_string()]);
        let clone = caller.clone();
        assert!(clone.require("agents.exec").is_err());
        assert_eq!(caller.denied(), Some("agents.exec"));
    }
}
