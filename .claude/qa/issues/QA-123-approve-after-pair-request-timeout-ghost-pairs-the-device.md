---
id: QA-123
title: Approving a pending request after its /pair/request long-poll already timed out silently ghost-pairs the device — phone never gets a token, desktop shows it as granted
status: fixed
severity: major
kind: bug
area: devices/pairing
mode: tauri
found: 2026-11-11
---

## Use case

Real-device pairing over the LAN discovery path (Portal Remote Android app →
"PCs on this network" → tap the desktop). The phone's `/pair/request` blocks
for up to 170s (`PAIR_REQUEST_TIMEOUT`, `pairing.rs:29`) waiting for a human
to click Allow on the desktop's Devices page. If nobody answers in time
(entirely plausible — this is QA-121's own scenario), the server times the
request out and answers the phone with `408`. The desktop operator may still
click "Allow" on that same pending card afterwards, since nothing marks it
as expired in the UI.

## Repro (live, real device)

1. Real Samsung S26 Ultra, Portal Remote app, "PCs on this network" →
   `CRYOSTATION`. Phone shows "Waiting for CRYOSTATION" and holds the
   connection open.
2. Let it sit past 170s without touching the desktop. The phone's own UI
   doesn't change while waiting (see QA-121's "Android UI not covered"
   verification note) — confirmed independently here: this request's
   underlying `device_approval` row is created at `15:08:22` and nothing
   on the phone screen indicates the first attempt died at `15:11:12`.
3. Minutes later (`15:32:47` in this run — 24 minutes after the original
   request), on the desktop's `/devices` page, open that same pending card
   and click **Allow** (granted `coder`, `terminal`).
4. Desktop: "Device approved" toast, the card moves to "Paired devices"
   with a "Temporary (24h)" badge and the granted capabilities —
   indistinguishable from a real, live pairing except for the "Never used"
   label.
5. Phone: still shows the "PCs on this network" screen with a red banner
   **"The PC answered HTTP 408"** (`QA-123-before-phone-408-after-timeout.png`)
   — it never received a token and was never waiting for this particular
   Allow click (its own request had already failed minutes earlier).

## Expected

Either the Allow button is disabled/removed once its request's long-poll has
already timed out (so there's nothing left on the other end to approve), or
approving it tells the operator plainly that the device didn't get the
grant and will need to retry pairing — not a plain "Device approved"
success toast and a normal-looking Paired entry.

## Actual

`pairing_service::approve` (`.../pairing_service.rs:198-245`) unconditionally
flips every unapproved row for the `device_id` to `approved = true`, mints a
token, and returns `success: true` — it never checks whether a live
long-poll is still waiting. The Tauri command built on top,
`approve_device` (`src-tauri/src/domains/network/commands.rs:107-139`),
then calls `resolve_pending(...)` (`pairing.rs:55-59`) to wake the
connection, but **discards its return value** — `resolve_pending` is a
documented no-op "if the request already timed out and removed itself", and
`approve_device` has no way to tell the two cases apart. The result: a
confidently "Paired" row for a device that has no token and no way to use
one short of pairing again from scratch — a ghost grant, the exact failure
mode QA-121 named and explicitly left unaddressed ("Cleaned up for this
report via `revoke_device`... left unaddressed this would be exactly
QA-113's 'stale paired-looking row' problem, but for the discovery/manual-IP
path"). QA-116 covers the same symptom for the QR path only; this is the
discovery/manual-IP counterpart, still open.

Evidence in this machine's `instance-2` database
(`%APPDATA%/com.tanvoid0.portal-desktop/instance-2/portal_desktop.db`):
row `b17f0522-4975-4f15-b204-4a206da6f8bd` — `created_at 15:08:22`,
`updated_at 15:32:47`, `approval_type temporary`, `approved 1`,
`capabilities ["coder","terminal"]` — approved live, 24 minutes after its
own request would have timed out.

## Evidence

- `.claude/qa/shots/QA-123-before-phone-408-after-timeout.png` — real phone,
  red "The PC answered HTTP 408" banner, still on the discovery screen.
- `.claude/qa/shots/QA-123-before-ghost-paired-card.png` — desktop
  Devices page, "Tanveer's S26 Ultra" / "Temporary (24h)" / "coder, terminal"
  / "Never used" card under Paired devices, from the same approval.

## Repro

Manual — needs a real device (or `curl` standing in for one, as QA-121's own
verification did) and 170s+ of wait, so no scripted repro. Steps above are
exact enough to rerun: `curl -X POST http://<host>:<port>/pair/request -d
'{"device":"QA timeout test"}'`, let it 408, then approve the resulting
pending row from `/devices` and confirm it still lands in "Paired devices".

## Fix

Root cause: `pairing_service::approve` filtered only `Approved.eq(false)`,
which also matches a row `reject()` already moved to
`approval_type = "rejected"` (the 170s `/pair/request` timeout path in
`daemon::pairing` calls `reject()`, it doesn't delete the row). A later click
on the same stale pending card re-selected that row and flipped it back to
`approved = true`, minting a token nobody receives.

Added `.filter(device_approval::Column::ApprovalType.eq("pending"))` to
`approve()`'s query (`pairing_service.rs:198-216`) — only a row still in its
initial, undecided state can be approved; a rejected/revoked row now fails
with "Device approval not found", which the existing frontend catch block in
`PendingRequestRow.svelte` already surfaces as a `toast.error`, matching this
issue's "Expected" option 2.

Files changed: `src-tauri/portal-core/src/domains/network/services/pairing_service.rs`.
Added regression test `approve_after_reject_does_not_ghost_pair`.
Checks run: `cargo test pairing_service` (10/10 pass, from `src-tauri/portal-core`
— the workspace-level `cargo test -p portal-core` is still broken by the
pre-existing tempfile dev-dependency issue tracked as board bug #6, unrelated
to this change); `cargo clippy --lib` shows no new warnings in this file.

Doesn't fix the UI staleness noted in the repro (a stale pending card can
still show as actionable after its request already timed out server-side) —
that's a UX polish gap, not the security hole; the backend now refuses the
ghost-pair either way.

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
