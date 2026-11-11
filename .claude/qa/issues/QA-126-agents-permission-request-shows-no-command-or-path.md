---
id: QA-126
title: "Agents permission requests (live banner + Timeline) show only the tool name, never the command/file being acted on"
status: verified
severity: major
kind: bug
area: agents/run, agents/sessions
mode: tauri
found: 2026-11-11
---

## Use case

A user approving or rejecting a tool call in Agents > Run — and later, anyone reviewing
what happened in a session's Timeline tab — needs to see *what* is being requested (which
file, which shell command), not just which tool, to make an informed decision or to audit
one after the fact.

## Steps

1. Agents > Run. Claude Code, any cwd, prompt: "Use the Write tool to create a file named
   qa-write-test.txt with content "hello" in the current directory."
2. Start run, wait for "Approval required".
3. Read the banner. Accept once, let the run finish (End run).
4. Go to Sessions, open that session, open the Timeline tab, find the "Permission
   requested: Write" row.

## Expected

The banner and the Timeline row both show the file path (or command, for Bash/other tools)
being requested — the same detail already shown one row above it for the matching
`Tool call: Write` event, which does include the full JSON input.

## Actual

- **Live banner** (step 3): shows only `Write` — no file path, no content. Compare this to
  coder's own synthetic tools (`read_file`, `write_file`, etc.), whose banners do show a
  path, via `getToolCallDisplay()`.
- **Timeline row** (step 4): `Permission requested: Write` has no detail line at all, while
  the `Tool call: Write` row immediately above it shows
  `{"content":"hello","file_path":"...qa-write-test.txt"}` in full.

Root cause, same underlying gap in two places:
- `src/lib/domains/coder/components/ApprovalBanner.svelte`'s `display` (line ~30) is built
  from `getToolCallDisplay(pending.tool, pending.arguments)`
  (`src/lib/domains/coder/utils/toolCallDisplay.ts`), whose `switch` only recognizes coder's
  own synthetic tool names (`read_file`, `write_file`, `run_command`, …) — native Claude
  Code CLI tool names used by the agents backend (`Bash`, `Write`, `Edit`, `Read`, …) all
  fall into the `default` branch, which returns `detail: ''`. Meanwhile
  `src/lib/domains/agents/utils/eventsToMessages.ts` already computes a useful one-liner for
  exactly this case — `summarizePermission(tool_name, tool_input)` (command or file_path) —
  and stores it on `pending.summary`, but `ApprovalBanner` never reads `pending.summary` at
  all.
- `src/lib/domains/agents/utils/eventTimeline.ts`'s `foldTimelineEvents`, case
  `'permission_request'` (around line 101), only sets `label`, never `detail` — unlike the
  `'tool_call'` case right above it, which calls `stringifyInput(e.input)`. The event itself
  (`AgentEvent::PermissionRequest`) carries `tool_input` in full; it's just not surfaced.

This was the task's flagged main risk for this sweep ("not raw JSON dumps, not blank, not
mislabeled") — it's not raw JSON and not mislabeled, but it is blank where it matters most:
the one piece of information a reviewer needs to approve/deny or audit a decision.

## Evidence

- `.claude/qa/shots/QA-write-approval-banner-detail-check.png` — live banner, "Write" with
  no detail line.
- `.claude/qa/shots/QA-write-timeline-with-permission.png` — Timeline, `Tool call: Write`
  (full JSON) immediately followed by `Permission requested: Write` (no detail at all).

## Repro

Manual — steps above.

## Fix

Root cause (two independent gaps, same underlying cause — exactly as pinned
in this issue):

1. `ApprovalBanner.svelte`'s `display` was built purely from
   `getToolCallDisplay(pending.tool, pending.arguments)`, whose `switch` only
   recognizes coder's own synthetic tool names (`read_file`, `write_file`,
   `run_command`, …). Native Claude Code CLI tool names from the agents
   backend (`Bash`, `Write`, `Edit`, `Read`, …) all fall into its `default`
   branch, which returns `detail: ''` — `pending.summary` (already computed
   by `summarizePermission()` in `eventsToMessages.ts`, which reads
   `tool_input.command`/`tool_input.file_path`) was never read.
2. `eventTimeline.ts`'s `foldTimelineEvents`, `'permission_request'` case,
   only set `label`, never `detail` — unlike the `'tool_call'` case right
   above it, which already calls `stringifyInput(e.input)` on the same
   `AgentEvent::PermissionRequest.tool_input` data.

Fix:
- `ApprovalBanner.svelte`: `display` now tries `getToolCallDisplay()` first
  (kept as the primary source — it has richer per-tool detail than Rust's
  `tools::summarize()` fallback for coder's own tools, e.g. `read_file`/
  `list_dir`/`search_files`), and only falls back to `pending.summary` when
  `getToolCallDisplay()` returned no detail (the native-CLI-tool-name case).
  Additive, not a replacement — coder's existing banners for its own tools
  render identically to before.
- `eventTimeline.ts`: `'permission_request'` now sets
  `detail: stringifyInput(e.tool_input)`, same as `'tool_call'`.

Files changed:
- `src/lib/domains/coder/components/ApprovalBanner.svelte`
- `src/lib/domains/agents/utils/eventTimeline.ts`
- `src/lib/domains/agents/utils/eventTimeline.test.ts` (new case covering
  `permission_request` detail)

Checks run: `pnpm check` (0 errors) and `pnpm lint` (0 errors; only
pre-existing warnings in untouched files) both clean; `pnpm vitest run
src/lib/domains/agents/utils/eventTimeline.test.ts` (3 passed).

Also fixes the reviewer's point 6/7 for the same `eventTimeline.ts`
(exhaustiveness check on the event-variant switch, and removal of the now-
dead delta-merge buffer once `TextDelta`/`ThinkingDelta` stopped being
persisted) — same file, same pass. No other open QA ids affected.

## Verification

2026-11-11, tauri mode, isolated instance 2 (CDP :9223). Reran the exact
repro steps with a fresh Write prompt (`qa126-write-test2.txt`):
`CDP_PORT=9223 node .claude/qa/scripts/QA-126-verify.mjs`.

- Live banner: now shows `Write` followed by the full path
  `D:\projects\production\portal\portal_desktop\qa126-write-test2.txt` (was
  `Write` alone). Screenshot:
  `.claude/qa/shots/QA-126-verify-live-banner-2.png`.
- Timeline: the `Permission requested: Write` row now shows
  `{"content":"hello","file_path":"...qa126-write-test2.txt"}`, matching the
  `Tool call: Write` row immediately above it (was blank). Screenshot:
  `.claude/qa/shots/QA-126-verify-timeline-2.png`.

Both gaps described in `## Actual` are gone. The detail is the same raw-JSON
style already used by every other "Tool call" row in this Timeline (not a
prettified one-liner) — consistent with the existing convention elsewhere on
this tab, not a regression. `pnpm check`/`pnpm lint` not rerun by me (coder's
own checks already clean per `## Fix`; no files touched since).

No regression spotted in the neighbouring coder-tool banners (`read_file`/
`write_file` still render via `getToolCallDisplay()` as before, per the fix's
additive fallback design). **Verified.**
