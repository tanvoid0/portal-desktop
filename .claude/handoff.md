# Handoff — devices/pairing QA sweep: 5 bugs fixed+verified, 1 real blocker found, not yet fixed

Written 2026-11-11, for someone with **no memory** of this session. Supersedes the previous
version of this file (the `agents`/coder claude-cli-engine work — that's fully committed, see
`cd674366`/`268121af` in git history, nothing left there).

## Goal

User reported "I still can't connect" pairing their phone to Portal Desktop. Continue the
devices/pairing QA sweep from `.claude/handoff-devices-phone-qa.md` (still present, still
accurate for its own narrower scope — QA-106/107/108/109/116), find out why real pairing
fails for the user, and fix what's fixable.

## State: all committed and verified, but **2 commits not pushed**

```
320b82a4 docs(qa): file QA-121, pairing approval window too short and invisible
18cdd163 fix(devices): QA-113/114/115/117/118 pairing polish fixes
```

Both are on top of `07263832` which **is** pushed (`origin/main` == `07263832` as of this
session's start). Nobody has asked for these two to be pushed yet — do that first if picking
this up, probably (`git push origin main`), unless the user wants to review first.

## What got done

1. **QA-113/114/115/117/118 — all fixed by a coder agent, verified live by a qa-tester agent,
   committed in `18cdd163`.** All were minor/polish bugs found in the devices/pairing domain
   (expired QR rows never pruned, a poll error could wipe the QR card, Refresh button flickered,
   unscanned QR grants couldn't be cancelled, a DB error was silently swallowed). Full detail in
   each `.claude/qa/issues/QA-11{3,4,5,7,8}-*.md` file's `## Fix` and `## Verification` sections.
   Nothing left to do here — these are done.

2. **QA-121 — found live, NOT fixed. This is almost certainly the user's actual bug.** Filed in
   `.claude/qa/issues/QA-121-pairing-approval-window-too-short-and-invisible.md`, board item #8
   (Portal MCP, assignee `owner`, priority high).

   **The bug**: `/pair/request` (discovery or manual-IP pairing, not QR) blocks server-side for
   up to 170s waiting for a human to click Allow on the Devices page
   (`src-tauri/src/daemon/pairing.rs:29` `PAIR_REQUEST_TIMEOUT`). But `device://pending`
   (emitted at `pairing.rs:103`) is **only ever listened to by
   `src/routes/devices/+page.svelte:88`**, and its only effect there is a silent
   `refetchDevices()` — no toast, no tray alert, nothing anywhere else in the app. So unless the
   user is already sitting on `/devices` when they start pairing from their phone, the request
   times out unnoticed, gets auto-rejected (`PairResolution::Denied`), and the Android app shows
   **"The PC turned the request down"** (`PairApi.kt:53` maps any HTTP 403 to that exact string)
   — which reads as a deliberate human decision that never actually happened.

   **Reproduced live**: used the real paired test phone (`adb-R5GL150LRCK-N75UEH`, a Samsung
   S26 Ultra, connected over wifi adb) to pair via "Type address" (manual IP, not QR) to
   `192.168.0.137:18420`. Took several minutes of normal exploration/debugging to get from
   "request sent" to "click Allow" on the desktop side (driving the Devices page over
   Playwright's `chromium.connectOverCDP('http://127.0.0.1:9223')`, the live CDP port of the
   already-running dev instance) — by the time Allow was clicked, the phone had already shown
   the "turned down" error, and the late approval produced an orphaned "paired" device on the
   desktop side with no matching token on the phone (a ghost pairing — cleaned up via
   `revoke_device` for this report, not left in the DB).

   **Suggested fix** (not yet implemented, see the issue file for full detail): (a) surface the
   pending request somewhere visible outside `/devices` — toast/tray at minimum; (b) give the
   170s-timeout path its own `PairResolution` variant distinct from an explicit Deny, so Android
   can say something honest like "Nobody answered in time" instead of implying rejection.

3. **Noted, not investigated further**: two stale "Browser"-type pending rows (dated
   06/07/2026 and 24/11/2025) have been sitting in "Pending requests" indefinitely in this dev
   instance's DB — same shape of bug (a `/pair/request` that was never answered and never got
   cleaned up), just for a different trust type than QR. Mentioned in QA-121's "Related" section.
   Worth a look but out of scope for this session.

4. **Clarified for the user**: the installed Android companion app (`com.portalremote`, in a
   **separate repo**, `D:/projects/production/portal/portal_remote`, not this one) has no UI at
   all for the `coder`/`terminal`/`agents`/`dashboard`/`projects`/`tasks` capabilities Portal
   Desktop's daemon exposes — its screens are Trackpad/Keyboard/Media/Remote/Monitor/
   Transfer/Deck/Browser/Assistant only (see `portal_remote/README.md`). And Portal Desktop's own
   `/control` WebSocket (`src-tauri/src/daemon/control.rs`) only implements `ping`/`pong` today —
   its own doc comment says so explicitly: "Input/mirror/audio/casting are later phases." So
   **"can the phone control the PC" is currently: pairing works (once QA-121 is fixed), but there
   is no built control surface yet** — neither trackpad-style input nor any Android UI for the
   coder/terminal/agents RPC domains that _are_ implemented. This was relayed to the user; they
   have not yet said whether to pursue testing the raw RPC layer directly (e.g. firing
   `/rpc/coder...` calls from the phone via `adb shell curl` with a granted token, the same
   technique used to verify QA-116/118) or to prioritize fixing QA-121 first.

## What's left (in order, per the user's open question)

1. **User was asked**: fix QA-121 first, or test raw RPC control first? Pick up their answer; if
   none was given, default to fixing QA-121 first — it's the actual blocker they hit.
2. **Push `18cdd163`/`320b82a4`** to `origin/main` (not yet done, nobody's explicitly asked, but
   there's no reason to hold them back — they're verified and clean).
3. If testing raw RPC control: pair the real phone (now revoked, needs re-pairing — Discover or
   manual IP both work, see QA-121's repro for the exact manual-IP sequence over adb `input text`
   if driving it headlessly), grant `coder`/`terminal`/`agents` (non-sensitive) caps, then issue
   an authenticated RPC call from the phone (`adb shell curl` with the device's bearer token
   against `http://192.168.0.137:18420/rpc/<command>`) to prove the control path works at the
   protocol level even with no Android UI for it yet. **Do not grant `terminal.exec` or
   `agents.exec` via automation** — the auto-mode classifier blocked exactly that mid-session
   (toggling the `terminal.exec` switch on a pending request got refused outright); if that
   capability needs testing, ask the user to grant it manually or get their explicit sign-off
   first.

## Files that matter

- `.claude/qa/issues/QA-121-pairing-approval-window-too-short-and-invisible.md` — the real bug,
  full repro and suggested fix.
- `src-tauri/src/daemon/pairing.rs` — `PAIR_REQUEST_TIMEOUT` (170s), `pair_request` handler,
  `PairResolution` enum (would need a third variant for "timed out" vs "denied").
- `src/routes/devices/+page.svelte:88` — the only `device://pending` listener, silent
  `refetchDevices()` only.
- `D:/projects/production/portal/portal_remote/android/.../PairScreen.kt`,
  `net/PairApi.kt` (`PairRejected`, the 403→"turned down" string), `net/PairUrl.kt` — the
  Android side, **separate repo**, read-only reference, don't edit without checking that
  project's own conventions.
- `src-tauri/src/daemon/control.rs` — confirms input/trackpad control is unbuilt (doc comment at
  top of file).
- `.claude/qa/scripts/manual-ip-*.mjs` — the Playwright-over-CDP scripts used to drive the live
  Devices page for this session's approval flow; `manual-ip-inspect.mjs` has the working pattern
  for finding/clicking shadcn `Switch`/`Button` elements by scoping through `data-slot`
  attributes via `page.evaluate` (Playwright's `.locator(...).filter({hasText})` approach failed
  / timed out for reasons not fully root-caused — the `page.evaluate` + raw DOM query approach
  worked reliably and is faster; prefer it for this app's shadcn-svelte components).
- `.claude/handoff-devices-phone-qa.md` — the prior, narrower handoff (QA-106-118 sweep before
  QA-121 was found). Still accurate, not superseded, just narrower scope than this file.

## Decisions made, don't re-litigate

- QA-121's root cause is the **notification gap**, not the 170s duration per se — raising the
  timeout doesn't fix anything if nobody ever sees the request. Fix the visibility first.
- Declined to grant `terminal.exec`/`agents.exec` to a test device via automation (the harness's
  own auto-mode classifier refused it) — treat this as a real guardrail, not a bug to work around.
  Get explicit user sign-off before granting sensitive caps to any device, test or otherwise.
- The ghost "paired" device created when my own late Allow click landed after the phone's request
  had already timed out was revoked, not left in the DB — don't be surprised if you don't see it.

## Things that bit this session

- Playwright's `locator(...).filter({ hasText: /^cap$/ })` on this app's shadcn-svelte `Switch`
  components reliably timed out (0 matches) for reasons not root-caused, despite the DOM
  structure looking correct on manual inspection. Switched to `page.evaluate` + raw
  `querySelector`/`closest`/`textContent.trim() === x` matching, which worked first try. If you
  hit the same thing, don't sink time into the Playwright locator API here — go straight to
  `page.evaluate`.
- `adb shell input text` with a literal "." does **not** reliably type a literal period into this
  app's digit-box IP entry fields — the app's own `onOctetChange` treats any non-digit character
  as "move to next box," so sending "." between octets works for _moving focus_ but can leave
  wrong values in boxes if cursor position inside a non-empty box isn't what you expect
  (backspace removes _before_ the cursor, and tapping a box doesn't guarantee cursor-at-end).
  Safest sequence per box: tap the box, `KEYCODE_MOVE_END`, several `KEYCODE_DEL`, then type the
  digits for that box only — don't try to type the whole dotted address as one `input text` call.
- Driving the live app over CDP: `http://127.0.0.1:9223` is this dev instance's actual CDP port
  (not the more commonly-assumed `:9222` — ports differ per `dev-instance.mjs` instance number).
  Confirm with `curl -s http://127.0.0.1:PORT/json/version` or `curl -s .../json` (lists open
  pages/URLs) before assuming a port. The built-in browser tool (`mcp__Claude_Browser__*`)
  **cannot** attach to an existing CDP target — it opens its own separate browser. Use
  `playwright`'s `chromium.connectOverCDP(...)` from a `node` script run via Bash instead (the
  project already has `playwright` in `node_modules`; run scripts from the project root so module
  resolution works — a script in the OS temp scratchpad dir fails with `ERR_MODULE_NOT_FOUND`).
