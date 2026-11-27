# Coder thread log — design (Phase 1, #1916)

Phase 1 of [CODER_REVAMP_PLAN.md](CODER_REVAMP_PLAN.md): server-owned, replayable Coder threads
for both stacks. Transport is unchanged (`/rpc` + `/events`). PCORE =
`src-tauri/portal-core/src/domains`.

Written 2026-11-27 by crew:architect from reading the code; the load-bearing claims in section 0
and section 8 were re-checked against the tree the same day. Board: #1916, slices #1925-#1935.

Where this differs from the plan's Phase 1 bullets, this document wins:

- **Vocabulary:** provider output is normalised at ingest into `ThreadEvent`, not stored as
  "`AgentEvent` plus Portal variants" (section 2).
- **Backfill:** an idempotent startup importer, not a one-off migration (section 6).
- **Receipts:** `command_id` lives on the event row only; no receipts table (section 5).

## 0. What the code does today

- **Portal transcript is not incremental.** At `done`, `sync_thread_from_platform` replaces the
  whole message list with the platform's (`coder/service.rs:2479-2511`). The live
  `coder://message` frames are synthetic previews (`service.rs:2244-2269`). Every persist
  rewrites the full `messages_json` blob (`service.rs:890-930`).
- **A Portal approval does not park a task.** The run returns with `pending`
  (`service.rs:2373-2388`) and `coder_approve` spawns a new run
  (`src-tauri/src/domains/coder/commands.rs:130-160`). The only waits on a human are host
  round-trips on oneshots: `ask_user` 290 s, terminal `run_command` 120 s, `list_terminals` 10 s
  (`service.rs:543-618`).
- **Agent mode, permission mode and rules are global, not per thread** (`service.rs:167-169`).
- **CLI stack:** one pump per run assigns `seq`, fills a 500-entry ring, emits `agents://event`,
  and persists non-delta events (`agents/service.rs:448-536`). Run id equals `agent_sessions.id`,
  and resume reuses it (`service.rs:342-346`). Feed, analytics and Timeline read `agent_events`
  directly (`service.rs:638-700`).
- **Both stacks key threads by UUID v4** (`coder/service.rs:986`, `agents/service.rs:316`).
- **`Caller` has no device identity** (`src-tauri/src/rpc.rs:34-41`); `caller_for` drops
  `Trust::Remote.device_id` (`src-tauri/src/daemon/mod.rs:647-652`).
- **Event caps are by event-name domain** (`daemon/mod.rs:461-474`). Live CLI events sit under
  `agents`; persisted CLI history is under the sensitive `agents.history` (`rpc.rs:135-160`).
- **Lag:** one 256-frame broadcast channel is shared by all events (`daemon/mod.rs:96`). A lagging
  socket gets `events://lagged` (`mod.rs:701-707`). The browser shim
  (`src/lib/domains/mobile/browserShim.ts:276-281`) and the mobile shell
  (`mobile/src-tauri/src/lib.rs:296-304`) re-emit every frame, so a client can listen for it.

## 1. Storage

**Decision: new tables, not an extension of `agent_events`.** `agent_events` has a foreign key to
`agent_sessions` and three readers that pattern-match its JSON; putting Portal threads in it
means a SQLite table rebuild. It stays as the raw provider log for Timeline and analytics.

**Thread id:** the existing id, verbatim. `coder_threads.id` and `agent_sessions.id` become
`threads.id`; no prefix, no mapping table, deep links keep resolving. `threads.provider` is
`portal` or the backend id (`claude-code`, `codex`, `cursor`).

Migration `m20261127_000056_create_thread_log.rs` — DDL only, additive, `if_not_exists`, no
`down`. Renumber if another branch takes 56 first.

```
threads
  id TEXT PK, provider TEXT NOT NULL, kind TEXT NOT NULL DEFAULT 'session',
  title TEXT NOT NULL, workspace_root TEXT NOT NULL, project_id INTEGER NULL,
  model TEXT NULL, llm_provider TEXT NULL,
  status TEXT NOT NULL,              -- idle | running | waiting | error
  state_json TEXT NOT NULL,          -- approvals, host requests, queue, usage, todos,
                                     -- last error, platform_thread_id / backend_session_id
  last_seq INTEGER NOT NULL DEFAULT 0, message_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
  idx_threads_updated_at(updated_at), idx_threads_project_id(project_id)

thread_events            -- append-only
  thread_id TEXT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
  seq INTEGER NOT NULL, kind TEXT NOT NULL, payload TEXT NOT NULL,
  command_id TEXT NULL, actor TEXT NOT NULL, at TEXT NOT NULL,
  PRIMARY KEY (thread_id, seq)
  idx_thread_events_command(thread_id, command_id) WHERE command_id IS NOT NULL

thread_messages          -- projection, one row per ChatMessage
  thread_id TEXT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
  idx INTEGER NOT NULL, role TEXT NOT NULL, message_json TEXT NOT NULL, seq INTEGER NOT NULL,
  PRIMARY KEY (thread_id, idx)
```

Messages are rows, not a blob, so an append is one insert and a phone can page later.
`FileChange` bodies stay in `coder_file_changes`; the log carries only `{change_id, path, status}`.
Old tables are untouched.

## 2. `ThreadEvent` and `ThreadCommand`

**Decision: normalise at ingest; do not wrap `AgentEvent`.** Two reasons:

- Wrapping means the `AgentEvent` → `ChatMessage` fold exists twice, in Rust `apply` and in the
  TS reducer. That fold is today's `agents/utils/eventsToMessages.ts`.
- The Portal loop already speaks `ChatMessage` (`coder://message`).

With normalisation the fold lives once, in pure Rust `decide`, and the TS reducer never sees a
provider enum. `AgentEvent` stays the adapter-facing type and stays in `agent_events`.

```rust
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThreadEvent {                       // persisted, one seq each
  Created { provider, model, llm_provider, workspace_root, project_id, kind, title },
  TitleSet { title },
  ModelSet { model, llm_provider },
  MessageAppended { message: ChatMessage },           // user, assistant, tool call, tool result
  MessagesRewritten { keep: u32, tail: Vec<ChatMessage> }, // edit, retry, platform reconcile
  TurnStarted { trigger },                            // send | retry | edit | approve | queue
  StopRequested,
  TurnEnded { outcome, error: Option<String>, final_text: Option<String> },
                                // completed | stopped | error | exhausted | interrupted
  ApprovalRequested { approval: PendingApproval },
  ApprovalResolved { call_id, allow, auto: bool },
  HostRequested { call_id, kind, payload },           // run_command | list_terminals | ask_user
  HostResolved { call_id },
  Enqueued { queue_id, text },
  Dequeued { queue_id, reason },                      // sent | removed
  ChangeRecorded { change_id, path, status },
  SubAgentUpdated { sub_agent: CoderSubAgent },
  UsageUpdated { .. }, TodosUpdated { items }, SubagentsRunning { count },
  SessionLinked { backend_session_id }, ProcessExited { code },
}
```

Who acted (`actor`: `local`, `device:<id>` or `system`) is on the frame and the row, not in each
variant.

- **`MessagesRewritten { keep, tail }`** covers three current behaviours: `prepare_edit_message`
  (`service.rs:1814`), `prepare_retry` (`service.rs:1867`), and the wholesale platform sync. The
  hub diffs the platform list against the projection by common prefix, so the usual tail is just
  the final assistant message.
- **Live-only, never persisted:** `Delta { text, thinking }` and `Heartbeat { waited_seconds }`.
- **Late joiner:** the actor keeps `partial_text` and `partial_thinking` in memory and the
  snapshot is answered by the actor, so it includes them. A delta frame carries the last
  committed `seq` and a character `offset`. A client whose partial length differs from `offset`
  re-snapshots. Nothing is written per token. A hub crash loses the partial, and the turn is dead
  anyway.

```rust
pub enum ThreadCommand {
  // client intents — the envelope carries command_id + actor
  Create {..}, Send { text }, RemoveQueued { queue_id }, EditMessage { index, text }, Retry, Stop,
  Approve { call_id, allow, remember, edited_pattern, edited_command },
  ResolveHost { call_id, result }, SetModel {..},
  // producer inputs — actor = system, no command_id
  Provider(AgentEvent),      // CLI pump; also the Portal loop where a variant fits
  Portal(PortalInput),       // Reconcile{messages,title}, Title, Heartbeat, HostRequest, Pending, Failed, Finished
  ChangeRecorded {..}, SubAgentUpdated {..}, DrainQueue,
}
```

- `Send` while `running` or `waiting` decides to `Enqueued`; otherwise `MessageAppended(user)` +
  `TurnStarted`.
- CLI events tagged `parent_tool_use_id` are dropped from the thread log, as the feed does today
  (`eventsToMessages.ts:71-76`). They remain in `agent_events`.
- Agent mode and permission mode stay global in Phase 1.

## 3. Single writer, decide/apply, effects

New domain `PCORE/threads/` with `types.rs`, `decide.rs`, `apply.rs`, `hub.rs`, `store.rs`,
`entities/`.

```rust
pub fn decide(state: &ThreadState, cmd: &ThreadCommand, ctx: &Ctx) -> Result<Vec<ThreadEvent>, Reject>;
pub fn apply(state: &mut ThreadState, event: &ThreadEvent);
pub fn react(state: &ThreadState, event: &ThreadEvent) -> Vec<Effect>;   // pure
```

`Ctx` carries `now`, the actor and pre-generated ids, so `decide` is deterministic.

**One tokio task per thread**, with an unbounded `mpsc` (the same choice the agents pump made,
`agents/service.rs:314`). It is spawned lazily, hydrated from the `threads` row plus
`thread_messages` (never by replaying events), and evicted after 10 idle minutes. Loop:

1. Receive the envelope. If `command_id` is already in `thread_events`, reply `duplicate` with the
   original seq.
2. `decide`. On `Err(Reject)`, reply `rejected`; nothing is written.
3. One transaction: insert events at `last_seq+1..`, apply them, write the `threads` row and the
   `thread_messages` delta. Commit.
4. Swap state, reply `accepted { seq }`, emit one frame per event.
5. `react` → effects, each in its own `tokio::spawn`. The actor never awaits one.

Deltas go through the same mailbox, so they order against commits, but they skip steps 1-3. The
actor coalesces them to one frame per ~40 ms (a tunable constant) to protect the shared 256-frame
channel.

**The lock rule holds by construction.** The transaction contains only SQL. Approvals, vault
prompts, provider calls and host tools all happen in effect tasks or in the existing run task.

**Effects** sit behind a `ThreadEffects` trait with two implementers:

- `CoderService`: `RunTurn(AgentTurn)`, `Cancel`, `DeliverHostResult`, `AddRule`.
- `AgentRunService`: `StartBackend`, `SendUser`, `DecidePermission`, `Interrupt`, `Kill`.

Results re-enter as commands.

**Producers are not rewritten.**

- **CLI:** `spawn_pump` gains one line forwarding each `AgentEvent` to
  `hub.submit(run_id, Provider(event))`. Ring, emit and persist are unchanged. Adapters are
  untouched.
- **Portal:** `advance_platform` keeps its control flow. Each `self.emit("coder://…")` site also
  submits a hub input:
  - `assistant` → `Delta`;
  - `tool_call` and tool result → `MessageAppended`;
  - `done` → `Reconcile`;
  - `pending` after `permissions::decide` says Prompt → `ApprovalRequested`;
  - `finish` → `TurnEnded`.

  Legacy emits and `coder_threads` writes stay until slice S5, so the old frontend works
  throughout.

- **Stop is uniform.** The command commits `StopRequested` and fires `Cancel`/`Interrupt`. The
  provider's own end produces `TurnEnded` (the cancel paths at `service.rs:1997` and `2317`;
  `TurnDone`/`Exited` for CLI). No turn tagging is needed, because `decide` never starts a second
  run while one is open.
- **Host tools.** `HostRequested` is answered only by the hub's own desktop webview, the only
  client with the terminal stack. Other clients render it. The 120 s fallback to a background
  shell (`service.rs:571-580`) covers a hub with no webview. `ask_user` can be answered from any
  device, first wins.

## 4. Snapshot, catch-up, frames

New RPC:

- `coder_thread_list() -> Vec<ThreadSummary>`
- `coder_thread_snapshot(thread_id) -> ThreadSnapshot { thread, seq, messages, state }`, where
  `state` holds status, approvals, host requests, queue, partial text and thinking, usage, todos,
  error and `can_retry`.
- `coder_thread_events_since(thread_id, seq) -> { events: Vec<ThreadEventFrame>, head_seq, reset }`.
  `reset` is true when `seq > head_seq` or the gap exceeds 2000 events; the client then
  re-snapshots.

**Writes keep their existing names.** `coder_send`, `coder_retry`, `coder_edit_message`,
`coder_approve`, `coder_stop`, `coder_submit_*` and
`agents_start_run` / `resume_session` / `send_user` / `decide_permission` / `interrupt` / `cancel`
each gain an optional `command_id` and return
`CommandAck { status: accepted | duplicate | rejected, seq, reason }`. Caps do not move, so the
test at `rpc.rs:341-367` stands. The only new write is `coder_queue_remove`.

**One cap for every provider's threads** (owner decision, 2026-11-27).

- Every thread frame goes out as `coder://thread-event` / `coder://thread-delta`, whatever the
  provider, so `event_cap` filters them all under one cap with no daemon change.
- The thread read RPCs (`coder_thread_list`, `coder_thread_snapshot`, `coder_thread_events_since`)
  require that one cap for every provider; no extra `agents.history` check.
- Writes keep their current caps: `coder.exec` for Portal-engine commands, `agents.exec` for CLI
  ones. `agents.history` keeps guarding the raw `agent_events` reads (Timeline, analytics,
  Overview).
- **The cap is `coder`** (owner decision, 2026-11-27): `THREAD_READ_CAP = "coder"`. Known
  consequence, accepted: `coder` is in `DEFAULT_CAPS` (`rpc.rs:116-123`), so every paired device
  can read CLI transcripts by default, which `agents.history` (opt-in, "everything ever typed",
  `rpc.rs:135-140`) kept from them until now. Portal-engine transcripts were already readable that
  way. To tighten later, point `THREAD_READ_CAP` at a new cap in `SENSITIVE_CAPS`; it is one line.

Frame: `{ thread_id, seq, at, actor, command_id?, event }`. Delta frame:
`{ thread_id, seq, offset, text, thinking }`.

**Client rules** (one implementation, `threadStore.svelte.ts`):

- **Subscribe:** listen and buffer, snapshot, drop buffered frames at or below `snapshot.seq`,
  apply the rest.
- **Per frame:** `seq == cursor+1` applies; `seq <= cursor` is dropped; `seq > cursor+1` buffers
  and triggers a single-flight `events_since(cursor)`.
- **Resync triggers:** `events://lagged` (socket-wide, so every open thread), bridge reconnect
  (`mobile://connection` going live) and `visibilitychange`. Each runs `events_since` per open
  thread and invalidates the list query.
- The local webview goes through `TauriSink` and never lags. Its only case is reload, which is a
  snapshot.

## 5. Dedup and approvals

- **Dedup:** `thread_events.command_id` with the partial index. An indexed lookup per command is
  cheap at human rates. The window is the life of the thread. No receipts table: a rejected
  command writes nothing, and re-deciding a retried rejection gives the same answer.
- **First wins:** `decide(Approve)` on a `call_id` not in `state.approvals` returns
  `Reject::AlreadyResolved { by, allow }` from a 32-entry `recently_resolved` map. The loser's ack
  is `rejected / already_resolved` with who and what, and it also receives the winner's
  `ApprovalResolved` frame.
- **CLI:** the hub records `ApprovalResolved` on the user command, then fires `DecidePermission`.
  The backend's later `PermissionResolved` is dropped as already resolved.
- **Which device:** `Caller` gains `device: Option<String>`, set in `caller_for`. It stays
  non-`Deserialize`, so a remote client cannot supply one. The display name is resolved once in
  the command layer from the paired-devices table (assumed).

## 6. Data migration

**Decision: not a migration.** `PCORE/threads/legacy_import.rs` is an idempotent importer run at
every startup, next to `reconcile_stale_sessions` (`src-tauri/src/lib.rs:320`).

It imports a `coder_threads` or `agent_sessions` row when either holds:

- there is no `threads` row for it;
- `threads.last_seq = 0` and the legacy `updated_at` is newer.

It writes projections only: `threads` + `thread_messages`, `last_seq = 0`, no synthetic events.
Agent sessions are folded from `agent_events` + `agent_messages` with the same function `decide`
uses; this mirrors `detailToLog` (`agentRuns.svelte.ts:46-70`), including imported transcripts
that have only message rows.

Why not a migration:

- The legacy path keeps creating rows during the dual-write window, so a one-off copy would need a
  catch-up migration, as `m20261119_000052` was for `000051`.
- The fold is domain logic that should not be frozen in a migration file.

Dev/release safety: migration 56 is additive, so a release only gains three empty tables. A dev
copy re-runs the importer on its own data.

**Startup reconcile:**

- `status = running` → `TurnEnded { interrupted }`.
- CLI `waiting` → approvals cleared, because the process is gone.
- Portal `waiting` → kept, because the platform still holds the pending call.

## 7. Frontend

New domain `src/lib/domains/threads/`, which touches none of #1836's dirty files:

- `utils/reducer.ts`: pure `fromSnapshot`, `applyEvent`, `applyDelta` →
  `ThreadView { messages, streamingText, status, approvals, queue, hostRequests, usage, todos, error, canRetry, seq }`.
- `state/threadStore.svelte.ts`: a runes class with ref-counted per-thread subscriptions and the
  cursor/gap/lag rules from section 4.
- `api/threadApi.ts`, `generated/` (ts-rs), and `threads` keys in `shared/query/keys.ts`. The list
  is a TanStack Query; the live view is runes state seeded from the snapshot, as
  `agentRuns.svelte.ts` already is. No `writable()`.

Consumers:

- `CoderFeed` keeps its props. `CoderPanel` reads `threadStore.view(id)` instead of
  `coderSession.runtimeFor(id)`.
- `AgentSessionPanel` and `AgentsWorkspace` read the same view in place of
  `eventsToMessages(log)`. `eventsToMessages.ts` is deleted; `eventTimeline.ts` stays.

What leaves `coderSession.svelte.ts` (line numbers from the 2026-11-27 dirty tree):

- from `ThreadRuntime` (`:70-101`): `messages`, `streamingText`, `waitingSeconds`, `running`,
  `pending`, `error`, `canRetry`, `contextUsage`, `llmUsage`, `lastRetry`, `messageQueue`,
  `subAgents`;
- the `coderService.on*` wiring (`~:355-490`);
- `refreshRunning` and reattach (`:739-770`);
- queue drain (`:1011-1026`);
- optimistic `running` flips (`:1233-1380`);
- `inferPending`.

What stays: active thread, draft input, provider/model selection, `terminalOpen`,
`pendingPrompt`, multitask flags, global modes and rules, and `FileChange` bodies (refetched on
`ChangeRecorded`).

New `coder/services/hostResponder.ts` holds the `run_command` / `list_terminals` responder,
enabled only in the hub's desktop webview. Every write passes `crypto.randomUUID()` as
`command_id`.

UI touches are three: a "decided by <device>" label on resolved approvals, an "Already approved
on <device>" toast (`$lib/utils/toast`), and a "Stopping…" state.

## 8. Phone wake

What exists:

- The mobile shell posts a native notification from its Rust events bridge only while
  backgrounded, and only for `agents://event` and pipeline frames (`mobile/src-tauri/src/lib.rs`,
  `alert_for` at `:413`). A Portal-engine approval (`coder://pending`) never reaches a phone.
- No FCM messaging and no Android foreground service in `mobile/src-tauri` (the merged manifest
  only carries Firebase's component registrar from a dependency). The bridge lives only as long
  as Android keeps the process and LAN socket alive.
- PORTAL_MOBILE.md phase 3 ("Push when it matters", line 135) is not built beyond this; phase 4
  (relay) is not built.
- The desktop inbox is per-webview `localStorage` (`notificationInbox.svelte.ts:1-3`), not shared.

Phase 1 does three things:

- `alert_for` maps `approval_requested` and `turn_ended` on `coder://thread-event`, and drops the
  old `permission_request` arm in the same change.
- `appNotifications.ts` fires on `approval_requested` for both stacks.
- A reconnecting client lands on a `waiting` thread through catch-up.

A phone in Doze is not woken in Phase 1 (owner decision, 2026-11-27). The options, kept here so
the choice can be made later:

| option                                    | reaches the phone                       | needs                                                                                                                                                               | privacy                                                                                                          | downside                                                  |
| ----------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Foreground service holding the LAN socket | on the same LAN (or a VPN to it)        | an Android foreground service in `mobile/`, a declared service type on Android 14+ (`remoteMessaging` or `specialUse`; check Play policy), a permanent notification | nothing leaves the network                                                                                       | battery, always-on notification icon, useless off the LAN |
| FCM data message                          | anywhere with internet                  | a Firebase project, Play Services on the phone, a sender holding service-account credentials (the hub or a relay)                                                   | the wake goes through Google; keep it content-free ("a thread needs you") and fetch details over our own channel | Google dependency, credentials to guard                   |
| UnifiedPush (e.g. self-hosted ntfy)       | anywhere the push server is reachable   | a distributor app on the phone, an ntfy server (self-hosted or ntfy.sh)                                                                                             | no Google; self-hosted keeps it in-house                                                                         | one more server to run, extra app to install              |
| WorkManager periodic check                | every ≥15 min, deferred further in Doze | nothing new                                                                                                                                                         | nothing leaves the network                                                                                       | not real time; too slow for approvals                     |

**Pick it together with reach.** Waking a phone that is off the LAN only helps if it can then
reach the hub to approve, which needs PORTAL_MOBILE.md phase 4 (relay) or a VPN to the hub. So:
LAN-only → foreground service; anywhere → FCM or UnifiedPush plus a relay or VPN. Tracked on the
board under epic #1907.

## 9. Slices

| slice | board | owns                                                                                                                                         | delivers                                                                                                 | gate                                                      | deps                              |
| ----- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- | --------------------------------- |
| S1    | #1925 | `PCORE/threads/*`, migration 56 + `migrations/mod.rs`, `src-tauri/src/domains/threads/`, one line in `agents/service.rs` `spawn_pump`        | schema, actor, CLI-subset vocabulary, snapshot + events_since; hub only observes the CLI path            | `cargo test threads::` in `src-tauri/portal-core`, clippy | none                              |
| S2a   | #1926 | `PCORE/threads/*`                                                                                                                            | full vocabulary, client commands, dedup, first-wins, queue, `ThreadEffects`                              | threads tests                                             | S1                                |
| S2b   | #1927 | `src-tauri/src/rpc.rs`, `daemon/mod.rs`                                                                                                      | `Caller` device identity, forced-lag daemon test                                                         | `cargo test` in `src-tauri`                               | none (S1 for the catch-up assert) |
| S3a   | #1928 | `coder/service.rs`, coder `commands.rs`                                                                                                      | Portal engine through the hub, legacy dual-write kept                                                    | cargo test + parity test                                  | S2a, S2b                          |
| S3b   | #1929 | `agents/service.rs`, agents `commands.rs`                                                                                                    | CLI commands through the hub                                                                             | cargo test, replay fixtures                               | S2a, S2b                          |
| S3c   | #1930 | `threads/legacy_import.rs`, `src-tauri/src/lib.rs`                                                                                           | importer + startup reconcile                                                                             | fixture-DB test, second run is a no-op                    | S2a                               |
| S3d   | #1931 | `src/lib/domains/threads/*`, `shared/query/keys.ts`                                                                                          | reducer, store, API                                                                                      | `pnpm check && pnpm lint`, vitest                         | S2a                               |
| S3e   | #1932 | `mobile/src-tauri/src/lib.rs`                                                                                                                | phone alert on approval                                                                                  | cargo test/check in `mobile/src-tauri`                    | S1                                |
| S4a   | #1933 | `coderSession.svelte.ts`, `coderPanel.svelte.ts`, `CoderPanel.svelte`, `coderService.ts`, `hostResponder.ts`                                 | Coder frontend cutover                                                                                   | check, lint, vitest, scenarios 1-3                        | #1836, S3a, S3d                   |
| S4b   | #1934 | `AgentsWorkspace.svelte`, `AgentSessionPanel.svelte`, `agentRuns.svelte.ts`, `agentsApi.ts`, `appNotifications.ts`, `NotificationRow.svelte` | Agents cutover + notifications                                                                           | check, lint, vitest                                       | #1836, S3b, S3d                   |
| S5    | #1935 | review, then removal of legacy emits                                                                                                         | log reviewed; `coder://*` legacy, ring and `agents_get_events` removed on owner's yes; no tables dropped | all gates                                                 | S4 on real data                   |

Waves: S1 with S2b in parallel; then S2a; then S3a-e (five agents, disjoint files); then S4a with
S4b; then S5. Peak concurrency is five. S3c adds one `mod` line to `threads/mod.rs`; S2a
pre-declares it.

## 10. Test plan

**Rust, `PCORE/threads/tests`:**

- `replay_matches_projection`: script commands for both stacks, then fold `events_since(0)`
  through `apply` from empty state and assert equality with `snapshot`.
- `double_approve_applies_once`: two `Approve` commands with different `command_id`s give one
  `ApprovalResolved`, one effect, and a second ack of `already_resolved` naming the first actor.
- `duplicate_command_id_is_noop`.
- `send_while_running_enqueues`, plus the queue surviving an actor restart.
- `late_join_sees_partial`.
- `reconcile_tail_only`: a platform `done` equal to the live messages plus a final message gives
  `keep = n, tail = [final]`.
- `stale_running_reconciled_on_startup`.
- S3a parity: `thread_messages` equals `coder_threads.messages_json` after a scripted turn.
- S2b daemon: with a small channel, a slow client gets `events://lagged`, calls `events_since`,
  and ends equal to a fast client.

**Vitest, `threads/utils/reducer.test.ts`:**

- JSON fixtures generated by the Rust tests (events in, state out) for a Portal turn and a Claude
  run.
- Out-of-order frame, duplicate frame, and delta offset mismatch.

**Done-criteria scenarios** (CDP on :9222 plus a browser client over the daemon):

1. Reload the webview mid-turn: identical feed, queue intact.
2. Second client opens mid-turn, including with a forced lag (test-only small channel):
   converges.
3. Approve from both: one tool run, loser toast.

**The one check that fails if this is wrong:**
`cd src-tauri/portal-core && cargo test threads::tests::replay_matches_projection` (portal-core is not a member of the `src-tauri` workspace, so `-p portal-core` from `src-tauri` fails)

## 11. Risks and open questions

Risks:

- **Two logs for CLI runs.** `agent_events` (raw) and `thread_events` (normalised) both hold tool
  output. Accepted for Phase 1; repoint analytics later, or cap raw retention.
- **Dual-write window for Portal.** `coder_threads.messages_json` and `thread_messages` are both
  written from S3a to S5. The parity test is the guard, and S5 must not start before real-data
  use.
- **Verbatim tool output.** Large results sit in events and snapshots. Upgrade path: truncate in
  the event and add a `thread_blobs` table fetched on demand. Measure on the biggest real thread
  first.
- **`assistant` SSE semantics.** The code assigns rather than appends (`service.rs:2213`), so each
  frame is treated as a chunk (assumed). The offset rule is correct either way.
- **Migration number 56** may be taken by another branch.
- **Mailbox is unbounded.** A runaway provider grows memory; delta coalescing limits frames, not
  inputs.

Owner decisions (2026-11-27):

1. **Phone wake:** not in Phase 1; options documented in section 8 for a later pick.
2. **Caps:** one unified cap for every provider's thread transcripts, the default-granted `coder`
   (section 4).
3. **Modes:** agent mode and permission mode stay global in Phase 1.
4. **Vocabulary:** normalise at ingest (section 2).

Still open:

5. **S5 trigger:** how long on real data before legacy emits and the `messages_json` write are
   removed? Owner's call after S4.
