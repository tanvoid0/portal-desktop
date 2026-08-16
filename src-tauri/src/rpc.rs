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
}

inventory::collect!(Entry);

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
}
