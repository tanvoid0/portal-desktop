---
id: QA-198
title: Terminal turns never reach a Claude Code thread that was resumed in Portal (#2017 / b473cac4 does not work in the running app)
status: open
severity: major
kind: bug
area: ai/coder/claude-code (agents + threads)
mode: tauri
found: 2026-11-27
---

## Use case

Start a Claude Code session in a terminal, carry on in Portal, go back to the terminal, and
read the whole conversation in Portal. This is the case b473cac4 set out to fix.

## Steps

1. In a scratch folder: `claude -p "reply with the single word alpha" --model haiku`.
2. Wait ~45s, open the imported session in AI > Code > Claude Code, send `reply beta`, then
   "End run".
3. In the terminal: `claude -p --resume <id> "reply gamma" --model haiku`. Wait ~45s.
4. Optional: `... --resume <id> "reply delta"` and at once send `reply epsilon` from Portal.

## Expected

gamma appears once in the thread; delta appears above epsilon.

## Actual

The thread shows alpha, beta, epsilon. gamma and delta never appear, live or after reload,
and the next Portal run's end mark (`transcript_turns` 4 -> 10) skips past them for good.
No error in the UI or the log. The session id stayed the same throughout.

The sweep did read them: `agents_get_session` rows are
alpha, beta, gamma, delta, epsilon in order, while `coder_thread_snapshot` has only
alpha, beta, epsilon.

## Evidence

`.claude/qa/shots/QA-198-before-thread-missing-terminal-turns.png`

`agents_get_session(...).events` still holds the Portal run's `agent_events` after the sweep
replaced the session row: 18 rows (written 23:12:10-23:13:16) after the 23:13:47 replace, 36
after the second run. `legacy_import::from_transcript` requires `events == 0`, so
`transcript_tail` returns `Resync::Unchanged` silently.

Where to look (inferred, not confirmed on the live DB): `agents/transcripts.rs` `store()`
deletes `agent_messages` explicitly but leaves `agent_events` to the FK cascade. The unit
test (`sweep_imports_once_dedups_across_dirs_and_reimports_on_growth`) says the cascade
works on a freshly migrated DB; on this instance (a copy of the real DB) the rows survive.
Check `PRAGMA foreign_key_list(agent_events)` / `PRAGMA foreign_keys` on a real database.
`delete_session` relies on the same cascade, so deleted sessions may leave rows behind too
(not tested).

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/QA-198.mjs` (use whichever CDP port your dev
instance has; makes three one-word haiku calls). Run once end to end on 2026-11-27 on a fresh
session (9db45f6b): `gamma in synced rows: true | gamma in thread: false`, 18 `agent_events`
survived the replace, `REPRODUCED`.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
