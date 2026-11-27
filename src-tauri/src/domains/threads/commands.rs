//! Read side of the thread log. All three carry `#[rpc(cap = "coder")]` (the macro
//! takes only a literal) and then require `THREAD_READ_CAP`, the one knob for who
//! may read any provider's thread transcript.

use std::sync::Arc;

use tauri::State;

use super::{ThreadEventsPage, ThreadHub, ThreadSnapshot, ThreadSummary};
use crate::rpc::{Caller, THREAD_READ_CAP};

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_thread_list(
    caller: Caller,
    hub: State<'_, Arc<ThreadHub>>,
) -> Result<Vec<ThreadSummary>, String> {
    caller.require(THREAD_READ_CAP)?;
    hub.list().await
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_thread_snapshot(
    caller: Caller,
    hub: State<'_, Arc<ThreadHub>>,
    thread_id: String,
) -> Result<Option<ThreadSnapshot>, String> {
    caller.require(THREAD_READ_CAP)?;
    hub.inner().snapshot(&thread_id).await
}

#[portal_rpc_macro::rpc(cap = "coder")]
pub async fn coder_thread_events_since(
    caller: Caller,
    hub: State<'_, Arc<ThreadHub>>,
    thread_id: String,
    seq: i64,
) -> Result<ThreadEventsPage, String> {
    caller.require(THREAD_READ_CAP)?;
    hub.events_since(&thread_id, seq).await
}
