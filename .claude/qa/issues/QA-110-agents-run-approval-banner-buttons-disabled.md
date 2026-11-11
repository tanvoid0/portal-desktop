---
id: QA-110
title: Agents → Run — Accept once / Accept & remember / Reject are all disabled for as long as a permission request is pending, so a tool call can never actually be approved or denied through the UI
status: verified
severity: blocker
kind: bug
area: agents/run
mode: tauri
found: 2026-11-11
---

## Use case

Agents → Run exists specifically to let a human approve or deny each tool
call an agent wants to make (the P1 "always ask-per-tool" design decision in
`claude_code.rs`'s header comment). This is the core safety feature of the
whole page.

## Steps

1. Open Agents → Run (`/agents/run`), isolated dev instance (`CDP_PORT=9223`,
   `scripts/dev-instance.mjs 2`).
2. Pick Claude Code, a working directory, and a prompt that requires a
   destructive/side-effecting tool (e.g. "Use your Bash tool to run `rm -f
   some-file` in this directory" or "...Write tool to create a file...").
   (A trivial `Read` or a benign `echo` does **not** reach this bug — those
   are correctly auto-allowed by the real CLI itself and never raise a
   permission request at all; this needs a tool call the CLI actually gates.)
3. Click "Start run". Wait for the "Approval required" banner.
4. Click "Accept once" (or "Reject").

## Expected

The click registers: the tool either proceeds (Accept) or is blocked and the
turn recovers (Reject), per `run.decide()` → `agents_decide_permission`.

## Actual

All three buttons ("Accept once", "Accept & remember", "Reject") are
`disabled` for the entire time the approval is pending — clicking does
nothing. Verified via Playwright's own click-retry log, not a visual
impression: the resolved DOM node is `<button disabled ...>` and the click
action keeps failing with "element is not enabled" for 30+ seconds straight
while the banner sits on screen.

Root cause: `ApprovalBanner`'s buttons are `disabled={busy}`
(`src/lib/domains/coder/components/ApprovalBanner.svelte`), and
`CoderFeed` passes `busy={running || decisionsDisabled}`
(`src/lib/domains/coder/components/CoderFeed.svelte`). In the **coder**
domain this is correct — its own state machine (`coderSession.svelte.ts`)
flips `rt.running = false` once a tool call is pending and only sets it
`true` again once the LLM is actually generating, so `running` genuinely
means "would be unsafe to double-submit a decision right now".

Agents' `eventsToMessages.ts` (`src/lib/domains/agents/utils/eventsToMessages.ts`)
does not have that distinction: it sets `running = true` on
`session_started`/`text_delta` and only ever sets it back to `false` on
`turn_done`/`exited`. A `permission_request` event does **not** clear
`running`, so for the entire window a tool call is paused awaiting a human
decision, `running` stays `true` — which, through the shared component,
disables the very buttons that are supposed to supply that decision. The
banner renders, looks interactive, and can never be clicked.

I confirmed the *backend* decide path itself is sound by bypassing the UI
and calling `agents_decide_permission` directly via
`window.__TAURI_INTERNALS__.invoke(...)`: both `allow: true` (file got
created) and `allow: false` (file was not created, run recovered and kept
going) worked correctly end to end. The bug is specifically the disabled
buttons in the UI, not the underlying permission/decide plumbing.

## Evidence

`.claude/qa/shots/qa-sweep2-approve-final-banner.png` — the "Approval
required" banner with all three buttons rendered.
Playwright click-retry log (from `qa-sweep2-approve-final.mjs`): 58 retries
over 30s, every one reporting `element is not enabled` for `Accept once`.
`.claude/qa/scripts/qa-sweep2-decide-direct.mjs` (`approve`/`deny` args) —
proves the backend decide path works when called directly, isolating the bug
to the disabled buttons.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-sweep2-approve-final.mjs` (point it
at any writable cwd) — starts a run that triggers a Bash `rm -f`, waits for
the approval banner, then times out trying to click "Accept once" because
it's disabled.

## Fix

Root cause: `eventsToMessages`'s reducer never cleared `running` while a
`permission_request` was outstanding, so the `busy={running}` passed into
`ApprovalBanner` (via `CoderFeed`) stayed `true` for the whole pending
window, keeping all three decision buttons `disabled`.

Fix: `eventsToMessages` now forces `running = false` whenever
`pending.length > 0`, at the end of the reducer, matching coder's own
semantics (`coderSession.svelte.ts`'s `onPending` handler sets
`rt.running = false` the moment a pending approval lands) — `running`
means "safe to submit a decision", not merely "turn in flight".

Files changed:
- `src/lib/domains/agents/utils/eventsToMessages.ts` — force `running = false`
  while any approval is pending.

Checks: `pnpm check` (0 errors) and `pnpm lint` (eslint 0 errors; prettier/
shadcn-ui clean on touched files — pre-existing unrelated formatting warning
on `.claude/handoff.md`, not touched by this change) both clean.

This same fix also unblocks QA-112, which could not be verified end-to-end
while these buttons were disabled.

Reran the repro live against the running dev instance (`:9222`, same CWD as
the original repro — had to create it, it didn't exist on this machine):
`node .claude/qa/scripts/qa-110-continue-verify.mjs` — "Accept once" was
clickable immediately (no more "element is not enabled"), the Bash tool ran,
and the turn completed: "Done — `qa-approve-marker.txt` has been removed
from the working directory." Screenshot:
`.claude/qa/shots/qa-110-no-approval-moment.png`.

## Verification

2026-11-11, mode: tauri, isolated instance 2 (`CDP_PORT=9223`,
`node scripts/dev-instance.mjs 2`), independent of the fix agent's own run.

Ran `.claude/qa/scripts/qa-verify-110.mjs` — two fresh runs, each asking for
a Write tool call:
- **Approve**: "Accept once" click resolved in 37ms (script used a 3000ms
  actionability timeout, same failure mode the original repro hit — no
  timeout this time). File `qa-verify110-approve.txt` was actually created
  on disk (confirmed via filesystem listing), and the assistant's follow-up
  message appeared.
- **Reject**: "Reject" click resolved in 9ms. File `qa-verify110-reject.txt`
  was **not** created (confirmed via filesystem listing), and the assistant
  reported "The write was blocked by a permission hook — it denied creating
  ... so the file was not created." — the run recovered cleanly, did not
  hang or error.

Screenshots: `.claude/qa/shots/qa-verify-110-approve.png`,
`.claude/qa/shots/qa-verify-110-reject.png`.

Root cause fix (`eventsToMessages.ts` forcing `running = false` while
`pending.length > 0`) confirmed present in the current source.

Both buttons clickable immediately in both outcomes, both decisions take
effect end to end, run recovers either way. **Verified.**
