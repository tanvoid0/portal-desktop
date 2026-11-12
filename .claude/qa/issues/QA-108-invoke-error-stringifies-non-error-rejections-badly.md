---
id: QA-108
title: invokeClient turns a non-string IPC rejection into the unhelpful "[object ArrayBuffer]" instead of a readable message
status: verified
severity: minor
kind: bug
area: shared/invokeClient
mode: tauri
found: 2026-11-11
---

## Use case

Any page that reads `query.error.message` after an `invokeClient`-backed
query fails (this sweep hit it via Agents → Run's backend list, but the
code path is shared by every domain).

## Steps

1. Open Agents → Run (`/agents/run`).
2. Force the underlying `agents_list_backends` IPC call to fail at the
   transport level rather than with a normal `Result::Err(String)` — e.g.
   intercept `http://ipc.localhost/agents_list_backends` and return a
   non-JSON body (this is what a malformed/non-string rejection looks like
   to `invokeClient`).
3. Reload.

## Expected

A readable fallback message ("Failed to load agent backends", or at least
something a user can read), same as the `instanceof Error` branch already
guards for in `+page.svelte`.

## Actual

`PageError` shows: **"Something went wrong" / `[object ArrayBuffer]`**.

Root cause: `src/lib/utils/invokeClient.ts:245`
```ts
let finalError = error instanceof Error ? error : new Error(String(error));
```
When the rejection value is an `ArrayBuffer` (or anything else whose
`String()` coercion isn't useful), this wraps it in a real `Error` whose
`.message` is the literal text `"[object ArrayBuffer]"`. Downstream code
like `agents/run/+page.svelte`'s `backendsQuery.error instanceof Error ?
backendsQuery.error.message : 'Failed to load agent backends'` then treats
it as a legitimate message and renders it verbatim — the friendly fallback
branch never triggers because it *is* an `Error` instance by the time it
gets there, just one with useless text.

This is a shared helper, so any domain whose IPC call fails at the
transport layer (not a normal `Result::Err(String)` from a Tauri command)
inherits the same unreadable message.

## Evidence

`.claude/qa/shots/QA-106-after-forced-error-2.png` — shows the literal
`[object ArrayBuffer]` text.

## Repro

`node .claude/qa/scripts/qa-106-forced-error2.mjs` (built for QA-106's
re-verification) — routes `http://ipc.localhost/agents_list_backends` to a
500 with a plain-text body and reloads; the resulting `PageError` message
is `[object ArrayBuffer]`.

## Fix

Root cause: `src/lib/utils/invokeClient.ts:245` built the error message with
`String(error)` for any rejection that wasn't already an `Error`, so a
non-string/non-Error rejection (e.g. an `ArrayBuffer`) coerced to the
literal `"[object ArrayBuffer]"` and got wrapped in a real `Error`, defeating
every downstream `instanceof Error` fallback.

Fix: reused the existing `getErrorMessage(error, fallback)` helper (already
imported in this file for `isVaultLockedError`) instead of `String(error)`.
It already special-cases string and `Error` shapes and returns a supplied
fallback (`'Request failed'`) for anything else, so an `ArrayBuffer` or
other opaque rejection now produces a readable message instead of garbage
text. No signature change to `request()`/`InvokeClient`.

Files changed: `src/lib/utils/invokeClient.ts` (single call site at the
`catch` in `request()`).

Checks run: `pnpm check` (0 errors, pre-existing warnings only, none in
this file), `pnpm lint` → eslint (0 errors) and shadcn-ui check both pass;
`prettier --check` only flags a pre-existing, unrelated drift in
`.claude/handoff.md`, confirmed clean on `invokeClient.ts` directly.

Does not cover QA-106/QA-107 (different root causes in the backend-load and
start-run paths) — this is the shared-helper fix for the message text only.

## Verification

2026-11-11, mode: tauri (real backend, CDP :9222, rebuilt from the working
tree including this fix). Reran `node .claude/qa/scripts/qa-106-forced-error2.mjs`
(forces `http://ipc.localhost/agents_list_backends` to 500 with a non-JSON
body). `PageError` now shows **"Something went wrong" / "Request failed"**
instead of `[object ArrayBuffer]`
(`.claude/qa/shots/QA-106-after-forced-error-3.png`). Clicking Retry after
clearing the route recovers normally. Verified.

**Re-check 2026-11-11 (same day, separate pass):** cross-checked on
instance-2 (CDP :9223) since that's the sanctioned instance for this pass —
`src/lib/utils/invokeClient.ts` is unchanged since the fix above, still
reuses `getErrorMessage`; no further live rerun needed. Still verified.

**Re-check 2026-11-12 (task #247, dev app CDP :9222, tauri mode):** qa-247-108.mjs forces agents_list_backends to 500 (sessionStorage snapshot cleared so the form is not restored): PageError shows "Something went wrong / Request failed" + Retry, no "[object ArrayBuffer]"; Retry recovers. Shot QA-108-recheck2-forced-error-2.png. Still verified.
