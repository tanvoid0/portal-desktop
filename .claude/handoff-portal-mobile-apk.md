# Handoff — Portal Mobile (the "pairing app"): Phase 0 built, installed, running on the real phone

Written 2026-11-12 for someone with **no memory** of this session. Scoped to the Portal Mobile
Android app (the Tauri 2 mobile target in `mobile/`). **Do not confuse with `.claude/handoff.md`**
or the other `.claude/handoff-*.md` files — this checkout is heavily multi-session; `git status`
before assuming any unfamiliar file is stray.

## Goal

Review the Portal Mobile plan (`docs/development/PORTAL_MOBILE.md`) and implement it. The plan's
Phase 0 (shell + connect) draft was already written and committed to `main` (commit
`7aeed3da feat(mobile)`), but the doc said it was "unverified, never compiled to completion."
Task became: verify it, get the Android APK to actually build and run, then test pairing.

## What's done (all verified this session)

1. **Reviewed Phase 0** — the draft is complete and correct. All wire contracts match between the
   mobile Rust shell and the daemon: `/whoami` (`src-tauri/src/daemon/mod.rs:389`, returns
   `{name,version,local,device_id,caps}`), `/events` frames (`{event,payload}`,
   `mod.rs:131`), `/rpc/{cmd}`, `/pair/request` (long-poll), UDP discovery `:8765`.
   Frontend glue wired: `src/routes/+layout.svelte` (connect gate), `app-shell.svelte` (nav
   cap-filter + disconnect), `notify.ts` (guarded), `app.html` (viewport-fit), `app.css`
   (`.portal-mobile` 16px inputs).
2. **Gates green**: `pnpm check` 0 errors, `pnpm lint` 0 errors. Mobile crate `cargo check`
   clean on **host + aarch64-linux-android + x86_64-linux-android** (added those rustup targets).
3. **Fixed two real build-blocking bugs** (the draft never built far enough to hit them):
   - `mobile/src-tauri/tauri.conf.json`: `node ../frontend.mjs` → `node frontend.mjs`. Tauri runs
     before{Dev,Build}Command from the app root (`mobile/`), so `../` pointed one level too high
     and the frontend build never ran. **COMMITTED** (`b575e599`, pushed to origin/main).
   - `mobile/src-tauri/gen/android/buildSrc/src/main/java/com/tanvoid0/portal/mobile/kotlin/BuildTask.kt`:
     the gradle rust-callback baked args `listOf("tauri", ...)` — bare `tauri`, so `node tauri`
     from `src-tauri` failed with MODULE_NOT_FOUND. `tauri android init` captured the runner wrong
     because the CLI was first invoked via the `.bin/tauri` shim. Patched the first arg to the
     absolute `D:/projects/production/portal/portal_desktop/node_modules/@tauri-apps/cli/tauri.js`.
     **NOT committed** — it lives in `gen/android/`, now gitignored (see below). A fresh
     `tauri android init` RE-BREAKS it. **This is the open durable-build follow-up.**
4. **Android toolchain set up**: `tauri android init` scaffolded `mobile/src-tauri/gen/android`.
   Build env that works: `JAVA_HOME=C:\Users\tanvo\.sdkman\candidates\java\21-tem` (JDK 25 default
   breaks AGP), `NDK_HOME=D:\sdk\android\ndk\28.2.13676358`, `ANDROID_HOME=D:\sdk\android`.
   Build command (from `mobile/`):
   `node "<abs>/node_modules/@tauri-apps/cli/tauri.js" android build --debug --apk --target <aarch64|x86_64>`
5. **Built debug APKs** for x86_64 and aarch64. Universal APK at
   `mobile/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`.
6. **Installed + launched on the real S26** (`adb-R5GL150LRCK-N75UEH...`, Android 17, arm64):
   MainActivity resumed, process healthy, no crash.
7. **Functionally tested via webview CDP** (debug build enables webview devtools): the Connect
   screen renders fully (`tauri.localhost/connect`, H1 "Connect to your PC", 3 inputs, Scan QR /
   Search / Pair buttons, no console errors). `mobile_pair` fires `POST /pair/request` and errors
   surface correctly. **The black `adb screencap` is only a webview hardware-surface capture
   limit — the UI really renders.**
8. **Investigated the 2026-11-11 LAN block — it is NOT active now.** Phone→desktop
   `192.168.0.137:18420`: `/health` = HTTP 200 (14ms); `POST /pair/request` TCP-connects in 8ms
   and long-polls correctly. Avast `SecureLine` service runs but isn't blocking the port; a
   `Portal Desktop` inbound-allow firewall rule is present. The earlier in-app "cannot reach"
   error was STALE. ICMP ping still 100% loss (just ICMP — irrelevant). `.gitignore` updated to
   ignore `mobile/src-tauri/gen/android/` + `mobile/src-tauri/target/` (COMMITTED in `b575e599`).

## What's left / in flight

1. **DONE: end-to-end pair works.** `POST /pair/request` (device "Claude pair test") → user
   approved on desktop → daemon returned HTTP 200 `{"name":"CRYOSTATION","port":18420,"token":"..."}`
   in 26s. Full handshake (reach → long-poll → approve → token) confirmed. A test device
   "Claude pair test" (and a stale "reachability-test") are now paired/pending on the desktop
   Devices page — **revoke/dismiss them** (test junk). The app wasn't connected with the token
   (test used raw adb curl, not the app's `mobile_pair`); to connect the real app, feed the token
   to `mobile_connect`.
2. **REAL BUG found: the "Review" button on the pairing notification does nothing.**
   `src/lib/components/layout/app-shell.svelte:186-193` fires `notify({ title:'A phone wants to
pair', route:'/devices', actionLabel:'Review', duration:0 })` on `device://pending`. Clicking
   Review does not navigate to /devices. Bug is in how `notify`'s action handles `route`/
   `actionLabel` — check `src/lib/utils/notify.ts` and the toast component's action wiring.
   Pairing still completes if you reach /devices another way. Desktop-side frontend fix → needs a
   release rebuild+reinstall to verify (the running desktop is the installed release, not dev).
3. **Discovery** ("No desktops found" in app) not independently confirmed — the busybox `nc` on
   the phone lacks `-b` (broadcast). Manual-IP pairing works regardless. Minor.
4. **Durable build fix** for BuildTask.kt (item 3b above) so `tauri android init` produces a
   working runner without hand-patching. Not started.
5. Phases 1-6 of PORTAL_MOBILE.md (phone polish, wider caps, push, relay, PC control, media) —
   not started; each ships on its own.

## Decisions made, don't re-litigate

- **Commit to `main` directly** (no branch/PR) — this repo commits straight to main (was already
  several commits ahead). No AI attribution line in commits (global CLAUDE.md forbids it; overrides
  the system reminder).
- **`gen/android/` gitignored, not committed** — it's a generated Gradle project and the BuildTask.kt
  fix is a machine-specific absolute path. Committing it as-is would be wrong. `gen/schemas/` stays
  tracked (the draft already tracked desktop/windows schemas; added android/mobile schemas).
- **Build x86_64 for emulator, aarch64 for the phone** — not all 4 ABIs, to save disk/time.

## Things that bit this session

- **The API-37 preview emulator (all local AVDs, `android-37.0 ps16k`) drops the launcher activity**
  on install — `am start` → "Activity class does not exist", result -92 — even though the APK is
  provably correct (aapt shows launchable-activity, MainActivity is in classes7.dex). Real Android
  17 parses it fine. Don't chase this as an app bug; use the real phone (or a stable API-34/35 image).
- `| tail`/`| head` on a build command **masks the real exit code** (reports the pipe's). Run build
  commands WITHOUT a pipe so the background task captures tauri's real exit code; read the output file after.
- A node script in the scratchpad can't `require('ws')` from the repo — set
  `NODE_PATH="<repo>/node_modules"`.
- Webview CDP over `adb forward` is flaky here: the devtools socket exists but `/json` intermittently
  returns empty and scripts hang. Add hard timeouts; have an adb-curl fallback for daemon tests.
- Non-fatal Kotlin-daemon "Could not close incremental caches" warning (against the tauri crate's
  android build dir in `~/.cargo/registry`) during build — didn't affect output.
- `tauri android init` regenerates `gen/schemas/*.json` (schema churn in git) — revert the
  desktop/windows ones, keep the new android/mobile ones.

## Key paths

- Plan: `docs/development/PORTAL_MOBILE.md`
- Mobile Rust shell: `mobile/src-tauri/src/lib.rs` (catch-all invoke handler, mobile_* commands, bridge)
- Mobile config (fixed): `mobile/src-tauri/tauri.conf.json`
- Frontend mobile domain: `src/lib/domains/mobile/` (connection.svelte.ts, navCaps.ts, pairUrl.ts, shell.ts)
- Connect route: `src/routes/connect/+page.svelte`
- Daemon /whoami + /events: `src-tauri/src/daemon/mod.rs`
- Generated android (gitignored): `mobile/src-tauri/gen/android/` — BuildTask.kt patch lives here
- Scratch CDP scripts: in this session's scratchpad (cdp-inspect.js, cdp-check.js, cdp-pair.js)
