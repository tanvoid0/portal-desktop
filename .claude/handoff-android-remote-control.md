# Handoff: Portal Remote → Portal Desktop Rust port, Phase 1 (continued)

Written 2026-11-10, for someone with **no memory** of the session that produced it. This
supersedes the phase-1 section of the handoff this session started from (which was itself
pasted in from an earlier session). Do **not** confuse this with `.claude/handoff.md` in this
same worktree — that file is a leftover from an unrelated earlier task (coder-engine testing)
and should not be touched or treated as current.

## Goal

Replace PortalRemote.Server (C#) with Portal Desktop's Rust daemon as the thing the existing
Android app (`D:\projects\production\portal\portal_remote\android`, a separate repo, **read-only
reference — never edit it**) talks to. Phase 1 = pairing + a live control channel + LAN
discovery, enough that a phone can pair and show "Connected" (every feature tab still errors —
input/mirror/files/audio/casting are phases 2-6, not started).

## Where this lives

Branch `claude/portal-android-remote-control-63ea6a`, worktree
`D:\projects\production\portal\portal_desktop\.claude\worktrees\portal-android-remote-control-63ea6a`.
**Work here, not in the main checkout** — the main checkout (`D:\projects\production\portal\portal_desktop`)
has unrelated dirty state from a different, possibly-still-active session (QA/coder-engine work).
Board task: **#227** in the `portal` MCP tool, project `portal_desktop` — read it for the full
history, log milestones on it, keep its notes **short** (there's a long-standing bug, #50/#9,
where a note over ~200 chars makes the whole call fail with a misleading "title too long" error —
if that happens, just shorten the note and retry, don't treat it as Portal being down).

## What's done (backend — committed: `7d8f93fd`, `27acf98e`)

The Android app's wire protocol was read directly from its Kotlin source
(`...\portal_remote\android\app\src\main\java\com\portalremote\net\{PairApi,PairUrl,WsClient,
Discovery,Protocol}.kt`) rather than guessed — it's fixed, nothing about it changes. The real
contract turned out to have **no passcode step at all**, which is why the backend is a rewrite
of the pairing flow, not an addition to it:

- `POST /pair/request` (anon, body `{"device": name}`) holds the HTTP response open
  (long-poll, ~170s) until a human approves/denies on the desktop, then answers on that same
  connection: `200 {token, port, name}` or `403`. Implemented in `src-tauri/src/daemon/pairing.rs`
  via a `oneshot` channel registry (`PENDING`, keyed by device_id) that `approve_device`/
  `reject_device` resolve.
- `GET /control` (bearer-protected WS) sends `{"t":"hello",...}` unprompted on connect, then
  answers `{"t":"ping","seq":n}` with `{"t":"pong","seq":n}`; anything else gets
  `{"t":"error","message":"not implemented yet: <t>"}` rather than hanging. Reads the optional
  `X-Portal-Device` header to rename a device on first real connect. `src-tauri/src/daemon/control.rs`.
- UDP discovery responder on `:8765`, literal ASCII probe `PORTALREMOTE?` → JSON reply.
  `src-tauri/src/daemon/discovery.rs`.
- `daemon.rs` (was 755 lines) is now the `daemon/` module (`mod.rs` + `pairing.rs` + `control.rs`
  + `discovery.rs`). `daemon::spawn`/`daemon::DEFAULT_PORT` unchanged as the public API `lib.rs`
  calls.
- `pairing_service.rs` (`src-tauri/portal-core/src/domains/network/services/`) rewritten: three
  approval types now — `"temporary"` (24h), `"long_term"` (30d), `"until_revoked"` (no expiry,
  ends only via `revoke`). New: `request_pairing`, `mint_qr_pairing`, `revoke`, `reject`,
  `update_capabilities`, `rename_device`, `list_devices`/`DeviceSummary`. Deleted (every caller
  also deleted this phase): `verify_and_register`, `device_status`, `SHARED_PASSCODE_KEY`, the
  staged-token buffer, `network_service.rs`'s `PasscodeStore` entirely (confirmed zero remaining
  references before deleting).
- **Decisions already made, don't re-ask:**
  - *Token lifetime:* `until_revoked` was added specifically so a paired phone doesn't need to
    re-pair periodically; revoke-from-the-UI is the only way to end one.
  - *QR capabilities:* scanning a QR connects **immediately, no click on the phone, ever** (confirmed
    from `PairUrl.kt`/`WsClient.kt` — the QR's token is already a live credential). So capabilities
    for a QR-minted device are chosen by whoever generates the QR **at generation time**, in the
    same UI, not a fixed default and not full access.
  - *QR leak protection:* a QR-minted grant gets a 5-minute `expires_at` pickup window; the first
    device to actually authenticate with that token (in `verify_token`) has the expiry cleared in
    the same DB write, making it permanent. An unscanned/leaked QR image grants nothing after 5
    minutes; a real connect makes it permanent immediately. This is the one genuinely subtle piece
    of logic in the backend — read the doc comment on `mint_qr_pairing`/`verify_token` before
    touching either.
  - *LAN toggle:* promoted from env-var-only to a persisted setting (`settings_service.rs`,
    `remote_lan_enabled`) with Tauri commands `get_remote_lan_enabled`/`set_remote_lan_enabled`.
    **Flipping it does not take effect live — requires an app restart.** This was flagged as a
    phase-1 shortcut, not silently chosen: making it live needs `AppState` to hold a cancellable
    handle for the LAN HTTP listener + the UDP responder, which today are fire-and-forget spawned
    tasks. Fine to leave for a later phase unless it actually bothers someone.
  - *Known gap, not a phase-1 requirement:* a token revoked while its `/control` socket is already
    open does not drop that socket — only blocks a *new* connect. Noted in a comment in `control.rs`.
- `remote_list_capabilities()` command added (`src-tauri/src/domains/network/commands.rs`) —
  wraps `crate::rpc::capabilities()`, which already existed but had zero callers (its own doc
  comment named a caller, `remote_list_capabilities`, that had never been written). This is the
  authoritative capability list — use it from the frontend instead of hardcoding capability
  strings anywhere.
- Gates: `cargo check`/`cargo test --lib`/`cargo clippy --lib --tests` all clean on `src-tauri`
  (app crate) and `portal-core`, except one **pre-existing, unrelated** clippy failure in
  `portal-core` (`scan.rs:619`, an always-true `|| true` in markdown cleanup) — filed as board
  bug **#239**, not fixed here, don't let it block anything in this phase.
- One other pre-existing, unrelated bug found and filed while reading `app-shell.svelte`'s
  imports: board bug **#241** — `DeviceAuthGuard.svelte:74` calls a Tauri command
  (`verify_access_token`) that no longer exists on the IPC surface. It's dead code from the
  already-decided-against "route the webview over a browser" path (see
  `docs/development/REMOTE_ARCHITECTURE.md` phase 3). **Not part of this phase's work** — leave
  `<DeviceAuthGuard>` in `app-shell.svelte` completely alone, don't fix #241 as a drive-by.

## What's left (frontend — not started)

A full brief for this was handed to a proposed session (title "Devices page: pair/revoke UI for
Android remote control") but it may not have been started — **check board #227 and this worktree's
git log before assuming it's done or in progress.** If it was never started, here's the shape of
the remaining work:

1. Delete `src/lib/components/QRCodeDialog.svelte` and `DeviceApprovalDialog.svelte` (read both
   first — they show the existing `device://pending` event-listening pattern and old capability
   UI to mirror). Remove their imports/state/trigger-buttons/render-tags from
   `src/lib/components/layout/app-shell.svelte` (search for `QRCodeDialog`/`DeviceApprovalDialog`
   — four or five spots). **Do not touch `<DeviceAuthGuard>`** in the same file (see #241 above).
2. Delete `get_pending_device_approvals` from `src-tauri/src/domains/network/commands.rs` + its
   registration in `lib.rs` — its only caller is `DeviceApprovalDialog.svelte`, which step 1 just
   deleted. (The new page uses `list_devices` instead, which already returns every approval_type.)
3. New frontend domain `src/lib/domains/devices/` (components/api/queries/types/generated — the
   `generated/` dir and `DeviceSummary.ts` already exist from the backend work: run `pnpm gen:types`
   again if they look stale after a Rust change). A new management page (check `src/routes/credentials/`
   or `src/routes/settings/` for this repo's folder convention for a top-level page) using the shared
   shell (`$lib/components/shell` — `PageHeader`/`PageStats`/`PageFilters`/`PageLoading`/`PageError`/
   `PageEmpty`) per `.cursor/rules/page-shell-conventions.md`. Add a nav entry in `$lib/config/main-nav.ts`.
4. Wire up: `list_devices`, `approve_device` (now 3 approval_type options, not 2),
   `reject_device`, `revoke_device` (needs a confirm step, not raw `confirm()`),
   `update_device_capabilities`, `generate_pairing_qr` (capabilities chosen *before* the QR is
   drawn — render `qr_url` as an actual QR code, check for an existing QR library before adding
   one), `remote_list_capabilities` (the capability checkbox source of truth),
   `get_remote_lan_enabled`/`set_remote_lan_enabled` (UI must say the restart caveat out loud).
5. Gates: `pnpm check && pnpm lint` must pass. `pnpm tauri:dev` may not start cleanly in this
   environment (an unrelated, pre-existing vite/vitest issue noted in an earlier session) — that's
   not a blocker, `pnpm check`/`pnpm lint` are the real gates.
6. Commit when green. **No AI attribution line in any commit** (`Co-Authored-By`, "Generated
   with..." — this user's global CLAUDE.md forbids it everywhere, every repo). Update/close board
   #227.

## Things that bit this session (so they don't bite the next one)

- **`cargo test` re-exports every ts-rs type in the crate, unprettified.** `scripts/gen-types.mjs`
  runs `cargo test ... export_bindings`, which matches ts-rs's auto-generated per-type
  `export_bindings_*` test functions — but a *plain* `cargo test --lib` (no filter) also runs all
  of them, as a side effect, re-exporting every `#[derive(TS)]` type project-wide in raw
  (non-prettier) form. This makes `git status` show ~90 unrelated `generated/*.ts` files as
  "changed" (cosmetic quote-style diffs) after any full test run. **Don't hand-revert these** —
  just run `pnpm gen:types` properly once at the end (it re-exports *and* prettifies), and the
  unrelated files go back to byte-identical with what's committed. Confirmed this works.
- **A new ts-rs-exported type needs its domain added to `GENERATED` in `scripts/gen-types.mjs`**
  (the barrel-index array near the top) or its `index.ts` barrel never gets written. Added
  `'src/lib/domains/devices/generated'` this session.
- **`ts_rs` is only a dependency of the `portal-core` crate, not `src-tauri`.** A struct in
  `src-tauri/src/domains/*/commands.rs` (the Tauri adapter layer) cannot derive `TS` — it'll fail
  to compile (crate not found). Response/request structs at that layer stay untyped on the wire by
  convention (frontend types them by hand); only `portal-core` service-layer types (like
  `DeviceSummary`) get real generated TS. Caught this before committing, not after.
- **D: drive is tight.** It hit 100% full (0 bytes free) mid-`cargo test` once already this phase,
  which silently kills the link step. This worktree's own `target/` is safe to clear (pure build
  cache, ~16GB) if it happens again — don't touch `target/instance-2` or any other worktree's
  `target/` without checking nothing is actively using it first. Was at ~3GB free as of this
  writing; worth checking `df -h /d` before another full `cargo test`.
- A fresh agent/session needs the exact file paths and command signatures spelled out — "match the
  Android contract" by itself is not enough context to act on. The Kotlin file:line citations in
  the git log of commit `7d8f93fd`'s message (and in this file's "What's done" section above) are
  the ground truth if anything here seems to contradict the actual code.
