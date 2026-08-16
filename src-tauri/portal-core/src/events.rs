//! Transport-agnostic event emission.
//!
//! Background work — a coder loop step, a pty read, a k8s watch — needs to push
//! an event without knowing who is listening. Calling `AppHandle::emit` directly
//! makes the webview the only possible audience, which is what stops any of it
//! reaching a phone or a browser.
//!
//! The transports themselves live where their dependencies do: `TauriSink` in
//! `src-tauri` because it needs an `AppHandle`, `WsSink` in the daemon because it
//! needs a socket. Startup hands services a [`FanoutSink`] over both, so the two
//! audiences see one identical stream rather than a summary of it.
//! See `docs/development/REMOTE_ARCHITECTURE.md`.

use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// Somewhere to push an event. Object-safe on purpose: services hold
/// `Arc<dyn EventSink>` so the transport can be swapped at startup rather than
/// threaded through every signature.
pub trait EventSink: Send + Sync + fmt::Debug + 'static {
    /// Returns false when nobody received the event — the webview has gone, or
    /// no client is attached. Most callers ignore it; a producer that only
    /// exists to feed listeners, like the pty reader thread, uses it to stop
    /// rather than spin against an audience of none.
    fn emit_json(&self, event: &str, payload: Value) -> bool;
}

/// `emit` lives here rather than on [`EventSink`] so the trait stays object-safe
/// while call sites keep passing typed payloads instead of hand-built `Value`s.
pub trait EventSinkExt: EventSink {
    fn emit<S: Serialize>(&self, event: &str, payload: S) -> bool {
        match serde_json::to_value(payload) {
            Ok(value) => self.emit_json(event, value),
            // Deliberately not propagated: every call site is a fire-and-forget
            // notification, and a payload that will not serialize is a bug in
            // this process, not something the caller can recover from.
            Err(e) => {
                eprintln!("event '{event}': payload did not serialize: {e}");
                false
            }
        }
    }
}

impl<T: EventSink + ?Sized> EventSinkExt for T {}

/// Delivers to every attached transport.
///
/// Reports delivery if *any* sink took it, which is what lets a pty reader keep
/// running for a phone after the desktop window has closed.
#[derive(Debug)]
pub struct FanoutSink(Vec<Arc<dyn EventSink>>);

impl FanoutSink {
    pub fn new(sinks: Vec<Arc<dyn EventSink>>) -> Self {
        Self(sinks)
    }
}

impl EventSink for FanoutSink {
    fn emit_json(&self, event: &str, payload: Value) -> bool {
        // Folded rather than short-circuited: every transport gets the event,
        // and `any` would stop at the first that accepted it.
        self.0.iter().fold(false, |delivered, sink| {
            sink.emit_json(event, payload.clone()) || delivered
        })
    }
}

/// Wrapper so the sink can live in Tauri's managed state, which is keyed by
/// concrete type. Commands take `State<'_, Events>` and deref straight to the
/// trait.
#[derive(Debug, Clone)]
pub struct Events(Arc<dyn EventSink>);

impl Events {
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self(sink)
    }

    /// For handing to a service that stores the sink for its own lifetime.
    pub fn sink(&self) -> Arc<dyn EventSink> {
        Arc::clone(&self.0)
    }
}

impl Deref for Events {
    type Target = dyn EventSink;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct Recorder(Mutex<Vec<(String, Value)>>);

    impl EventSink for Recorder {
        fn emit_json(&self, event: &str, payload: Value) -> bool {
            self.0.lock().unwrap().push((event.to_string(), payload));
            true
        }
    }

    #[derive(Debug)]
    struct Deaf;

    impl EventSink for Deaf {
        fn emit_json(&self, _event: &str, _payload: Value) -> bool {
            false
        }
    }

    #[derive(Serialize)]
    struct Progress {
        done: u32,
    }

    #[test]
    fn typed_payloads_reach_the_sink_through_a_trait_object() {
        let recorder = Arc::new(Recorder::default());
        let events = Events::new(recorder.clone());

        // Through the newtype, as a command would.
        events.emit("scan://progress", Progress { done: 7 });
        // Through the bare trait object, as a service holding an Arc would.
        let sink: Arc<dyn EventSink> = recorder.clone();
        sink.emit("coder://change", serde_json::json!({ "id": "abc" }));

        let seen = recorder.0.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].0, "scan://progress");
        assert_eq!(seen[0].1, serde_json::json!({ "done": 7 }));
        assert_eq!(seen[1].0, "coder://change");
        assert_eq!(seen[1].1, serde_json::json!({ "id": "abc" }));
    }

    /// The pty reader thread stops on this, so a sink with no audience has to
    /// say so rather than silently swallowing.
    #[test]
    fn delivery_is_reported() {
        assert!(Recorder::default().emit("x", 1));
        assert!(!Deaf.emit("x", 1));
    }

    /// A phone keeps a terminal alive with the desktop window closed, so a
    /// fanout reports delivery when *any* transport took it — and every sink
    /// still gets the event, which a short-circuiting `any` would not do.
    #[test]
    fn fanout_reaches_every_sink_and_reports_any_delivery() {
        let recorder = Arc::new(Recorder::default());
        // Deaf first: a short-circuit on the *failing* sink would skip the rest.
        let fanout = FanoutSink::new(vec![Arc::new(Deaf), recorder.clone()]);
        assert!(fanout.emit("x", 1));
        assert_eq!(recorder.0.lock().unwrap().len(), 1);

        let all_deaf = FanoutSink::new(vec![Arc::new(Deaf), Arc::new(Deaf)]);
        assert!(!all_deaf.emit("x", 1));
    }

    #[test]
    fn unserializable_payload_reports_undelivered_instead_of_panicking() {
        // JSON object keys must be strings, so this fails in `to_value`.
        let bad: std::collections::HashMap<(i32, i32), i32> =
            std::collections::HashMap::from([((1, 2), 3)]);
        let recorder = Recorder::default();
        assert!(!recorder.emit("x", bad));
        assert!(recorder.0.lock().unwrap().is_empty());
    }
}
