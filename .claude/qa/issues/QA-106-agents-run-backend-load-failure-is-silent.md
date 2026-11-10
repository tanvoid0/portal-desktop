---
id: QA-106
title: Agents → Run launcher fails silently when the backend list can't load — empty Select, no error, no retry
status: open
severity: major
kind: bug
area: agents/run
mode: tauri
found: 2026-11-10
---

## Use case

Open Agents → Run and pick an agent backend (Claude Code) to start a run.

## Steps

1. Open Agents → Run (`/agents/run`).
2. Click the "Agent" select.
3. `agentBackendsQuery` (`agentsApi.listBackends()` → `agents_list_backends`) fails or
   never resolves to a non-empty list for any reason (backend panic, stale build missing
   the command, transient IPC error, etc. — see Evidence for how this run hit it).

## Expected

Either the query error is shown (inline message + retry, or a toast, matching how
`run.start()`/`run.sendUser()`/`run.interrupt()` already report errors via
`toast.error`), or at minimum some empty/error state distinguishable from "still
loading" / "nothing installed".

## Actual

The `Select` just sits on the "Select an agent" placeholder with an empty, silently
unopenable-looking dropdown (it does open — to zero `[role=option]` rows) forever. No
toast, no inline message, no loading spinner distinguishing from an empty result, no way
to retry. `RunLauncherForm.svelte` only branches on `selected.available` once a backend
is already selected — it never reads `backendsQuery.isError`/`.error` from
`+page.svelte` at all, so a failed query and a query that legitimately found zero
backends look identical to the user: a dead form with no way forward and no clue why.

This is exploitable at nearly 100% of the surface of this page: the whole flow (steps 2-5
of the given test plan — approval banner, deny, interrupt, cancel, bad-cwd) is gated
behind this one `Select` ever populating.

## Evidence

`.claude/qa/shots/QA-106-before-empty-backend-select-no-error.png` — select opened, zero
options, no error text anywhere on the page.

How this run hit it: this session's dev instance's Rust binary could not be rebuilt
against the uncommitted `agents` domain (`cargo` failed with "There is not enough space
on the disk (os error 112)" on `D:`, a QA-environment issue, not app code), so the
running webview's `__TAURI_INTERNALS__.invoke('agents_list_backends')` resolved to
`"Command agents_list_backends not found"` while a known-good command
(`get_all_projects`) on the same window returned real data — confirming the IPC bridge
itself was fine and this specific query failed. That root cause is environmental, but the
gap it exposed — this page has no path to tell the user their backend list failed to load
— is real and worth closing regardless of what causes the query to fail in production
(the CLI's `which` check panicking, a future second backend misbehaving, etc.).

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/QA-106.mjs` (adjust `CDP_PORT` to whichever
instance is up) — prints the option count and whether any error/retry text is visible.
Manual: steps above: once `agents_list_backends` fails for any reason, the Select stays
empty forever with no feedback.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
