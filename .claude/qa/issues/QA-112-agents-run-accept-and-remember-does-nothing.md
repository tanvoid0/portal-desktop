---
id: QA-112
title: Agents → Run — "Accept & remember" button is wired identically to "Accept once"; nothing is ever remembered
status: verified
severity: minor
kind: ux
area: agents/run
mode: tauri
found: 2026-11-11
---

## Use case

The approval banner (reused from the coder domain) shows three choices:
"Accept once", "Accept & remember" (with a "remember rule" pattern input),
and "Reject".

## Steps

1. Trigger a permission request (see QA-110 for how — note QA-110 also
   blocks actually clicking any of these three buttons; this is about what
   happens once that's fixed and "Accept & remember" becomes clickable).
2. Click "Accept & remember".
3. Trigger the same tool again later in the same run.

## Expected (assumed, not confirmed as the real intent)

Either the second call auto-proceeds without asking again, or the "remember"
option doesn't appear at all if P1 doesn't support persisted rules yet.

## Actual

`+page.svelte`'s `onDecision` handler only forwards the `approve` boolean:
```ts
onDecision={(approve) => firstPending && run.decide(firstPending.call_id, approve)}
```
`run.decide()` (`agentRun.svelte.ts`) only takes `(requestId, allow)` and
calls `agentsApi.decidePermission(runId, requestId, allow)`
(`agentsApi.ts:32-33`), which only sends `{ runId, requestId, allow }` to
`agents_decide_permission`. There is no "remember" concept anywhere in this
call chain, in `ClaudeCodeRun::decide_permission`
(`claude_code.rs:210-219`), or in the hook protocol
(`permission_decision()`). "Accept & remember" and its "remember rule" text
input are decorative — clicking it does exactly what "Accept once" does,
and the next identical tool call will ask again.

This is likely inherited as-is from reusing `ApprovalBanner`/`CoderFeed`
(coder's own domain presumably does implement remember) rather than a new
bug introduced for Agents, but as shipped it's a control that promises
something it doesn't do.

## Evidence

Code reading only — `src/routes/agents/run/+page.svelte`,
`src/lib/domains/agents/state/agentRun.svelte.ts`,
`src/lib/domains/agents/api/agentsApi.ts`,
`src-tauri/portal-core/src/domains/agents/claude_code.rs`. Not independently
exercised end-to-end in the live UI because QA-110 currently blocks clicking
either Accept button at all.

## Why needs-decision

Could be P1-scope-as-designed (remember rules weren't promised for this
domain yet) — in which case the fix is hiding/removing "Accept & remember"
from the Agents Run page rather than implementing persistence, which is a
product call, not mine to make.

## Fix

Decision: P1 scope for Agents has no remember-rule persistence (backend
`decide_permission`/hook protocol has no "remember" parameter at all —
implementing that is bigger scope than this fix pass). Faking the control
would be dishonest UI, so the "Accept & remember" button and its "remember
rule" pattern input are hidden for the Agents route specifically, rather
than building persistence. **This hides the control; it does not implement
remember-rule persistence** — flagging that explicitly for whoever revisits
this.

Change: added `showRemember?: boolean` (default `true`, so coder's existing
usage is unaffected) to `ApprovalBanner` and forwarded it through
`CoderFeed`; the Agents route (`src/routes/agents/run/+page.svelte`) passes
`showRemember={false}` to both its `CoderFeed` (first pending approval) and
the `ApprovalBanner` used for any concurrent extra approvals.

Files changed:
- `src/lib/domains/coder/components/ApprovalBanner.svelte` — `showRemember`
  prop, gates the rule input + "Accept & remember" button.
- `src/lib/domains/coder/components/CoderFeed.svelte` — forwards
  `showRemember` to `ApprovalBanner`.
- `src/routes/agents/run/+page.svelte` — passes `showRemember={false}` on
  both call sites.

Verified coder's own usage (`CoderPanel.svelte` → `CoderFeed`) doesn't pass
`showRemember`, so it defaults `true` and is unchanged.

Checks: `pnpm check` (0 errors) and `pnpm lint` (eslint 0 errors) clean —
see QA-110 for the shared run.

Depends on QA-110's fix to be clickable/verifiable end-to-end (both buttons
were disabled before that fix).

Reran live against the running dev instance (`:9222`) after the QA-110 fix:
`node .claude/qa/scripts/qa-110-continue-verify.mjs` — the approval banner
on `/agents/run` showed only "Accept once" / "Reject", no "Accept &
remember" at any point; confirmed in `.claude/qa/shots/qa-sweep2-approve-final-after-accept.png`
(an earlier banner mid-run) and the script's own assertion
(`"Accept & remember" ever shown on Agents run: false`).

## Verification

2026-11-11, mode: tauri, isolated instance 2 (`CDP_PORT=9223`), independent
of the fix agent's own run.

Ran `.claude/qa/scripts/qa-verify-112.mjs`: triggered an approval banner on
`/agents/run` (first-pending slot, the `CoderFeed`-rendered one) twice
across this sweep (once here, once incidentally in QA-110's verification) —
both times only "Accept once" / "Reject" rendered, no "Accept & remember"
button and no "remember rule" pattern input. Screenshot:
`.claude/qa/shots/qa-verify-112-agents-banner.png`.

Did not independently force a *second, concurrent* pending approval (the
`ApprovalBanner` call site for `extraPending`) in a live run — hard to
engineer deterministically — but confirmed via source that both call sites
in `src/routes/agents/run/+page.svelte` (lines 89 and 95) pass
`showRemember={false}`, and `ApprovalBanner.svelte` gates both the button and
the rule input on the same `showRemember` prop, so the same code path covers
it.

Confirmed coder's own route (`/ai/coder` → `CoderPanel.svelte` →
`CoderFeed`) does not pass `showRemember` anywhere — grepped
`src/lib/domains/coder/components/CoderPanel.svelte` and
`src/routes/ai/coder/+page.svelte`, no `showRemember` reference in either —
so it still defaults to `true` and is unaffected. Loaded `/ai/coder` live to
confirm the route still renders cleanly (no console errors); did not drive
a full coder session to visually re-confirm the button renders there, since
the prop default is unchanged and untouched by this fix.

"Accept & remember" confirmed hidden on every Agents Run approval path that
was reachable; coder's own usage unaffected. **Verified.**
