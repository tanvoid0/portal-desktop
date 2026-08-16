//! The webview transport, plus a re-export of the sink abstraction itself.
//!
//! The trait, the fanout and the `Events` state wrapper live in
//! [`portal_core::events`] — they have no business knowing about Tauri, and the
//! absence of a `tauri` dependency in that crate is what enforces it (phase 1 of
//! `docs/development/REMOTE_ARCHITECTURE.md`).
//!
//! What has to stay here is [`TauriSink`], which needs an `AppHandle`. The
//! re-export below means every `use crate::events::…` in the tree keeps working
//! unchanged; there is no reason to make 40 files name the crate split.

use serde_json::Value;

pub use portal_core::events::{EventSink, EventSinkExt, Events, FanoutSink};

/// Forwards to the Tauri webview.
#[derive(Debug, Clone)]
pub struct TauriSink(tauri::AppHandle);

impl TauriSink {
    pub fn new(handle: tauri::AppHandle) -> Self {
        Self(handle)
    }
}

impl EventSink for TauriSink {
    fn emit_json(&self, event: &str, payload: Value) -> bool {
        use tauri::Emitter;
        self.0.emit(event, payload).is_ok()
    }
}
