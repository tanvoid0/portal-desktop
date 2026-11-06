---
id: QA-084
title: Coder turn summary reports "Created 2 files" when both write_file calls failed
status: fixed
severity: major
kind: bug
area: coder / feed summary
mode: tauri
found: 2026-11-06
---

## Steps

1. AI -> Code, engine **Ollama**, workspace `todo-rn`, permission chip **Auto**.
2. New session, then ask for an edit the model will fumble, e.g.
   _"In App.tsx, add a Text element directly under the heading that shows how many todos are not done, e.g. '2 remaining'. Nothing else."_
3. Read the one-line activity summary on the finished turn.

## Actual

The summary reads:

```
Created 2 files, Read a file, Listed a directory
Worked for 25s
```

Nothing was created. Both `write_file` calls returned an error. From
`coder_threads.messages_json` on thread `77074dc4-349b-410a-a9fc-890c0aa4d3c1`:

```
tool | .gitignore\nApp.tsx\nREADME.md\napp.json\nindex.ts\n...   <- list_dir   ok
tool | import AsyncStorage from '@react-native-async-storage/...  <- read_file  ok
tool | Error: missing arg: path                                   <- write_file FAILED
tool | Error: missing arg: path                                   <- write_file FAILED
```

`coder_file_changes` for that thread is **0 rows**, which is correct — the backend's
`record_change` is guarded on the result not starting with `Error: `
([service.rs:781-791](../../../src-tauri/portal-core/src/domains/coder/service.rs#L781)).
So the backend knew the writes failed; only the summary does not.

## Root cause

[feedBlocks.ts:179](../../../src/lib/domains/coder/utils/feedBlocks.ts#L179) builds the
phrase from `bucket.total`, the raw count of tool *invocations* by tool name, and
`activityPhrase` ([feedBlocks.ts:76](../../../src/lib/domains/coder/utils/feedBlocks.ts#L76))
maps `write_file` -> `Created`. Neither looks at the tool result. There is no `isError`
check anywhere in the file, so a failed call is indistinguishable from a successful one.

## Expected

A failed tool call is not counted as a success. Either exclude results starting with
`Error: ` from the bucket total, or surface them, e.g.
`Created 0 files (2 failed), Read a file, Listed a directory` with the error text
reachable from the summary.

## Why it matters

The whole point of the summary is to tell the user what the agent did without reading the
transcript. Here it asserts that two files were created when the turn in fact changed
nothing, and the error (`missing arg: path`) is never shown. This is the
"do not trust the agent's self-report" trap from `.claude/handoff.md`, except the UI is
now the thing doing the misreporting — a user has no reason to doubt it.

Note the underlying tool failure is **not** a Portal bug: the `write_file` spec correctly
declares `path` as required ([tools.rs:28-39](../../../src-tauri/portal-core/src/domains/coder/tools.rs#L28)),
and `gemma4:e4b` simply omitted it. Portal's job here is to report that honestly.

## Evidence

- `.claude/qa/shots/coder-B-ollama-turn.png`

## Fix — 2026-11-06

`feedBlocks.ts` now tracks failures per tool bucket and never counts them as work done:

- `ToolBucket` gained `failed`, incremented in the same branch that already set `hasFailed`.
- The summary is built from `succeeded = total - failed`:
  - some succeeded, some failed -> `Created a file (1 failed)`
  - none succeeded -> `Failed creating 2 files` (reuses the existing in-progress
    phrasing as a gerund, so no new per-tool vocabulary was needed)
  - nothing failed -> unchanged
- In-progress calls still count toward the active phrase, so a running turn reads the
  same as before.

Covered by `src/lib/domains/coder/utils/feedBlocks.test.ts` (5 cases, including the exact
two-failed-`write_file` case from this report).

**Verified in the running app on the original data.** Reopening thread
`77074dc4-349b-410a-a9fc-890c0aa4d3c1` — the very turn this issue was filed from — now
renders:

```
Failed creating 2 files, Read a file, Listed a directory
```

where it previously read `Created 2 files, Read a file, Listed a directory`.
Shot: `.claude/qa/shots/coder-QA-084-after-fix.png`.
