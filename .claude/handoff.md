# Handoff — Android remote-control: merged to main, live-tested, one UI tweak pending

Written 2026-11-11, for someone with **no memory** of this session. Supersedes the previous
version of this file (that was about unrelated coder-engine work; see git history if needed).

## Goal

Test the Android remote-control pairing feature (Phase 1: phone pairs with desktop, shows
"Connected") end to end with a real phone connected over wifi adb.

## State: on `main`, pushed? **no** — nothing pushed this session

Three commits landed on `main` (local only):
1. `c5de8161` — unrelated dirty work that was sitting in the main checkout (new `agents`
   domain: Run launcher, backend/process/service layers; QA-106 filed). Committed first
   because it had to land before the merge (same files the merge touched).
2. `c6cae484` — merge of `claude/portal-android-remote-control-63ea6a` into `main`. One
   trivial conflict in `src/lib/domains/shared/query/keys.ts` (both branches added a new
   top-level key) — resolved by keeping both.
3. `ade510ed` — real bug fix found while verifying the merge: `devices/+page.svelte` and
   `LanToggleCard.svelte` did `createQuery(createDevicesQuery)` instead of
   `createDevicesQuery()` — the query hooks already call `createQuery` internally, so this
   was double-wrapping and produced 14 `pnpm check` errors. Fixed both call sites, reran
   `pnpm gen:types` to reformat the generated TS bindings that had drifted to raw
   (unprettified) form from an earlier `cargo check`/`cargo test` run.

All gates green on `main` as of `ade510ed`: `cargo check` 0, `cargo clippy --all-targets` 0,
`pnpm check` 0 errors, `pnpm lint` 0 errors (prettier+eslint+shadcn-check all clean).

**Live pairing test: succeeded**, but via the manual flow, not QR:
- Desktop: `node scripts/dev-instance.mjs --kill-port 1` then
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222" pnpm run tauri dev`
  (plain `pnpm run tauri dev` does **not** get you CDP — that env var is what `dev-instance.mjs`
  sets internally; bypassing the script means setting it yourself). `remote_lan_enabled` was
  flipped to `true` in `%APPDATA%\portal_desktop\settings.json` (`app.remote_lan_enabled`,
  requires an app restart to take effect — it's not live-reloadable, documented as a known
  phase-1 shortcut).
- Phone (`Tanveer's S26 Ultra`, Samsung, already connected over wifi adb): opened the
  `portal_remote/android` debug build (already installed — see below), it discovered the
  desktop and showed "Waiting for CRYOSTATION" (the desktop's hostname).
- Approved from the desktop's **Devices** page (`/devices`) by clicking Allow on the pending
  request — confirmed "Device approved — Tanveer's S26 Ultra" toast, device now shows in the
  Paired table.
- Did **not** independently re-verify the `/control` WS ping/pong after approval — the phone
  user picked the phone back up (opened Camera app) before that could be checked. If you need
  to confirm the WS is actually alive, check `dumpsys activity` / logcat on the phone, or the
  desktop daemon's stdout, for a `hello`/`pong` exchange.

**QR pairing: does NOT work via the phone's stock Camera app** — and that's expected, not a
bug. The QR encodes a custom `portalremote://host:port/token` URI (see
`portal_remote/android/.../net/PairUrl.kt`); Samsung's (and most stock) camera QR scanners
only auto-resolve `http(s)://` links to an app, so it says "no apps can use this." To pair via
QR, scan it from **inside the PortalRemote app's own QR scanner**, not the system camera. If
stock-camera QR scanning is actually wanted, that needs a verified Android App Link
(`https://` URL with an `assetlinks.json`) redirecting into the app — a separate, unscoped
task, not started.

## What's left

1. **Uncommitted right now**: `src/lib/domains/devices/components/CapabilityChecklist.svelte`
   — added a "Select all / Deselect all" button and switched the per-capability `Checkbox` to
   the iOS-style `Switch` component, per the owner's explicit ask mid-session. Verified
   visually via a CDP screenshot (looked right) and `pnpm check` passed 0 errors right after
   the edit. **`pnpm lint` was still running in the background when the session ended** —
   confirm it's clean (it almost certainly is, the edit only touches one file and uses
   existing `$lib/components/ui/switch` + `$lib/components/ui/button`), then commit.
2. Decide whether to push `main` (3 commits, local only) — the owner was asked and hadn't
   answered yet when this session ended.
3. Optional: independently confirm the `/control` WS actually stays connected post-pairing
   (not just that approval succeeded) — wasn't finished, see above.
4. Optional/unscoped: Android App Links for stock-camera QR scanning, if wanted.

## Decisions made, don't re-litigate

- **Committing the unrelated `agents` domain work first was intentional**, not scope creep —
  the owner explicitly asked for it ("the other changes in the main tree is also required, so
  if not committed, commit first") before the merge, to avoid duplicating node_modules/target
  across a worktree and the main checkout (disk was at 100% full, see below).
- **The original worktree is deleted.** It lived at
  `.claude/worktrees/portal-android-remote-control-63ea6a` and hit two real, separate bugs
  before being abandoned in favor of merging to main:
  - Windows `MAX_PATH` (260 chars): the worktree's deep nesting pushed pnpm's long
    content-hash paths (e.g. `vitest@5.0.5_@types+node@26.6.5_@vitest+browser-playwright@...`)
    past 260 chars, breaking Node's own package self-import resolution
    (`#module-evaluator` in `vitest`, `#internal` in `@sveltejs/kit`) with a misleading
    `ERR_PACKAGE_IMPORT_NOT_DEFINED`. **`LongPathsEnabled=1` in the registry did NOT fix
    this** (verified empirically — Node's resolver doesn't honor it here); only an actually
    shorter path fixed it. If this bites again in some other deeply-nested worktree, don't
    waste time on the registry key.
  - **D: drive hit 100% full** mid-session (this is a recurring, documented problem — see
    `[[current-session-state]]`-style memory and the prior handoff this superseded). A
    worktree's own `src-tauri/target/` is safe to clear (pure build cache); do NOT touch
    another worktree's or the main checkout's `target/` without checking nothing is using it.
  - Net effect: don't recreate a worktree for this branch, it's merged. If a similar
    situation comes up again, prefer merging over deep worktree nesting, or put worktrees
    directly under a short root (e.g. `D:\wt\<name>`) from the start.
- **Old `PortalRemote.exe` (the C# server being replaced) was stopped** and should stay
  stopped when testing the Rust daemon — it held UDP/TCP `:8765`, which the new daemon's
  discovery responder also needs.
- No AI attribution was added beyond what the global CLAUDE.md allows (the system-reminder's
  `Co-Authored-By: Claude Sonnet 5` line) — already applied to all three commits.

## Files that matter

- `src/lib/domains/devices/components/CapabilityChecklist.svelte` — the uncommitted change.
- `src/routes/devices/+page.svelte`, `src/lib/domains/devices/components/LanToggleCard.svelte`
  — the double-wrapped-query-hook fix (`ade510ed`), already committed.
- `src-tauri/src/daemon/` (`mod.rs`, `pairing.rs`, `control.rs`, `discovery.rs`) — the merged
  Rust daemon; `mod.rs`'s `event_cap` now has both the `agents` branch (from main) and the
  pairing/control restructuring (from the feature branch), auto-merged cleanly by git.
- `%APPDATA%\portal_desktop\settings.json` — `app.remote_lan_enabled` is currently `true` on
  this machine (left on after the live test; flip back to `false` if that's not wanted
  persisted).
- `D:\projects\production\portal\portal_remote\android` — separate repo, read-only reference,
  the Android client. Debug build is installed on the test phone (replaced whatever
  production build — v11 — was there before; that's gone, only the debug build remains).

## Things that bit this session

- `pnpm run tauri dev` directly does **not** give you CDP on `:9222` — only
  `node scripts/dev-instance.mjs <N>` sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` for you.
  Set it yourself if bypassing the script.
- `git worktree move`/`mv`/`rm -rf` all failed with "busy"/"permission denied" on specific
  subdirectories of the worktree (`src`, `.claude`, `.cursor`, `docs`, `scripts` — notably
  **not** `node_modules`, `src-tauri`, `.git`) even after the worktree was unregistered from
  git. Root cause never fully confirmed; theory is a live file-watcher (possibly this very
  Claude Code host app's own file pane) holding directory handles open. Workaround: copy
  (not move) the directory, then `git worktree repair` to fix the gitdir cross-references —
  copy isn't blocked by whatever holds the rename/delete lock.
- A `cargo test`/`cargo check` run can leave the generated TS bindings in a raw
  (unprettified) state (`src/lib/domains/*/generated/*.ts`) — this is normal, not a merge
  artifact. Fix with `pnpm gen:types`, never hand-format.
