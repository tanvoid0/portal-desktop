---
id: QA-121
title: Pairing requests die silently in 170s with no alert outside the Devices page — real-world pairing fails almost every time
status: verified
severity: blocker
kind: bug
area: devices/pairing
found: 2026-11-11
mode: tauri
---

## Use case

A phone pairs with Portal Desktop by discovery or manual IP entry (not QR): it
POSTs `/pair/request` and that call blocks, waiting for a human to click
Allow/Deny on the desktop. This is the normal, documented pairing path for
both the Portal Remote Android app's "PCs on this network" and "Type address"
modes (`portal_remote/android/.../PairScreen.kt`).

## Repro (live, real device)

1. Desktop: Devices page open, `CRYOSTATION` discoverable on the LAN.
2. Phone (`adb-R5GL150LRCK-N75UEH`, real Samsung S26 Ultra): "Type address" →
   `192.168.0.137:18420` → Connect. Phone shows "Waiting for 192.168.0.137".
3. On the desktop, the request appears as a new row under "Pending requests"
   on `/devices` — but with **no toast, no tray notification, no sound, no
   badge anywhere else in the app**. `device://pending` (`pairing.rs:103`) is
   only ever listened to by `src/routes/devices/+page.svelte:88`, and its only
   effect is a silent `refetchDevices()`. If the user isn't already looking at
   that exact page, there is nothing to see.
4. It took several minutes (normal human latency: notice the phone's
   "Waiting" screen, walk to the PC, find Portal Desktop, open it, navigate to
   Devices, see the new row, read it, click Allow) before anyone clicked
   Allow.
5. By then the server's own `PAIR_REQUEST_TIMEOUT` (170s, `pairing.rs:29`) had
   already fired, auto-rejected the request (`pairing_service::reject`), and
   the phone had already shown **"The PC turned the request down"** —
   `PairApi.kt:53` maps any HTTP 403 to that exact message, which reads as a
   deliberate human decision. Nobody denied anything; nobody even saw it in
   time.
6. The late Allow click still ran against the by-then-removed pending row's
   `device_id` via a fresh row the UI had, and produced a "ghost" paired
   device: Desktop showed "Device approved" / Paired count incremented, but
   the phone's original `/pair/request` call had already died with 403 and
   never received a token — so the phone is not actually paired, while the
   desktop believes it is. (Cleaned up for this report via `revoke_device`;
   left unaddressed this would be exactly QA-113's "stale paired-looking row"
   problem, but for the discovery/manual-IP path rather than QR.)

## Impact

170 seconds is not enough time for a first-time pairing in the realistic case
where the user starts on their phone and the desktop app isn't already open
to `/devices`. This is very likely the cause of real user reports of
"pairing doesn't work" / "it says the PC turned me down" even though no one
on the PC ever made that choice — the message actively points blame at a
human decision that didn't happen.

## Suggested fix

Two independent, both worth doing:

1. **Surface the pending request outside the Devices page.** At minimum a
   toast from wherever a global toast listener lives (same place other
   cross-page notifications originate), ideally a tray/taskbar alert, so a
   request is noticeable without the user having to already be on `/devices`.
2. **Don't word the timeout path as a human rejection.** `PairResolution`
   already distinguishes nothing between "a human clicked Deny" and "nobody
   answered in time" — both become `PairResolution::Denied` → HTTP 403 →
   Android's "The PC turned the request down" (`pairing.rs:111-118`,
   `PairApi.kt:53`). Giving the timeout branch its own resolution variant (and
   a different status code, or a body field) would let the Android client say
   something honest like "Nobody answered in time — try again" instead of
   implying a deliberate no.

Also worth a look while in this code: bumping `PAIR_REQUEST_TIMEOUT` itself
further than 170s doesn't fix the root problem (no visibility) and makes
Android's own 3-minute `readTimeout` the binding constraint instead if raised
much further — the notification gap is the real fix.

## Fix

Both parts implemented:

1. **Visibility.** `device://pending` is now also heard app-wide, not just on
   `/devices`. `src/lib/components/layout/app-shell.svelte` (always mounted,
   root of every route) adds a second `listen('device://pending', ...)` in
   `onMount` (cleaned up in `onDestroy`) that shows
   `toast.info('A phone wants to pair', { description: '<device_name> is
   waiting for approval.', duration: 0, action: { label: 'Review', onClick:
   () => goto('/devices') } })` — `duration: 0` so it doesn't auto-dismiss
   before a human notices (same convention as error toasts), and the action
   button navigates straight to `/devices`. Skipped when already on
   `/devices` (that page's own listener/row covers it there, avoiding a
   redundant toast every time). `src/routes/devices/+page.svelte:88`'s
   existing listener is untouched — still does its silent `refetchDevices()`
   so the pending row appears live for anyone already on the page.

2. **Timeout identity.** `src-tauri/src/daemon/pairing.rs`: added
   `PairResolution::TimedOut`, distinct from `PairResolution::Denied`. The
   170s timeout branch in `pair_request` now produces `TimedOut` instead of
   reusing `Denied`; the explicit-Deny path (`reject_device` in
   `src-tauri/src/domains/network/commands.rs`) is untouched and still sends
   `Denied`. The HTTP layer maps `TimedOut` to `408 Request Timeout` with a
   JSON body (`{"error": "no response from host before timeout"}`), instead
   of the `403 Forbidden` used for an explicit Deny. The current Android
   client (`PairApi.kt`, separate `portal_remote` repo, not edited) only
   special-cases `403` as `PairRejected`/"turned down" — a `408` falls
   through to its generic `"The PC answered HTTP ${it.code}"` branch, which
   no longer implies a human decision that never happened. A future Android
   change can special-case 408 for a friendlier "nobody answered in time —
   try again" message; this backend change makes that signal available
   without needing to touch the other repo now.

Files changed:
- `src-tauri/src/daemon/pairing.rs` — new `PairResolution::TimedOut` variant,
  timeout branch now produces it, HTTP response maps it to 408.
- `src/lib/components/layout/app-shell.svelte` — app-wide `device://pending`
  toast listener with a "Review" action routing to `/devices`.

Checks run: `pnpm check` (0 errors, pre-existing warnings only), `pnpm lint`
(prettier/eslint/shadcn-ui all clean on touched files — the one prettier
failure reported is a pre-existing uncommitted `.claude/handoff.md` diff from
before this fix, unrelated), `cargo check` and `cargo clippy` from
`src-tauri/` (clean, no new warnings on `pairing.rs`).

Does not cover the "two stale Browser pending rows never cleaned up" item
under Related — that's a separate accumulation issue, not re-investigated
here.

## Verification

2026-11-11, mode: tauri (live, real desktop app over CDP at `127.0.0.1:9223`,
real Samsung S26 Ultra over wifi adb, PC LAN IP `192.168.0.137:18420`).
Baseline before starting: Pending 2 (pre-existing stale "Browser" rows),
Paired 1 (pre-existing "Browser" device) — both untouched, confirmed same
after cleanup.

1. **Toast off Devices route — PASS (live).** Desktop on `/` (Overview).
   Phone "Type address" → `192.168.0.137:18420` → Connect. Desktop showed a
   persistent toast "A phone wants to pair" / "Tanveer's S26 Ultra is waiting
   for approval." with a "Review" button
   (`QA-121-verify-desktop-toast.png`). Clicking Review navigated to
   `/devices` (`QA-121-verify-review-click.png`).
2. **No duplicate toast on `/devices` — PASS (live).** Dismissed the first
   toast, denied the first request (doubles as regression check 4, see
   below), re-filled the phone form and hit Connect again while already on
   `/devices`. New "Tanveer's S26 Ultra" pending card appeared instantly, no
   toast fired at any point (`QA-121-verify-test2-result.png`,
   `QA-121-verify-no-duplicate-toast-top.png`).
3. **Timeout → 408 — PASS, confirmed live via direct curl (not the 170s UI
   round-trip, to keep it deterministic/scriptable).** Ran `adb shell curl -i
   -X POST http://192.168.0.137:18420/pair/request -d
   '{"device":"QA timeout test"}'` and let it idle unanswered. After ~170s:
   `HTTP/1.1 408 Request Timeout` / `{"error":"no response from host before
   timeout"}` — distinct from the 403 seen in check 4. (A duplicate second
   curl fired by my own script error also timed out the same way, harmless.)
   Output captured in `.claude/qa/scripts/QA-121-timeout-curl-output.txt`.
   Did not additionally drive this through the real Android UI — Android's
   generic "answered HTTP {code}" fallback for non-403 codes is already
   described in the Fix section and not re-verified against the actual app
   string here (that client code lives in the other repo, out of scope).
4. **Explicit Deny still 403 — PASS (live).** From check 2's first request:
   clicked Deny on desktop promptly. Phone immediately showed "The PC turned
   the request down" (`QA-121-verify-phone-after-deny.png`) — unchanged,
   still the human-rejection path.
5. **Approve-with-Allow still works end to end — PASS (live).** New request
   from check 2, granted `agents, coder, dashboard, projects, tasks,
   terminal` (not `agents.exec`/`terminal.exec`), clicked Allow. Desktop
   showed "Device approved / Tanveer's S26 Ultra" and a paired row with
   exactly those 6 capabilities, Temporary (24h)
   (`QA-121-verify-after-approve.png`). Phone's "Waiting" screen resolved
   straight into the live remote-control UI for "CRYOSTATION"
   (`QA-121-verify-phone-after-approve.png`), confirming it actually received
   a working token, not just a desktop-side "approved" state — the exact
   ghost-pairing failure mode from the original repro did not recur.

Fixtures: the one real paired test device ("Tanveer's S26 Ultra") was revoked
via the UI's confirm dialog immediately after check 5. The two "QA timeout
test" curl-minted pending rows self-cleaned via the timeout path itself (no
manual action needed — that's the fix working as intended). Final state:
Pending 2 / Paired 1, identical to the pre-test baseline; history count grew
by the expected number of resolved test fixtures (rejected/revoked rows have
no delete path in this app, consistent with prior QA passes in this
session).

No regressions found in the neighbourhood (pending-row capability toggles,
Select-all, expiry radio buttons, the confirm dialog on Revoke all behaved
as expected while driving these flows).

## Follow-up: "Review" button looked dead (2026-11-12)

Reported after the phone→desktop pairing test: clicking **Review** on the
"A phone wants to pair" toast did nothing.

Reproduced with a real mouse on the installed release (v0.15.0): the window was
already on `/devices`, so `goto('/devices')` was a same-route no-op, and the toast
(`duration: 0`) never dismissed itself — nothing visibly changed. Toasts raised
while the window is unfocused also skip `notify()`'s "already on the target
route" early-return, so this is the normal case, not an edge. Off `/devices` the
button always navigated correctly (re-checked over CDP on the dev app with a real
`/pair/request` and a coordinate click), so the `notify`/`goto` wiring was fine.

Root cause is shared: `src/lib/components/ui/toast.svelte` wired the action
button straight to `action.onClick` and never closed the toast, for every caller
(`notify`, the agent notifications, the CoderPanel / Ollama / AI-dialog toasts).
Fix: `handleAction()` runs the action, then dismisses the toast (animated, via
the existing `handleClose`). Verified in the browser against this worktree's
dev server — both off-route (navigates and clears) and already-on-route (clears).

Not fixed here: a persistent pairing toast outlives its request (it still says
"is waiting for approval" after the request is approved, denied or timed out).
Auto-dismissing it once the request resolves would be a separate change.

## Related

Two other pending rows ("Browser", dated 06/07/2026 and 24/11/2025) have been
sitting in "Pending requests" indefinitely in this same dev instance — the
same `/pair/request`-never-answered shape, just never timed out/cleaned up
client-side either. Not chased further here (out of scope for this report),
but may be the same underlying gap, or a QA-113-shaped accumulation issue for
timed-out (not just unscanned-QR) pending rows. Worth a glance when someone
picks this up.
