---
id: QA-121
title: Pairing requests die silently in 170s with no alert outside the Devices page — real-world pairing fails almost every time
status: open
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

## Related

Two other pending rows ("Browser", dated 06/07/2026 and 24/11/2025) have been
sitting in "Pending requests" indefinitely in this same dev instance — the
same `/pair/request`-never-answered shape, just never timed out/cleaned up
client-side either. Not chased further here (out of scope for this report),
but may be the same underlying gap, or a QA-113-shaped accumulation issue for
timed-out (not just unscanned-QR) pending rows. Worth a glance when someone
picks this up.
