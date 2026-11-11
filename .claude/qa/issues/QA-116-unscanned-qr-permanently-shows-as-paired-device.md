---
id: QA-116
title: Generating a pairing QR immediately (and permanently) lists it as a "Paired" device, even if never scanned and long past its 5-minute pickup window
status: verified
severity: major
kind: bug
area: devices
mode: tauri
found: 2026-11-11
---

## Use case

Pair a phone to Portal Desktop via Devices → "Pair a phone" → "Generate QR
code", or decide which devices currently have access by reading the
Devices page's "Paired" count / "Paired devices" list.

## Steps

1. Open Devices (`/devices`). Note the "Paired" stat.
2. Under "Pair a phone", (optionally pick capabilities,) click "Generate QR
   code". Do **not** scan it with a phone.
3. Wait — the QR's own copy says "an unscanned code expires in 5 minutes."
   Wait past that, then reload/refresh.

## Expected

An unscanned, expired QR invite should not count as, or appear as, a paired
device — at minimum it should be visually distinct from an actually-
connected phone, and ideally should disappear or show a clearly "expired"
state once its 5-minute pickup window has passed.

## Actual

The instant "Generate QR code" is clicked, the "Paired" stat increments
(2 → 3 in this run) and a new card appears in "Paired devices": **"Unnamed
device", "Until revoked", "no capabilities", "Never used"**, with a
"Revoke" button — indistinguishable in the UI from a real connected phone
except by reading the fine print. This stays forever: `list_devices`
(`src-tauri/portal-core/src/domains/network/services/pairing_service.rs:340`)
returns every `device_approval` row with no `expires_at` filter at all, and
the frontend's `paired` filter
(`src/routes/devices/+page.svelte:32-36`, `d.approved && approval_type !==
'revoked' && approval_type !== 'rejected'`) doesn't check `expires_at`
either — so this ghost entry is still sitting in "Paired devices" well
past its 5-minute window, with no automatic cleanup or expired-state
indicator anywhere in this codebase.

This directly contradicts the feature's own stated intent — the pickup-TTL
doc comment right above `mint_qr_pairing`
(`pairing_service.rs:19-21`) says *"a leaked or unscanned QR image grants
nothing"* after the window lapses, and `verify_token` does correctly treat
an expired pickup token as dead for auth purposes — but nothing ever
reflects that in the device list a human reads. A user auditing "what has
access to my desktop" via this page cannot tell a live paired phone from
every QR code they ever generated and forgot about.

## Evidence

`.claude/qa/shots/sweep-devices-qr-generated.png` — QR just generated,
"Paired 3" (was "Paired 2").
`.claude/qa/shots/sweep-devices-after-unscanned-qr-paired-list.png` —
"Unnamed device / Until revoked / no capabilities / Never used / Expires
11/11/2026" sitting in "Paired devices", never scanned.

## Repro

`node .claude/qa/scripts/sweep-devices-qr.mjs` then
`node .claude/qa/scripts/sweep-devices-qr-check.mjs` — generates a QR with
no capabilities selected and shows the resulting "Unnamed device" entry in
the Paired devices list.

## Fix

Root cause: the Devices page's "Paired" bucket
(`src/routes/devices/+page.svelte`) only checked `approved` and
`approval_type`, never `expires_at` — so a QR-minted `until_revoked` grant
counted as paired from the instant it was minted, whether or not it had
ever been scanned, and stayed that way forever once its 5-minute pickup
window lapsed (nothing clears `expires_at` except a real first use in
`verify_token`). This directly contradicted the QR card's own copy ("the
grant appears below once scanned").

The state machine already distinguishes "picked up" from "not yet":
`mint_qr_pairing` is the only path that sets `approval_type: "until_revoked"`
with a non-null `expires_at`; `verify_token` clears `expires_at` to `None`
on first real use, and the normal `approve()` path never sets `expires_at`
for `until_revoked` at all. So `approval_type === 'until_revoked' &&
expires_at !== null` uniquely identifies an unscanned QR grant, scanned or
expired alike — no backend/schema change needed, `list_devices` already
exposes `expires_at`.

Fix: excluded that combination from the `paired` filter in
`src/routes/devices/+page.svelte`, so an unscanned QR (within its window or
past it) no longer appears in "Paired devices" or counts toward the
"Paired" stat. It still isn't in "Pending requests" or "history" either —
it's simply invisible until scanned, matching the feature's stated design.
Once a phone actually scans it, `verify_token` clears `expires_at` and the
row appears in Paired on the next refresh, same as today.

**Follow-up (reviewer gap):** with the filter fix alone, a *real* scan had no
way to make the device reappear in "Paired devices" until the page was
manually refreshed — `createDevicesQuery()` isn't invalidated by anything on
scan. `PairDeviceCard`'s `onPaired` only fires at mint time (now correctly a
no-op for the "Paired" view, since the row is hidden then), and the only
push event, `device://pending`
(`src-tauri/src/daemon/pairing.rs:98`), is emitted for passcode requests
only — `verify_token`'s QR first-use branch
(`src-tauri/portal-core/src/domains/network/services/pairing_service.rs:412`,
reached via the axum guard in `src-tauri/src/daemon/mod.rs:342`) has no
Tauri `AppHandle` in scope to emit from (`portal-core` is intentionally
Tauri-free) and fires on every authenticated request, not just the one
scan, so it's not a one-line emit-on-transition either. Smaller correct fix
per review guidance: poll instead of pushing, scoped to exactly when a QR
is on screen. Added a `$effect` in `PairDeviceCard.svelte` that, while
`qrDataUrl` is set, calls the existing `onPaired` (→
`queryClient.invalidateQueries`) every 3s for up to the QR's 5-minute
pickup window, with the interval/timeout cleared on unmount or once a new
QR is generated. Verified live: the `setInterval(fn, 3000)` registers the
moment the QR appears and is cleared the moment the user navigates away
(checked by wrapping `window.setInterval`/`clearInterval` in the running
app), no console errors either side.

Files changed: `src/routes/devices/+page.svelte` (one filter),
`src/lib/domains/devices/components/PairDeviceCard.svelte` (poll while QR
shown).

Checks: `pnpm check` (0 errors, same 142 pre-existing unrelated warnings),
`pnpm lint` → eslint (0 errors on `src`) and `prettier --check` on both
changed files clean; the only `pnpm lint` failure is a pre-existing,
unrelated `.claude/handoff.md` prettier formatting issue not touched by
this change. `cargo check` not run — no Rust touched.

Reran the repro live against the running dev instance (CDP :9222):
generating an unscanned QR left the "Paired" stat unchanged (2 → 2) and no
"Unnamed device" card appeared in "Paired devices"
(`.claude/qa/shots/QA-110-after-fix-paired-list.png`).

Does not cover QA-111/QA-112/QA-113 (separate reviewer findings, tracked
and left open) or any other open QA id.

Needs a reviewer pass before merge (frontend state/data-path fix).

## Verification

2026-11-11, mode: tauri (real backend, CDP :9222, rebuilt from the working
tree including this fix), using the real connected Android phone
(`adb-R5GL150LRCK-N75UEH._adb-tls-connect._tcp`, which has `com.portalremote`
installed) for the actual scan.

(a) Generating a QR no longer bumps "Paired" or shows a ghost card: with no
capabilities selected, "Paired" stayed at 2 before and after clicking
"Generate QR code", and the paired-devices list showed no new card
(`node .claude/qa/scripts/qa-110-verify-a.mjs`,
`.claude/qa/shots/QA-110-after-qr-generated-no-ghost.png`,
`QA-110-after-qr-generated-paired-list.png`).

(b) Real scan → appears within ~3s, no manual refresh: the installed
`com.portalremote` app has no `portalremote://` URI intent-filter (deep
links aren't how it scans — confirmed via `adb shell dumpsys package
com.portalremote`, camera-only), so a literal adb `VIEW` intent can't
trigger it. Instead simulated the one network-observable effect an actual
scan has — the companion app's first authenticated request to the daemon
with the QR's bearer token — by running `curl` **on the phone itself**
(`adb shell curl ... -H "Authorization: Bearer <token>" http://<lan-ip>:18420/health`,
same codepath `verify_token`'s first-use clearing depends on, real LAN hop,
non-loopback peer). Result in `qa-110-verify-b3.mjs`: "Paired" went 3→4
exactly 2.0s after the curl, with no `page.goto`/reload in between —
confirms the new 3s poll picks up a scan on its own
(`.claude/qa/shots/QA-110-verify-b3-result.png`, shows "Paired 4" and the
`coder` switch flipped on from the real claim).

(c) Existing pairing unaffected: "Tanveer's S26 Ultra" stayed "Active now"
throughout, untouched by any of the above.

Cleaned up: revoked both test fixtures ("Unnamed device" / `tasks` and
`coder`, from the two scan-simulation runs) via the UI; "Paired" is back to
its baseline of 2, no `Unnamed device` entries remain
(`node .claude/qa/scripts/qa-110-cleanup4.mjs`, confirmed
`paired count now: 2`).

One environment note: the dev instance restarted on its own mid-verification
(a concurrent `cargo`/`cargo-clippy` process — presumably another agent's
build/checks running on the same tree — appears to have contended for the
build lock and triggered a rebuild-and-relaunch; CDP was briefly
unreachable, `app.mjs` fell back to "mode: browser" for one call, then
recovered on its own within the next call). Not a product bug, noting it in
case it explains any gap between tool calls.

Verified.

**Re-check 2026-11-11 (same day, separate pass, instance-2/CDP :9223 — the
sanctioned instance for this pass; the original verification above ran on
:9222):** re-ran both halves independently against this instance's own
backend/DB, since cross-instance state doesn't carry over:

(a) `node .claude/qa/scripts/QA-116-reverify.mjs` — baseline "Paired 1",
generated an unscanned QR with no capabilities, "Paired" stayed at 1, no
"Unnamed device" ghost in the page text
(`.claude/qa/shots/QA-116-reverify-qr-generated.png`). Confirmed via raw
`list_devices` (`.claude/qa/scripts/QA-116-raw-devices.mjs`) that the row
exists with `approved: true, expires_at: <+5min>, last_used_at: null` but is
correctly excluded from the paired view.

(b) Real-scan simulation hit a snag worth recording: my first two attempts
at "curl on the phone with the QR's bearer token" (`QA-116-reverify-scan.mjs`
+ manual `adb shell curl -H "Authorization: Bearer <token>" ...`, and
`QA-116-reverify-scan2.mjs` using Node's `execFile('adb', [...])` with the
header as a separate argv entry) got HTTP 200 from `/health` but **never
actually updated the device row** (`last_used_at` stayed `null` for all 3
tokens tried) — turned out `adb shell` flattens a multi-word execFile
argv into one remote command string, which silently dropped the `-H` value
(confirmed with `adb shell curl -v`: the outgoint request had no
`Authorization` header at all, and curl tried to resolve "Bearer" and the
token itself as hostnames). This was my test harness's bug, not the
product's — fixed by quoting the whole remote command as one pre-escaped
string (`adb shell "curl ... -H 'Authorization: Bearer <token>' ..."`,
also used as a single string in the execFile call in
`QA-116-reverify-scan3.mjs`). Confirmed the header now actually arrives by
diffing `/control`'s response with a bogus vs. the real token: bogus token →
`401 pair this device before calling the API`; real token → `400 Connection
header did not include 'upgrade'` (i.e. auth resolved, the route handler ran
and complained about something else entirely) — a clean way to tell
"token accepted" from "token rejected" on a route that's public either way.

With the header actually sent, `QA-116-reverify-scan3.mjs` reproduced the
original finding cleanly: "Paired" went 2 → 3 at **+2.1s** after the phone's
curl call, watching the *same already-open* page (no `goto`/reload in
between) — confirms the 3s poll still picks up a real first-use on this
instance too. Raw `list_devices` confirmed the row's `expires_at` cleared to
`null` and `last_used_at` set to the scan time.

Cleaned up: all 5 QR-pairing fixtures created across (a) and (b) on
instance-2 were revoked (2 reachable from the UI's "Revoke" button once
scanned; the other 3 unscanned/expired ones are invisible in the UI by
design — the known QA-117 gap — so revoked via the same single-device-scoped
`revoke_device` command the UI button itself calls, not a bulk/reset
operation: `.claude/qa/scripts/QA-116-cleanup2.mjs`). Confirmed back to the
pre-test baseline, "Paired 1 / Pending 2", no "Unnamed device" rows left
(`.claude/qa/shots/QA-116-final-cleanup-check.png`).

Still verified — holds on instance-2 as well as instance-1.
