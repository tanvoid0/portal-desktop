# Portal Mobile — the phone (and any other device) as a client of Portal Desktop

**Status:** Phase 0 shipped and verified on `main` (`7aeed3da`, `b575e599`,
2026-11-12/13) — real-phone pairing confirmed end to end. The browser/
other-desktop client ("Other clients", below) is merged to `main` too —
Rust/daemon side compiles and the existing daemon tests pass; the frontend
shim has not yet been exercised in a real browser (board bug #552, the
broken node/pnpm toolchain, is now fixed, but the live browser check itself
still hasn't been run). See "State of the tree" for what's built and the
open follow-ups.
Supersedes "Phase 5 — Android client" in
[REMOTE_ARCHITECTURE.md](REMOTE_ARCHITECTURE.md), which planned to extend Portal
Remote's Kotlin app.

## Product

Portal is a tool for software developers: it automates and simplifies day-to-day
development, on the PC and from any secondary device — phone, laptop, another
desktop. The secondary device is the same product at a distance — check a
pipeline, answer an agent's permission prompt, steer a coder thread, triage the
board — not a media remote. Same Wi-Fi today; an intermediate relay for
any-internet access later.

That ordering decides priorities: developer workflows first, PC control
(trackpad, mirror) after.

## Decision

One Android app, built from **this repo**, as a Tauri 2 mobile target that ships
the **same SvelteKit frontend** and talks to the desktop daemon.

| Option                                | UI reuse                 | Cost of each new Portal feature on phone | Verdict   |
| ------------------------------------- | ------------------------ | ---------------------------------------- | --------- |
| Extend Portal Remote (Kotlin/Compose) | none — different toolkit | rebuilt by hand in Compose, forever      | rejected  |
| React Native                          | none — Svelte ≠ React    | rebuilt by hand, plus a third UI stack   | rejected  |
| **Tauri Android, thin client**        | all of `src/`            | add a `cap` to the command; UI is free   | **taken** |

Why the Kotlin app loses despite being 23k lines of working code: it is a PC
remote with its own design language (HUD, cut corners, monospace). Controlling
Portal itself means tasks, projects, coder, agents, terminal, pipelines — about
30 frontend domains that already exist here in Svelte. Rebuilding them in
Compose doubles every future feature. With the thin client the phone UI _is_ the
desktop UI, so it cannot drift.

## Other clients: laptop, a second desktop

The transport (`/rpc` + `/events` over a Bearer token) and the UI (this repo's
`src/`) are already device-agnostic — nothing below is phone-specific. A
laptop or another desktop reaching your main PC needs no Android/Tauri shell
at all: it already has a browser. Point it at the daemon with a thin web
build of `src/` (same `VITE_PORTAL_MOBILE` flag family, same `/connect`
pairing flow) and it's the identical client, just served as a page instead of
an APK. One client implementation (phase 0-3 below), two delivery shapes:
installed app (phone) and browser page (laptop/desktop). Falls out of the
existing plan; not a separate phase to design.

## How it works

```
phone                                         desktop
┌─────────────────────────────┐               ┌──────────────────────────┐
│ webview: same src/ build    │               │ Portal Desktop           │
│   invoke('get_tasks', args) │               │  daemon :18420 (axum)    │
│        │                    │               │                          │
│ mobile/src-tauri (~300 LOC) │  HTTP + WS    │                          │
│   catch-all invoke handler ─┼──────────────▶│ POST /rpc/get_tasks      │
│   events bridge ◀───────────┼───────────────│ WS   /events             │
│   mobile_pair / _connect    │──────────────▶│ POST /pair/request       │
└─────────────────────────────┘  Bearer token └──────────────────────────┘
```

- **No transport layer in the frontend.** `invoke` and `listen` are untouched;
  the mobile Rust shell forwards any command name to `/rpc/{command}` and
  re-emits `/events` frames as Tauri events. The frontend only knows
  `VITE_PORTAL_MOBILE=1` (connect screen, nav filtered by capability, desktop-only
  startup calls skipped).
- **Requests leave from Rust, not the webview**, so they carry no `Origin`
  header. The daemon's "refuse every Origin" rule stays exactly as it is.
- **Security model is the daemon's, unchanged:** per-device token, capabilities
  chosen at approval, credentials/vault never remote, sensitive caps opt-in.
- `mobile/src-tauri` is a separate crate on purpose: it must not link
  `portal-core` (pty, enigo, sqlite have no business on a phone), and the
  desktop build cannot be broken by it.

## Phases

Each ships on its own.

0. **Shell + connect** — `mobile/src-tauri`, `GET /whoami`, `/connect` screen
   (discover / QR / manual), nav filtered by caps, startup hardening.
1. **Phone polish of the capped domains** — dashboard, tasks, projects, coder,
   agents, terminal at phone width. Fix per page; no redesign.
2. **Widen the remote surface to the dev workflows** — give caps to
   pipelines/actions (run, watch, Diagnose & fix), n8n, github, and
   notifications (`event_cap` currently withholds them). One attribute per
   command, plus a row in the approval UI.
3. **Push when it matters** — agent needs a permission decision, pipeline
   failed, run finished: Android notification with the action on it. This is
   the feature that makes the phone worth carrying for development.
4. **Relay (any internet)** — an intermediate server both sides dial out to.
   Prerequisites, in order: end-to-end encryption keyed at pairing (the relay
   must only ever see ciphertext — it would otherwise sit in front of terminal
   exec), then the relay itself. The phone side is a one-file change: the
   connection becomes a URL instead of `host:port`; `/rpc` and `/events` keep
   their shape. Not built until LAN use is solid.
5. **PC control in Svelte** — trackpad + keyboard page over `/control`
   (`remote.input` is already served by the daemon), resource monitor (one
   `sysinfo` command + page, which the desktop gets for free too).
6. **Media-remote features** — screen mirror, files, now-playing/volume. Port
   from Portal Remote's C# server into the daemon only as each is wanted.

## Phase 0 spec (for whoever executes)

**Preconditions**

- **Disk:** `D:` had 0 GB free on 2026-11-12 (3.7 GB after deleting the mobile
  `target/`). An Android build needs roughly 10–15 GB (cargo target per ABI +
  Gradle + NDK sysroot). Free space or point `CARGO_TARGET_DIR` at `C:` first.
- `ANDROID_HOME=D:\sdk\android`, `NDK_HOME=D:\sdk\android\ndk\28.2.13676358`,
  `JAVA_HOME` = `~/.sdkman/candidates/java/21-tem` (default JDK 25 will likely
  break AGP).
- `rustup target add aarch64-linux-android x86_64-linux-android` (not installed).
- AVDs exist (`Pixel_9a`, `portal_claude`, …). No physical device attached.

**Rust shell — `mobile/src-tauri/`** (crate `portal-mobile`, id
`com.tanvoid0.portal.mobile`; must not depend on `src-tauri` or `portal-core`)

- `frontendDist` = root SvelteKit build, built with `VITE_PORTAL_MOBILE=1`.
- Stored connection `{host, port, token, name}`: JSON in the app data dir.
- Local commands — the contract the frontend codes against:

| Command                               | Behaviour                                                                                                       |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `mobile_status()`                     | `{connected, host, port, name, caps: string[]}`                                                                 |
| `mobile_pair({host, port, device})`   | `POST /pair/request`, long-poll ≤175 s. 403 denied / 408 timeout / 429 rate-limited → distinct readable errors. |
| `mobile_connect({host, port, token})` | Verify with `GET /whoami`, store, start events bridge. QR payload: `portalremote://{ip}:{port}/{token}`.        |
| `mobile_disconnect()`                 | Stop bridge, delete stored connection.                                                                          |
| `mobile_discover()`                   | UDP broadcast `PORTALREMOTE?` to 8765, collect replies ~1.5 s → `[{host, port, name, version}]`.                |

- Event `mobile://connection` (payload = status) on every change, including a
  401 from the daemon (token revoked → disconnected).
- **Catch-all:** custom `Builder::invoke_handler` closure. `mobile_*` handled
  locally; every other command → `POST /rpc/{command}`, Bearer token, JSON args
  as sent (`/rpc` accepts camelCase). Non-2xx → reject with the body text.
  `plugin:*` commands are routed by Tauri and never reach the closure. Check the
  `Invoke` / `InvokeResolver` / `InvokeBody` API against the tauri source —
  don't guess it.
- **Events bridge:** tokio-tungstenite to `/events` with the Authorization
  header, `app.emit(event, payload)` per frame, capped-backoff reconnect.
- `tauri-plugin-barcode-scanner` for QR. Android manifest: INTERNET +
  cleartext traffic allowed (daemon has no TLS).

**Daemon — `src-tauri/src/daemon/mod.rs`, additive**

- `GET /whoami` (same guard as `/rpc`) →
  `{name, version, local, device_id, caps}`; local caller → all caps. One test.

**Frontend — `src/**`, everything behind `VITE_PORTAL_MOBILE === '1'`\*\*

- `src/lib/domains/mobile/`: `isMobileShell`, runes connection state over
  `mobile_status` + `mobile://connection`, pure `parsePairUrl` + one vitest.
- `/connect` route: discovered list → pair (waiting-for-approval state), Scan
  QR, manual host/port (default 18420), editable device name.
- Root layout: not connected → `/connect`; dropped/revoked → `/connect` + toast.
- Nav (`src/lib/config/main-nav.ts`) filtered to what the device's caps can
  serve; entries with no capped backend hidden; Disconnect entry + PC name.
  Reuse the existing sidebar's mobile drawer.
- Guard desktop-only startup calls (updater, window APIs, notification/dialog/
  opener plugins, uncapped settings commands) so they don't throw or toast.
- `viewport-fit=cover`, safe-area padding, inputs ≥16px.

**Gates**

- `pnpm check && pnpm lint` (redirect output; piping masks the exit code).
- `cargo check` + `cargo test` in `src-tauri`; `cargo check` in
  `mobile/src-tauri` for host and `aarch64-linux-android`.
- `tauri android build --debug` → APK; install on an AVD. Note `10.0.2.2` is
  loopback to the daemon, so UI can be exercised there but pairing/token/caps
  need a real phone on the LAN (blocked by the network issue under Known
  ceilings until that is resolved).
- Reviewer pass on the catch-all handler and `/whoami` (trust boundary).

**State of the tree (2026-11-13)** — Phase 0 is done, on `main`, debug APK
built and installed on a real phone, pairing handshake confirmed over the LAN.
Full account: [.claude/handoff-portal-mobile-apk.md](../../.claude/handoff-portal-mobile-apk.md).

- `mobile/` — Rust shell (`src/lib.rs`: catch-all invoke handler, `mobile_*`
  commands, events bridge), `tauri.conf.json`, capabilities. `gen/android/` is
  gitignored (generated Gradle project; a machine-specific build-runner path
  fix lives only there, see follow-ups below).
- `src/lib/domains/mobile/*` (`connection.svelte.ts`, `navCaps.ts`,
  `pairUrl.ts`, `shell.ts`), `src/routes/connect/+page.svelte`.
- `src-tauri/src/daemon/mod.rs` — `/whoami`, `/events` frames, `/rpc/{cmd}`,
  `/pair/request`, UDP discovery `:8765`.
- Frontend glue: `+layout.svelte` (connect gate), `app-shell.svelte` (nav
  cap-filter + disconnect), `notify.ts` (guarded), `app.html`/`app.css`
  (viewport-fit, 16px mobile inputs).

**Open follow-ups** (none block using the app; tracked loosely under board
task #542, pick up as wanted):

1. **Real bug:** the pairing notification's "Review" button doesn't navigate
   to `/devices` — `notify()`'s action wiring
   (`src/lib/components/layout/app-shell.svelte` / `src/lib/utils/notify.ts`).
   Pairing still completes if you reach `/devices` another way.
2. **Durable build fix:** `tauri android init` regenerates
   `gen/android/buildSrc/.../BuildTask.kt` with a bare `tauri` arg that fails
   (`MODULE_NOT_FOUND`); it needs hand-patching to the absolute CLI path every
   time. Not started.
3. UDP discovery (`mobile_discover`) not independently confirmed — the test
   phone's busybox `nc` lacks `-b`. Manual host/port pairing works regardless.
4. Phases 1-6 below — not started; each ships on its own.

## Other clients: browser and other-desktop (2026-11-14)

Phase 0's "one Android app" decision extends to any device with a browser on
the LAN — a laptop, another PC, a tablet — with no install at all, by serving
the same thin-client frontend build straight off the daemon instead of a
second native shell.

**Trust change (prerequisite, already landed):** `classify()` in
`src-tauri/src/daemon/mod.rs` used to give any `Origin`-bearing request a flat
403, which also meant the free loopback pass never had to consider browsers.
Narrowed to: loopback only gets the free `Local` pass when there is **no**
`Origin` header (every native command is loopback-no-Origin); an
`Origin`-bearing request — browser-originated, loopback or remote — instead
falls through to the same bearer-token check a remote peer needs. A visited
page can't forge a token it was never handed, so this is what actually keeps
it off the port, not the `Origin` value itself. `/events`'s browser
`WebSocket` handshake can't set a custom header, so it additionally accepts
the token as `?token=` on that one path.

**The browser shim** (`src/lib/domains/mobile/browserShim.ts`) is the
web-client counterpart to `mobile/src-tauri/src/lib.rs`: it installs
`window.__TAURI_INTERNALS__` when `isMobileShell` is true and nothing is
there already (so it never touches the real Android Tauri runtime), and
mirrors the Rust shell's contract exactly — `mobile_status/_pair/_connect/
_disconnect/_discover` run locally against `localStorage` (no filesystem in a
browser); everything else forwards to `POST /rpc/{command}` with a bearer
token after the same ascii-alnum/underscore command-name check
(`mobile/src-tauri`'s `forward()`, mirrored in `commandName.ts`'s
`isForwardableCommand`) so a command like `plugin:barcode-scanner|scan` (no
browser equivalent — no QR scanning here, manual/discovered pairing only)
rejects cleanly instead of forwarding garbage; `/events` is bridged with a
plain `WebSocket` and the same capped-backoff reconnect as the Rust client. No
transport code in the rest of the frontend changes — `invoke`/`listen` stay
exactly what the desktop app already calls.

**Serving it:** `pnpm build:client` builds the SvelteKit frontend with
`VITE_PORTAL_MOBILE=1` to its own `build-client/` output dir (NOT `build/` —
the desktop app's `frontendDist` already points there, and Android's mobile
build also lands in the repo-root `build/`; a third build sharing that folder
would silently stomp whichever one built last). The daemon serves it via a
`fallback_service` (`tower-http`'s `ServeDir`) added to the router _after_
`.with_state()`, which — because axum's `Router::layer` only wraps routes
registered before it's called — puts it outside the `guard` middleware
entirely: anyone who can reach the port gets the app shell unauthenticated,
same as any other static site, since it holds no data or secrets; pairing and
every `/rpc` call the shell itself then makes still go through `guard` like
any other page-originated request. Open design call: where the daemon finds
`build-client/` at runtime. Dev resolves it at compile time relative to
`src-tauri`'s own manifest dir (mirrors `frontendDist` being a relative path
in dev). Release resolves it via Tauri's own resource dir, shipped there by
`tauri.conf.json`'s `bundle.resources` + a `beforeBundleCommand: "pnpm
build:client"` hook — so packaging fails loudly if the client build is
missing rather than shipping an app with a silently absent web client.
`build-client/.gitkeep` is tracked (empty) so `cargo check`/`build` succeed
before anyone has run the real build — `tauri-build` validates `bundle.
resources` paths exist on every compile, not only on package.

**Reachability:** unchanged from phase 0 — the LAN listener only binds when
the remote toggle is on, so this is opt-in, not a port that's always open to
the network.

## What happens to Portal Remote

Frozen, not deleted. It stays the tool for what it alone does well today
(mirror, phone-as-speaker, casting, in-app browser) until phase 4 covers the
parts worth keeping. No new features go there.

## Known ceilings

- No TLS: token and traffic are cleartext on the LAN. Same as Portal Remote;
  fix belongs in the daemon (self-signed cert pinned at pairing) before any
  internet relay.
- Token is stored in an app-private file, not the Android Keystore.
- `remote_lan_enabled` needs an app restart to take effect.
- Revoking a device does not close its already-open `/control` socket.
- Open network issue from 2026-11-11: the S26 Ultra could not TCP-connect to the
  desktop (Avast SecureLine suspected) — see
  `.claude/handoff-phone-pairing-vpn-firewall.md`. Emulator testing goes through
  `10.0.2.2`, which the daemon sees as loopback (so it bypasses token auth — fine
  for UI work, useless for testing pairing).
