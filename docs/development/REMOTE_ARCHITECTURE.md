# Remote architecture — making Portal Desktop's features reachable from anywhere

**Status:** phases 0, 1, 2 and 3 done — phase 3 with one part deliberately not
done, see it below. Phases 4 and 5 not started.

**Phase 2 ran before phase 1.** The crate split only buys a compiler-enforced
boundary and the option of a headless binary — the daemon works inside
`src-tauri` today, and it is the part that actually delivers remote access.
Building it first also validated the phase 0 sink design before 350 files move.

All 423 commands are converted to `#[rpc]` and verified reachable against a
running app: `POST /rpc/coder_list_threads` returns the real thread list from the
database over HTTP.

## The goal

Portal Desktop is the hub: it holds the database, the agent-platform connection, the
pty, git, and the coder threads. Everything else — the desktop window, an Android
phone, a browser, agent-platform itself — should be a _client_ of that hub, not a
special case built into it.

Today only one client exists (the Tauri webview), and it reaches the hub over Tauri
IPC, which nothing else can speak.

## What's already here, and what was broken

The pairing half is finished, in Rust: `generate_device_passcode`,
`verify_device_passcode`, `approve_device`, `verify_access_token`,
`get_pending_device_approvals`, `get_local_network_ip`
([`domains/network/commands.rs`](../../src-tauri/src/domains/network/commands.rs)),
plus `QRCodeDialog.svelte` and `DeviceAuthGuard.svelte`.

**What was missing was the listener**, and it could not simply be added where the
client expected it. [`invokeClient.ts`](../../src/lib/utils/invokeClient.ts) routed
three ways — Tauri → `invoke()`, localhost → HTTP, remote →
`POST /api/tauri/{command}` with a `Bearer` token — but:

1. `src/routes/api/tauri/[command]/+server.ts` returned `501`, with a comment
   saying a Tauri HTTP server plugin is required.
2. [`svelte.config.js`](../../svelte.config.js) uses `adapter-static` in SPA mode.
   **SvelteKit server routes are not part of the Tauri build at all.** Every file
   under `src/routes/api/` was dead in production. Anything server-side has to live
   in `src-tauri`.

So `invokeClient` advertised a remote path that could never answer.

The listener now exists (phase 2), and `src/routes/api/` is deleted — all three
`+server.ts` files, none of them reachable. One of them,
`api/terminal/execute/+server.ts`, `spawn()`ed a command string straight from the
request body; unreachable under `adapter-static`, but one adapter change away from
being remote code execution, which is reason enough not to leave it lying there.

The browser branches that pointed at the deleted proxy now throw a message saying
so, rather than failing on an HTML parse error. Repointing them at the daemon was
considered and decided against — see phase 3.

## The shape

Stop treating Tauri IPC as the API. Make HTTP/WS the only API, and let the Tauri
window be a client of it like everything else.

```
crates/
  portal-core/     domains/, entities/, migrations/    — no tauri dependency
  portal-daemon/   axum over portal-core               — the actual product
src-tauri/         webview shell + window-bound plugins only
```

One contract, one auth path, one event stream. No client gets a curated subset, so
no client can drift out of sync with the server.

### Why this is cheaper than it looks

Measured before any of it started, which is what made the plan look affordable:

| Fact                      | Count                                            |
| ------------------------- | ------------------------------------------------ |
| `.rs` files               | 373                                              |
| Files importing `tauri::` | **33**                                           |
| `#[tauri::command]`       | 350 — all in `commands.rs` adapter files         |
| `.emit()` call sites      | 61, across 9 files                               |
| Service construction      | plain `Arc::new(X)` → `app.manage()` in `lib.rs` |

91% of the Rust already had no idea Tauri existed, and the services were already
built standalone before being handed to `.manage()`. The coupling was two things:
`State<'_, Arc<T>>` in the command adapters (mechanical), and the 61 `.emit()`
calls (needed an abstraction — that was phase 0).

That estimate held. After phases 0–2:

| Fact                                | Count                                               |
| ----------------------------------- | --------------------------------------------------- |
| `.rs` files                         | 408                                                 |
| `tauri::` anywhere in `portal-core` | **0**                                               |
| `portal-core/src/domains`           | 261 files — every service, entity, repository, type |
| `src-tauri/src/domains`             | 59 files — 32 command adapters, 27 shim `mod.rs`    |
| `#[rpc]` commands                   | 422                                                 |
| `.emit()` still in `commands.rs`    | 0                                                   |

## Phases

Each phase ships on its own and nothing is thrown away by a later one.

### Phase 0 — event sink `[done]`

Replace direct `AppHandle`/`Window` `.emit()` calls with an `EventSink` trait.
`TauriSink` forwards to the webview exactly as today; phase 2 adds a WebSocket
sink and a fanout over both.

Scope: the service/manager layer only — the files that must eventually move to
`portal-core`. The 11 emit sites inside `commands.rs` files (`domains/ai`, `domains/disk`) stay as-is; those
files are the Tauri adapter by design and are dealt with in phase 2.

| File                                                       | Sites |
| ---------------------------------------------------------- | ----- |
| `domains/coder/service.rs`                                 | 24    |
| `domains/kubernetes/manager.rs`                            | 9     |
| `domains/sdk/ollama_manager.rs`                            | 7     |
| `domains/terminal/manager.rs`                              | 6     |
| `domains/projects/pipelines/services/execution_service.rs` | 2     |
| `utils/logger.rs`                                          | 1     |

Also removes the one non-emit `AppHandle` use in `coder/service.rs`
(`service_arc()` fetches `Arc<Self>` back out of Tauri state — replaced with a
`Weak<Self>` set at init).

The webview receives the same events on the same names. Two things worth knowing
about the result:

- `EventSink::emit_json` returns `bool` — false meaning nobody received it. Only
  one caller reads it: the pty reader thread in `terminal/manager.rs` used
  `emit(..).is_err()` to stop itself when the webview went away, and without a
  delivery signal that thread would spin forever. In phase 2 the fanout sink
  should return true if _any_ transport took it, so a phone keeps a terminal
  alive with the desktop window closed.
- `utils::logger::set_event_sink` is now wired in `lib.rs`, where the old
  `set_app_handle` call had been commented out as `TEMPCHECK`. `Logger::log`
  holds the global `std::sync::Mutex` while emitting, so **a sink that logs on
  its own failure path will deadlock** — `std::sync::Mutex` is not reentrant.
  Phase 2's WebSocket sink must not use `log_*!` macros in its send path.

### Phase 1 — workspace split `[done]`

`src-tauri/portal-core`, a path dependency with no `tauri` line in its
`Cargo.toml` and a comment saying there must never be one. That absence is the
whole mechanism: the boundary is now a compile error, not a review convention.

`src-tauri/src` is down to `daemon.rs`, `domains/`, `events.rs`,
`invoke_handler.rs`, `lib.rs`, `main.rs`, `rpc.rs`.

Moved in slices, each one compiling on its own: `events` first (everything else
emits through it), then `entities` + `migrations`, then the leaves
(`error`, `process_ext`, `command_executor`, `app_paths`, `database`, `utils`),
then `domains`.

**The shim is what kept it cheap.** `lib.rs` re-exports each moved module under
its old path, so `crate::entities::…`, `crate::utils::logger::…` and
`crate::domains::coder::…` all still resolve and no call site had to be
rewritten:

```rust
pub(crate) use portal_core::{app_paths, command_executor, database, entities, error, process_ext, utils};
pub(crate) use portal_core::{log_debug, log_error, log_info, log_warn};
```

The macro line matters as much as the module line: `log_*!` are
`#[macro_export]`, so after the move they live at `portal_core`'s root, and
re-exporting them under `crate::` is what kept the six files that import them
untouched. Each domain gets the same treatment — `src-tauri/src/domains/<d>/mod.rs`
glob re-exports `portal_core::domains::<d>` and adds `pub mod commands;`, which
is why `lib.rs`'s 471-line `generate_handler!` list needed no edits at all.

Drop a line from those `use` lists once nothing in `src-tauri` refers to that
module any more; `migrations` already went that way when `database.rs` moved.

**Command files cannot move.** `#[rpc]` expands to `#[tauri::command]` plus glue
that calls `tauri::Manager::state()`, so the 32 adapter files stay. That is the
boundary, and it is now visible in the directory layout.

**Three couplings the compiler found** that no review had:

- `sdk/services/language_config_service.rs` imported from
  `sdk/commands/manager_commands.rs` — a service depending on its own Tauri
  adapter. The 259 lines of pure workflow logic became
  `portal-core/src/domains/sdk/services/manager_workflow.rs`; the adapter calls
  down into it.
- `tasks/services/ai_task_generator.rs` imported `TaskContext` and four sibling
  structs from `tasks/commands.rs` — plain serde shapes declared in the adapter,
  now `portal-core/src/domains/tasks/types.rs`.
- Two `pub(crate)` items used across what is now a crate line:
  `KubernetesManager::current_cluster`, and `environment`'s `platform`/`types`
  modules. Widened to `pub`.

`dashboard` turned out to be commands-only, so it has no core half — the shim
there is a bare `pub mod commands;` rather than a glob re-export of nothing.

**Side effect worth knowing:** `cargo test --lib` had been failing to load its
harness outright (`STATUS_ENTRYPOINT_NOT_FOUND`, the `inventory` +
`crate-type = ["staticlib", "cdylib", "rlib"]` combination). The split fixed it —
65 of the tests now live in `portal-core` and run there, and the 7 left in
`src-tauri` load again. If it ever returns, `tests/` builds its own binary and is
immune; `tests/bundle_identifier.rs` is there for that reason as well as for
layering.

### Phase 2 — daemon

axum in [`src-tauri/src/daemon.rs`](../../src-tauri/src/daemon.rs), spawned as an
in-process tokio task rather than a separate binary: nothing to supervise, no
orphan process, no port-conflict dialog on launch. A bind failure is logged and
the app carries on with the webview transport alone.

**Done:**

- `GET /health` — name and version
- `GET /events` — WebSocket, fed by a `WsSink` over a `tokio::sync::broadcast`
  channel. `lib.rs` now wires a `FanoutSink` over `[TauriSink, WsSink]`, so the
  webview and every attached client see the identical stream.
- binds `127.0.0.1:18420` (`PORTAL_DAEMON_PORT` to override), clear of
  agent-platform's 18410
- frames are `{"event": "...", "payload": ...}`, using the same event names as
  the Tauri side so one client can subscribe by either route
- a lagging client gets an `events://lagged` notice and resumes at the present
  rather than being disconnected

**Trust boundary as it stands.** Loopback keeps other machines out. It does _not_
keep a web page out: a browser will open `ws://127.0.0.1` from any origin and
WebSockets are not subject to CORS. So `/events` refuses any upgrade carrying an
`Origin` header — native clients send none, browsers always do and cannot be
talked out of it. That guard has an end-to-end test; it is the only thing between
a visited page and this machine's event stream, so it must keep one.

**`POST /rpc/{command}`.** Tauri offers no way to invoke a command by name from
Rust, so serving the command surface over HTTP needs a second entry point per
command. Writing those by hand would be 345 shims and a standing invitation to
drift — the remote surface would quietly become whatever subset somebody
remembered to wire up. So the two entry points are one edit instead:
[`rpc-macro/`](../../src-tauri/rpc-macro/src/lib.rs) provides `#[rpc]`, which
replaces `#[tauri::command]` and additionally registers a dispatch entry.

Parameters are classified by type: `State<'_, T>` resolves from managed state
(the same instance IPC would get), `AppHandle` gets the handle, anything else is
deserialised from the JSON argument object by name — accepting the camelCase
spelling the frontend already sends as well as snake_case. A command taking
`Window` cannot be served without one, so it is a compile error until marked
`#[rpc(local_only)]`; being absent from the remote surface has to be a decision,
not an oversight.

Entries are collected by `inventory` at link time, so there is no central list to
keep in step. That mechanism fails by producing an _empty_ registry rather than a
build error, which at runtime looks exactly like "the daemon serves nothing" —
hence the test in [`rpc.rs`](../../src-tauri/src/rpc.rs) asserting a known
command is present.

Conversion was a mechanical rename of `#[tauri::command]` and the short
`#[command]` form. All 423 are done. One command needed a real change: `greet`
took `&str`, and a borrowed parameter cannot be deserialised from a request body
— the macro now rejects references with that sentence, because serde's own error
("implementation of `Deserialize` is not general enough") names neither the
parameter nor the fix.

Two registry guards worth keeping: a floor on the registered count, so reverting
a domain's attributes fails a test rather than silently shrinking the remote
surface, and a uniqueness check, since two commands under one name would resolve
to whichever the linker happened to order first.

The emit sites left in `commands.rs` are done too — the six disk commands and the
AI streaming command took an `AppHandle` purely to emit, and now take
`State<'_, Events>` and push through the fanout, so a remote client sees
scan/quarantine/dev-clean progress and the AI token stream rather than only the
webview seeing them. No `.emit()` on a Tauri handle is left anywhere in
`src/domains`.

One semantic shift worth recording. `EventSink::emit` answers `bool`, not
`Result`, so the two AI sites that used to abort the turn on a Tauri emit error
now abort when _no transport took the frame at all_:

```rust
if !chunk_sink.emit(&format!("ai-stream-chunk-{stream_id_clone}"), &chunk) {
    return Err(AIError::GenericError("no client is receiving this stream".to_string()));
}
```

That is a wider trigger than before — a closed webview with no WebSocket client
attached now stops generation, where previously Tauri's emit would usually still
succeed. It is the same call the pty reader makes, and continuing to burn tokens
into an audience of nobody is not worth preserving. But it does mean a client
that reconnects mid-stream has lost the turn, not just the frames it missed.

### Phase 3 — collapse `invokeClient` `[done, except the part we decided against]`

The goal was: delete the three-way branch, always HTTP to the daemon, `invoke()`
only for window-bound plugins.

**Done, by deletion rather than unification.** The `501` handler and the whole
`src/routes/api/` tree are gone; so are the browser streaming transports, which
routed to `/api/stream/{cmd}` (SSE) and `/ws/{cmd}` (WebSocket) — neither endpoint
ever existed, both would have had to be SvelteKit server routes, and
`adapter-static` never built those. That was ~230 lines of `invokeClient`
addressing URLs that could not answer. What remains of the branch is a guard that
says so.

`src/hooks.server.ts` went with them. It advertised itself as "blocks unauthorized
browser access at the server level" and both of its branches returned
`resolve(event)` unchanged — it blocked nothing, and `isLocalhostRequest` was
imported and never called. A file claiming a control nobody has is worse than no
file. `src/lib/utils/serverUtils.ts` was its last consumer.

**Decided against: routing the webview over HTTP.** `daemon.rs::browser_origin`
refuses any request carrying an `Origin` header, on both `/events` and `/rpc`:

```rust
if headers.contains_key(axum::http::header::ORIGIN) {
    return (StatusCode::FORBIDDEN, "browser origins are not permitted on this port");
}
```

That is what stops any web page the user has open from reaching a port exposing
terminal execution and the credential vault, and WebSockets ignore CORS so a
same-origin check is the only lever available. The webview is served from
`http://tauri.localhost`, so its `fetch` to `127.0.0.1:18420` is cross-origin and
the browser attaches `Origin` whether the page wants to or not — the refusal
catches the webview along with everything else.

Lifting it would need an explicit allowlist. We are not doing that, because the
reasons for it got smaller on inspection:

- **The stated goal is already met.** "No client gets a curated subset" is true
  today: `/rpc` serves all 422 commands to any non-browser client — a script, CI,
  agent-platform. The webview using IPC restricts nothing.
- **The remaining benefit was tidiness in one file**, and most of that arrived
  through deletion instead.
- **The cost is permanent.** IPC is faster than a loopback round trip, so this
  would be a small regression on every command forever, for no user-visible gain.
- **The allowlist is platform-dependent** — `http://tauri.localhost` on
  Windows and Linux, `tauri://localhost` on macOS — so it is three strings that
  must stay correct indefinitely on the one port where being wrong is expensive.
- **Phase 4 does not need it.** A phone is not a page on `tauri.localhost`; it
  needs pairing and tokens regardless, and origin-vs-token policy should be
  settled there as one question rather than pre-empted here.

So IPC stays the webview's transport. Revisit only if running Portal's UI in a
real browser becomes a requirement; nothing else is blocked on it.

Type contract: generate the TS types from the Rust structs so the client cannot
drift. `ts-rs` is the intended pick — build-time only, no runtime dependency, no
framework lock-in. Confirm the current version before committing to it. Independent
of the origin question, so it can go first.

### Phase 4 — pairing and LAN

The existing passcode/approval commands become daemon middleware instead of Tauri
commands. Add a UDP discovery responder (~80 lines, port of Portal Remote's
`Pairing/DiscoveryResponder.cs`). Bind `0.0.0.0` only when explicitly enabled.

**Security gate — this phase does not ship without capability scoping.** Portal
Remote's model is a single shared token where holding the token is equivalent to
sitting at the PC. That is a reasonable trade for a media remote. It is not
acceptable here: this daemon exposes terminal execution, git write access, and a
credential vault backed by the OS keychain
([`credentials/services/credential_service.rs`](../../src-tauri/src/domains/credentials/services/credential_service.rs)).

Required before any `0.0.0.0` bind:

- a capability attribute on each route at registration time, denied unless listed
- credentials and terminal-exec domains non-remotable by default, opt-in per device
- a phone token grants a route subset, never the whole `/rpc/*`

### Phase 5 — Android client

Point Portal Remote's existing Android app at the daemon. Its `net/WsClient.kt`
and `net/PairApi.kt` already model this exact flow — discover, pair, hold a token,
one WebSocket plus plain HTTP for bulk. New tabs for coder and terminal; the rest
of the app is unchanged.

## What happens to Portal Remote

Its Windows-only half — Win32 `SendInput`, screen mirroring, WASAPI loopback, the
WinRT media session, DLNA/Roku casting — does not belong in a cross-platform core.
Two options:

- **(a) keep it as an optional Windows sidecar**, with the daemon proxying `/win/*`
  to it. No rewrite; two installers.
- **(b) port input and mirroring to Rust** behind `cfg` gates in a `portal-input`
  crate. Cross-platform; a rewrite plus hardware re-verification.

Take (a). Revisit (b) when someone actually wants a trackpad on macOS.

## Ordering note

Phases 0–3 are worth doing even if the phone never happens: they delete the dead
remote path and leave a headless daemon that CI, a script, or agent-platform can
drive. Phases 4–5 are what make the phone work, and phase 4 is gated on the
security work above.

## Landmine: `cargo test` and the Windows manifest

If `cargo test` starts failing with

```
process didn't exit successfully: ...exe (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```

while `cargo check` and `cargo build` are clean, the cause is in
[`build.rs`](../../src-tauri/build.rs), not in the test.

`tauri_build::build()` embeds a Windows manifest into the _application_ binary,
and one thing that manifest asks for is comctl32 version 6. The dialog plugin
imports `TaskDialogIndirect`, which only version 6 exports — bind against the
default 5.82 and the loader kills the process before `main` runs. Test harnesses
are separately linked binaries that never saw that manifest, so `cargo test`
breaks while `cargo build` stays fine. It surfaces only once the linker has
reason to retain the dialog code path, which is why it can appear to arrive out
of nowhere after an unrelated change.

`build.rs` fixes it with a `/MANIFESTDEPENDENCY` link arg. Two traps if you touch
that code:

- **Do not add `/MANIFEST:EMBED`.** Tauri's manifest is already embedded as a
  resource, and a second one fails the app link outright with
  `CVT1100: duplicate resource` / `LNK1123`. Left external, link.exe writes
  `<binary>.manifest` beside each output; Windows reads it for the test harness,
  and the app binary keeps Tauri's richer embedded one.
- **`cargo:rustc-link-arg-tests` does not work here.** It applies to declared
  `[[test]]` targets, and this package has none — the unit tests live in the lib.
  Cargo rejects it with "does not have a test target". The unscoped
  `cargo:rustc-link-arg` is what reaches the lib's test harness.

To find the offending symbol next time, dump the binary's imports and check each
against what the DLL actually exports; the answer here was
`comctl32.dll -> TaskDialogIndirect`.
