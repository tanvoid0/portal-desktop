# Handoff — phone pairing connectivity: legacy server removed, self-fix diagnostics built, real blocker still unconfirmed

Written 2026-11-11, for someone with **no memory** of this session. **Do not overwrite or confuse
this with `.claude/handoff.md`** — that is a different, separately-active concurrent session's
handoff (agents domain, P1 review-fix + live-QA, nothing committed there either). This repo has
several scoped handoff files (`.claude/handoff-*.md`) specifically because multiple sessions work
this checkout concurrently at once — don't collapse them into one file.

## Goal

User (Tanveer) couldn't pair his Samsung S26 Ultra to Portal Desktop over the Portal Remote
Android app on wifi. Debug why, fix what's fixable in app code, and build some self-diagnosis so
the next person doesn't have to repeat this by hand.

## What's done

1. **Found and removed a legacy blocker**: the old C# `PortalRemote.exe` server (the one being
   replaced per `.claude/handoff-android-remote-control.md`'s Android-rewrite work) was still
   installed and running on this dev machine, and had grabbed UDP discovery port `:8765` _before_
   the new Rust daemon could bind it — so the phone's discovery probe was getting answered by the
   **old** server (`version: 0.8.0`, real GUID `id`) instead of the new one (which always replies
   `id: null`). Fully uninstalled: process killed, `C:\Users\tanvo\AppData\Local\Programs\Portal
Remote\` deleted, its registry uninstall key removed (`HKCU\...\Uninstall\{7A870589-8B8C-4577-
B080-E73414CA9C86}_is1`), Start Menu shortcuts removed, and its `HKCU\...\Run\PortalRemote`
   startup entry removed. **If a fresh session ever sees "discovery replies look wrong" again,
   check `tasklist | grep -i portalremote` first** — this could recur on another dev machine.
2. **Found the real, still-unconfirmed blocker**: even with the legacy server gone and the new
   daemon correctly bound to `0.0.0.0:18420`, the phone still can't connect — TCP connect from the
   phone to the desktop times out on **every** port tested (`18420` with an explicit firewall
   allow rule, _and_ `1421`, vite's dev port, with no rule at all). `ping` is asymmetric: desktop →
   phone works (23ms), phone → desktop is 100% loss. Ruled out: a competing Windows Firewall Block
   rule (none exists in the Public profile), interface-scope restriction on the existing allow
   rule (scoped `Any`), wrong network profile (confirmed `Public`, matches the rule). **Leading
   candidate, not yet confirmed**: `Avast VPN` (`SecureLine` service) is installed and its service
   is `Running`/`Automatic` startup — VPN clients commonly drop inbound LAN traffic as an anti-leak
   default even when not actively "connected" (no adapter or process was up at time of testing,
   but the background service was). No other 3rd-party AV/firewall is registered in Windows
   Security Center (`root\SecurityCenter2`) — only Windows Defender. Confirming this further needs
   `netsh wfp show state` (Windows Filtering Platform dump, the layer _below_ the Windows Firewall
   where a VPN driver would filter invisibly to `Get-NetFirewallRule`) — **this requires admin
   elevation**, which this session did not do without being asked. Simplest real test: stop the
   `SecureLine` service (or find its "allow local network" setting) and retest from the phone.
3. **Built self-fix diagnostics** (new code, all in this repo, all uncommitted):
   - `src-tauri/src/daemon/firewall.rs` (**new file**) — on `daemon::spawn()`, when LAN pairing is
     on: checks whether a Windows Firewall inbound-allow rule already exists for the _current_
     running exe path via `netsh advfirewall firewall show rule name="Portal Desktop" verbose`,
     and best-effort adds one if missing (`netsh ... add rule ...` — silently no-ops without admin
     rights, no UAC prompt, no retry). Also exposes `active_vpn_adapter()`, which parses
     `ipconfig /all` for a _connected_ adapter matching a keyword list (`vpn`, `tap-windows`,
     `wintun`, `ppp adapter`, `nordvpn`, `expressvpn`, `avast secureline`, `openvpn`, `wireguard`,
     `tailscale`, etc.) — report-only, there's no way for an app to "fix" someone else's VPN
     client. 8 unit tests on both parsers (pure functions, fed realistic fixture text).
   - `src-tauri/src/daemon/mod.rs` — added `pub mod firewall;` and one call,
     `firewall::spawn_check(port);`, inside the existing `if lan_enabled() { ... }` block.
   - `src-tauri/src/domains/network/commands.rs` — new `RemoteConnectivityStatus { lan_enabled,
firewall_rule_present, port, vpn_adapter }` struct (plain `Serialize`/`Deserialize`, **not**
     `TS` — this crate has zero `derive(TS)` anywhere; ts-rs only works in `portal-core`) and
     `get_remote_connectivity_status()` command (loopback/IPC-only, no cap).
   - `src-tauri/src/lib.rs` — registered the command in `generate_handler!`.
   - Frontend, following the devices domain's existing shape: `src/lib/domains/devices/types/
index.ts` (hand-typed `RemoteConnectivityStatus`), `src/lib/domains/devices/api/
deviceApi.ts` (`fetchConnectivityStatus`), `src/lib/domains/shared/query/keys.ts`
     (`devices.connectivity`), `src/lib/domains/devices/queries/deviceQueries.ts`
     (`createConnectivityStatusQuery`), `src/lib/domains/devices/components/
FirewallWarningBanner.svelte` (**new** — shadcn `Alert variant="destructive"`, takes
     `firewallRuleMissing`/`vpnAdapter` props, renders a bulleted warning naming whichever
     problem(s) apply), `src/routes/devices/+page.svelte` (wired the query + two derived flags +
     the banner, rendered between `LanToggleCard` and `PairDeviceCard`).
4. **Found and fixed two bugs in the first implementation pass** (a `crew:coder` subagent wrote
   the first draft; this session caught these live, not in review):
   - **Startup panic**: `tokio::task::spawn_blocking` (the free function) panics with "there is no
     reactor running" when called from `daemon::spawn()`, which runs synchronously during Tauri
     setup, outside any active Tokio task context. Fixed by switching to
     `tauri::async_runtime::spawn_blocking`, matching every other async spawn in this file.
   - **`\\?\` path prefix**: `current_exe_path()` called `.canonicalize()`, which on Windows
     prepends the `\\?\` extended-length-path prefix. `netsh`'s `program=` argument rejects that
     prefix outright ("The application contains invalid characters, or is an invalid length"),
     and it would _also_ have silently broken the read-side match against `netsh show rule`'s
     plain-path output (meaning it would have kept re-attempting the add, and kept failing, every
     single startup). Fixed by stripping the prefix after canonicalizing, in the one place both
     `ensure_rule` and `rule_present` get the path from.
5. **All gates green** (re-run after the manual fixes above): `cargo check` / `cargo clippy --lib
--tests` / `cargo test --lib` (8/8 new tests in `daemon::firewall::tests`, no regressions) from
   `src-tauri/`; `pnpm check` (0 errors), `eslint` on touched files (0 errors, only pre-existing
   `prefer-const` warnings on destructured `$props()` — same pattern as sibling files in this
   domain, not a real issue), `prettier --check`/`--write` and `cargo fmt` applied to touched files.
6. **Verified live** on dev instance-2 (`node scripts/dev-instance.mjs 2` — webview `:1421`, CDP
   `:9223`, daemon `:18420`, data dir `instance-2`): daemon logs `[daemon] firewall rule OK for
...`; `curl -s -X POST http://127.0.0.1:18420/rpc/get_remote_connectivity_status` returns
   `{"firewall_rule_present":true,"lan_enabled":true,"port":18420,"vpn_adapter":null}` (correct —
   matches that Avast's service exists but no adapter was actually up at test time); Devices page
   (`http://localhost:1421/devices`) renders with no console errors, banner correctly hidden since
   nothing was actually wrong at that exact moment.

## What's left

1. **Confirm or rule out Avast VPN as the actual blocker.** Stop the `SecureLine` Windows service
   (`Get-Service SecureLine` / `Stop-Service SecureLine` — needs admin) or find its LAN-access
   setting, then retest phone → desktop TCP from the phone (see "useful repro commands" below).
   This is the single most valuable next step — nothing else in this list matters if this isn't
   the real cause.
2. **If Avast isn't it**: next diagnostic layer is `netsh wfp show state file=<path>` (needs admin
   elevation this session didn't take) to dump the Windows Filtering Platform and look for a
   non-Windows-Firewall callout provider/filter blocking inbound TCP. This is genuinely below what
   app code can fix or detect — host-level forensics only.
3. **Decide commit granularity and commit.** Nothing from this session is committed. Suggest this
   session's slice (see "files that matter") as its own commit, separate from whatever else is
   uncommitted in this checkout from concurrent sessions (see `.claude/handoff.md` and
   `.claude/handoff-devices-phone-qa.md` — both describe their own in-flight uncommitted work in
   this exact same checkout; **do not `git add -A`**, stage this session's files by name).
4. **Not touched, not this session's concern**: `.claude/handoff-devices-phone-qa.md`'s own
   leftover QA items (QA-113/114/115/117/118 triage) — untouched, belongs to that thread.
   `src-tauri/portal-core/src/domains/agents/*`, `src/lib/domains/agents/*`,
   `src/routes/agents/run/*` — owned by `.claude/handoff.md`'s session, not touched here.
5. Portal's own task-board MCP server (`mcp__plugin_crew_portal__*`) was not running when this
   session tried to log a task on it (`add` failed: "Portal is not running"). Noted once per this
   user's global CLAUDE.md convention, then continued without it for the whole session — no board
   task exists for this work.

## Files that matter (this session's slice)

- `src-tauri/src/daemon/firewall.rs` — **new**, firewall self-repair + VPN detection, 8 unit tests.
- `src-tauri/src/daemon/mod.rs` — `pub mod firewall;` + one call in `spawn()`.
- `src-tauri/src/domains/network/commands.rs` — `RemoteConnectivityStatus` + the new command.
- `src-tauri/src/lib.rs` — command registered in `generate_handler!`.
- `src/lib/domains/devices/types/index.ts`, `api/deviceApi.ts`, `queries/deviceQueries.ts`,
  `src/lib/domains/shared/query/keys.ts` — frontend plumbing for the new command.
- `src/lib/domains/devices/components/FirewallWarningBanner.svelte` — **new**, the warning banner.
- `src/routes/devices/+page.svelte` — wired the query + banner.
- **Not a code change, won't show in git diff**: `PortalRemote.exe` was fully uninstalled from
  this dev machine (see item 1 above) — worth remembering if "phone pairing used to work via some
  other app" confusion ever comes up.

## Decisions made, don't re-litigate

- **No same-host reachability self-ping.** A TCP self-connect to this machine's own LAN IP almost
  always succeeds via Windows' loopback short-circuit regardless of whether an external device
  could actually get in — that would have been a misleading "all clear" in exactly the scenario
  that started this whole session. Deliberately not built; see the module doc at the top of
  `firewall.rs`. The diagnostics report **verifiable facts** (rule exists? VPN adapter up?)
  instead.
- **No UAC/elevation flow** for the firewall-rule add. Best-effort only; if the process isn't
  elevated, `netsh add rule` silently fails and it's just logged. No retry loop, no prompt.
- **VPN detection is report-only.** The app names the adapter in the banner; it does not and
  should not try to disconnect or reconfigure someone else's VPN client.

## Things that bit this session

- `tokio::task::spawn_blocking` (free function) panics with "there is no reactor running" when
  called from a thread with no ambient Tokio context — exactly `daemon::spawn()`'s call site
  (synchronous, during Tauri setup). Use `tauri::async_runtime::spawn_blocking` in this file, not
  the raw `tokio::` one, even though both exist elsewhere in this crate.
- `std::path::Path::canonicalize()` on Windows prepends `\\?\` — strip it before handing a path to
  `netsh` or any other external tool with its own naive argument parser.
- **Instance-2's dev server tab exits entirely on a Rust panic**, not just the child process —
  `cargo tauri dev`'s watch mode does not auto-restart after a crash. After any panic, re-run
  `node scripts/dev-instance.mjs 2` in a fresh terminal tab rather than waiting for the watcher.
- This checkout is heavily multi-session right now (confirmed via `ListAgents`, and by the sheer
  number of `.claude/handoff-*.md` files). Always `git status` before assuming an unfamiliar
  modified/untracked file is "stray" — it's much more likely another session's in-progress work.

## Useful repro commands (phone connected over wireless adb at the time of writing)

```
adb connect 192.168.0.235:45867
adb -s 192.168.0.235:45867 shell "curl -s -m 5 -o /dev/null -w 'http_code=%{http_code} time=%{time_total}\n' http://192.168.0.137:18420/health"
```

A `http_code=000` with `time=5.00...` means still blocked. Desktop's LAN IP may differ by the time
this is read — reconfirm with `ipconfig` (look for the `Wi-Fi` adapter's IPv4 address) and the
phone's adb mDNS address may also have changed (`adb mdns services`).
