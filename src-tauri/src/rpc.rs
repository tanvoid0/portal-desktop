//! The command registry behind `POST /rpc/{command}`.
//!
//! Entries are submitted by `#[rpc]` at the definition site and collected by
//! `inventory` at link time, so there is no central list to keep in step — a
//! command is reachable from every client the day it is written, or not at all.
//! See `docs/development/REMOTE_ARCHITECTURE.md`.

use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use tauri::AppHandle;

type Call = for<'a> fn(
    &'a AppHandle,
    Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>>;

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
pub const DEFAULT_CAPS: &[&str] = &["coder", "projects", "tasks", "dashboard", "terminal"];

/// Capabilities rendered unchecked in the approval dialog: granting one lets a
/// remote device do something that reaches outside the app's own data.
pub const SENSITIVE_CAPS: &[&str] = &["terminal.exec"];

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

/// Every distinct capability the registry declares, sorted. Backs the exact
/// cap-list guard test and `remote_list_capabilities`.
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

/// Run a command by name with a JSON argument object.
pub async fn dispatch(app: &AppHandle, name: &str, args: Value) -> Result<Value, DispatchError> {
    let entry = lookup(name).ok_or(DispatchError::Unknown)?;
    (entry.call)(app, args).await.map_err(DispatchError::Failed)
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

    /// The exact set of capabilities the registry declares. Adding or removing a
    /// `cap = "..."` anywhere changes this set, so this guard forces the remote
    /// surface to be a deliberate decision rather than a drift. The dangerous
    /// project commands (execute_command_in_directory, select_directory,
    /// open_project_in_explorer) and the whole credential vault are deliberately
    /// absent — loopback only.
    #[test]
    fn the_cap_list_is_exactly_the_annotated_set() {
        assert_eq!(
            capabilities(),
            vec![
                "coder",
                "dashboard",
                "projects",
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
        assert_eq!(capability("execute_command"), Some("terminal.exec"));
        assert!(capability("execute_command_in_directory").is_none());
    }
}
