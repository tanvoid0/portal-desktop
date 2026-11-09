---
id: QA-105
title: Logs page crashes to a 500 error screen when two log entries land in the same millisecond
status: verified
severity: blocker
kind: bug
area: logs
mode: tauri
found: 2026-11-09
---

## Use case

Open the Logs page to read the live tail.

## Steps

1. `/logs` (fresh load, client-side nav from anywhere, or `page.reload()`).
2. Wait ~1s.

## Expected

The log list renders.

## Actual

Reproduced on 3/3 fresh loads this session (also hit mid-script while verifying QA-090). The
page renders the sidebar/chrome but the content pane shows a SvelteKit error boundary:

```
500 Something went wrong
Internal Error
[Go back] [Go home]
```

Console:

```
Svelte error: each_key_duplicate
Keyed each block has duplicate key `1794253242944Getting settings` at indexes 3 and 4
https://svelte.dev/e/each_key_duplicate
    ...
    at http://localhost:1420/src/lib/components/logs/UnifiedLogViewer.svelte:488:16
```

Root cause: the keyed `{#each}` over the log list
([UnifiedLogViewer.svelte:242](../../../src/lib/components/logs/UnifiedLogViewer.svelte#L242))
uses `entry.timestamp.getTime() + entry.message` as the key. `SettingsService.getSettings()`
logs `'Getting settings'` ([settingsService.ts:361](../../../src/lib/domains/settings/services/settingsService.ts#L361))
from two concurrent callers on app boot; when both calls land in the same millisecond (which
they did in all 3 repros here), the two log entries produce an identical key
(`<sameMs>Getting settings`), and Svelte's keyed each throws instead of rendering — taking
the whole page down to the generic error boundary, not just that one row.

This is timing-dependent (same source line fires from two call sites close together), not
something `UnifiedLogViewer.svelte` caused — but on this machine, right now, it reproduces
every time.

## Why blocker

The Logs page is unusable: every fresh load this session hit the crash screen before any log
entry became visible. It also blocked live verification of QA-090's "scroll to bottom" fix,
since the page never got far enough to test it.

## Evidence

`.claude/qa/shots/qa-logs-crash-check-1.png`, `-2.png`, `-3.png` — identical 500 screen, 3/3
attempts. Console error quoted above (same shape, different timestamp, in all 3).

## Repro

`node .claude/qa/scripts/app.mjs logs <name>` — any fresh load of `/logs` reproduces it in this
session; rerun a few times if it doesn't hit first try (timing-dependent on the two
`Getting settings` calls landing in the same ms).

## Fix

Root cause confirmed as described: the keyed `{#each}` used `entry.timestamp.getTime() +
entry.message` as the key, which collides whenever two entries share both a millisecond and
message text (two concurrent `'Getting settings'` callers landing in the same ms). Switched
the key to the array index (`{#each filteredLogs as entry, i (i)}`) — always unique since
`filteredLogs` is a plain array recomputed fresh each time, append-only until cleared.

File: `src/lib/components/logs/UnifiedLogViewer.svelte`.
Checks: `pnpm check && pnpm lint`.

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/app.mjs logs <name>` x5 fresh loads — 0/5
crashed (originally reproduced on every attempt). `node .claude/qa/scripts/QA-105-recheck.mjs`
— 40 rapid-fire theme toggles back-to-back with no inter-click delay (the scenario most likely
to land two "Theme changed to:" log lines in the same millisecond): 51 rows rendered, no
`each_key_duplicate`, no 500 screen, no console errors. Keying by array index instead of
`timestamp+message` removes the collision entirely regardless of how many entries land in the
same millisecond. Verified.
