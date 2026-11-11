---
id: QA-124
title: "/pair/request from a real phone (and from curl) never creates a pending row — phone stuck on \"Waiting for <PC>\" forever"
status: open
severity: blocker
kind: bug
area: devices/pairing
mode: tauri
found: 2026-11-11
---

## Use case

User asked to verify the Android app (Portal Remote) can actually connect to and control
Portal Desktop — the real pairing handshake, not just the UI around it.

## Repro

1. Real Samsung S26 Ultra, Portal Remote app (`com.portalremote`), discovers `CRYOSTATION` at
   `192.168.0.137:18420` (instance-2, the only running instance with "Allow pairing over the
   local network" on — confirmed instance-1/`:9222` has no LAN listener at all right now, by
   design, since that toggle is off there).
2. Tap the discovered PC. Phone shows "Waiting for CRYOSTATION... Portal Remote is asking on
   that PC. Choose Allow there to finish pairing." with a spinner.
3. Watched the Devices page on `:9223` (the matching instance) for 4+ minutes, refreshing —
   **no new pending request ever appears**, under any name. Only two long-stale fixture rows
   ("Browser", dated 06/07/2026 and 24/11/2025) are present throughout.
4. Checked the instance's own SQLite DB directly
   (`%APPDATA%/com.tanvoid0.portal-desktop/instance-2/portal_desktop.db`,
   `device_approvals` table) — no row is ever inserted for this attempt, at any point.
5. Cancelled and retried on the phone — same result, indefinitely.
6. Reproduced independently of the phone: `curl -X POST http://192.168.0.137:18420/pair/request
   -d '{"device":"CurlTest3"}'` (correct field name, confirmed via the 422 the server returns
   for a wrong body shape) — the request hangs (consistent with the intended 170s long-poll)
   but likewise **never inserts a `device_approvals` row**, whether sent to the LAN IP or to
   `127.0.0.1:18420` on the same machine. So this isn't Android- or network-specific.

## Expected

`pairing_service::request_pairing` inserts a `pending` row synchronously before the handler
starts its long-poll wait (see `pairing.rs`'s `PAIR_REQUEST_TIMEOUT` flow). The row — and
therefore the Devices page's "Pending requests" card — should appear within a second or two of
the phone/curl request landing, so the desktop operator has 170s to click Allow.

## Actual

No row, ever, from any of ~5 separate attempts over several minutes (1 real phone x2, curl x3).
The HTTP endpoint is definitely reachable and definitely routes correctly — a malformed body
gets an immediate, correct 422 ("missing field `device`") — so this isn't a network/firewall/
reachability problem (firewall rules for this exact instance's binary are present, enabled, and
scoped to the active "Public" network profile; a raw curl POST from the same machine via its own
LAN IP gets the 422 instantly). A well-formed body is accepted (no immediate error) but the
promised DB insert never becomes visible, and the caller just hangs until the real 170s timeout
would presumably fire (not confirmed out that far — every attempt was abandoned and retried
well before then, matching what a real impatient user would do, which is itself part of the
problem: there's no feedback at all in that window).

Not yet isolated to a specific line — most likely candidates, in order:
- `request_pairing`'s `.exec(conn)` genuinely stalling (SQLite lock contention from a
  concurrent writer/connection-pool issue), which would also explain why the HTTP call never
  returns at all rather than erroring.
- The long-poll handler calling something before the insert that itself blocks.
- `list_devices` or the frontend's polling/display somehow filtering out rows created in the
  last few minutes (checked directly against the DB file, not just the UI, so this is the less
  likely of the two, but not ruled out).

## Impact

This is the core feature the whole pairing UI (QA-113/114/115/117/118/121/123, all already
filed) sits on top of — right now, on this build, a real phone cannot pair with Portal Desktop
at all; the request never reaches the point where a human could even click Allow. QA-123's own
"ghost pair" fix (this session, `77a496b2`) and all the capability-grid/timing issues filed
around pairing are moot if the request itself never lands.

## Repro script

`curl -X POST http://<lan-ip-or-127.0.0.1>:18420/pair/request -d '{"device":"test-name"}'`
against instance-2 (the only instance with LAN pairing on) while polling
`device_approvals` in its DB (`%APPDATA%/com.tanvoid0.portal-desktop/instance-2/portal_desktop.db`)
for a matching `device_name`.

## Fix

Root cause: `daemon::configured_port()` (`src-tauri/src/daemon/mod.rs`) always
resolved to the literal constant `DEFAULT_PORT` (18420) regardless of
`PORTAL_INSTANCE`. Every running dev instance calls `daemon::spawn` with that
same port unconditionally (`serve_on(..., LOCALHOST, port, "loopback")` runs
even when the LAN toggle is off), so only the *first* instance to start wins
the OS-level bind on 18420 — every later instance's bind attempt fails
silently (`eprintln`, no crash). Reproduced directly: `curl -X POST
http://127.0.0.1:18420/pair/request -d '{"device":"RootCauseTest1"}'` against
the live multi-instance dev setup on this machine inserted the row into the
**default/instance-1** `portal_desktop.db`, not `instance-2`'s — confirming
the insert was never missing, it was landing in whichever instance happened
to own the port, independent of which instance's "Allow pairing" toggle was
on or which Devices page was being watched.

Fix: `configured_port()` now offsets `DEFAULT_PORT` by `PORTAL_INSTANCE`
(`DEFAULT_PORT + (n - 1)`), the same numbering `dev-instance.mjs` already
uses for the vite/CDP ports — numeric instances (2-9) each get their own
daemon port, so there's no port race. Default/instance-1 is unaffected
(`PORTAL_INSTANCE` unset → offset 0 → unchanged 18420), so a normal single
packaged install sees no behavior change.

Files: `src-tauri/src/daemon/mod.rs`. Checked: `cargo check -p portal_desktop`
(clean, pre-existing warnings only).

Follow-up, same class, fixed in the same session: the UDP discovery
responder (`daemon::discovery::spawn_responder`, port 8765) had the same
unconditional-bind pattern, but can't be offset per instance like the HTTP
port — Android broadcasts its probe to the literal `8765`, fixed on their
side. Fixed instead with `SO_REUSEADDR` (added `socket2` as a direct
dependency, `src-tauri/Cargo.toml`, to set it before bind — `std`/`tokio`
`UdpSocket::bind` offer no way to set socket options first) so every running
instance can bind `0.0.0.0:8765` concurrently instead of only the first to
start. Files: `src-tauri/src/daemon/discovery.rs`, `src-tauri/Cargo.toml`.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
2026-11-11, tauri dev, by the coder immediately after the fix (curl only, not
re-run from a real phone yet): killed all running dev instances, rebuilt, and
restarted instance-1 (`pnpm tauri:dev`) and instance-2
(`pnpm tauri:dev:instance 2`). Logs confirm instance-1 binds 18420 as before
and instance-2 now binds **18421** (both loopback and LAN) — no port
collision. `curl -X POST http://127.0.0.1:18421/pair/request -d
'{"device":"PostFixVerify"}'` against instance-2's own port landed the row in
instance-2's own `device_approvals` table (confirmed via sqlite3), not the
default db.

After the discovery fix: restarted both instances again, no
"could not bind UDP discovery on :8765" error from either (previously always
logged on the second instance to start). A unicast UDP probe to 127.0.0.1:8765
got exactly one reply back (instance-1, `{"port":18420,...}`); did not confirm
whether a real LAN broadcast fans out to both instances' sockets on Windows —
that's a platform question a loopback test on one machine can't settle, and
doesn't matter for a normal single-instance install (no bind contention
there at all). Still open: an actual real-phone repro end to end, and
clicking Allow through to a live token, have not been re-tested.
