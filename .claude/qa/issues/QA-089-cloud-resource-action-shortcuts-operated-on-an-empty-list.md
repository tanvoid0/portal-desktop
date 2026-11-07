---
id: QA-089
title: Cloud resource action shortcuts operated on the pre-load empty list on all ten list pages
status: fixed
severity: major
kind: bug
area: cloud
mode: code read (no cluster on this machine)
found: 2026-11-07
---

## Use case

Open a cloud resource list, move the cursor to a row, and press an action shortcut —
`d` describe, `y` yaml, `l` logs, `s` scale.

## Problem

`useResourceActions` (`src/lib/domains/k8s-navigation`) was handed a plain getter or array for
its resource list. The hook reads that value **exactly once, at subscribe time**, which on
every one of the ten cloud list pages happened before `loadResources()` had returned. The
shortcuts therefore operated against the empty initial list for the lifetime of the page: the
cursor's row was never found, so the action either did nothing or resolved against the wrong
index.

This was duplicated ten times, because all ten list pages carried their own copy of the same
`useResourceActions` wiring (the subject of UX-013).

A second instance of the same class of bug was in the old deployments list, which hand-rolled
its own `<table>` to get a per-row Scale button and read the selection with a non-reactive
`get(tableNav.selectedIndex)`.

## Expected

An action shortcut acts on the row currently under the cursor, against the loaded list.

## Actual (before the fix)

It acts against the empty list captured at mount.

## Not a regression from the UX work

Pre-existing on all ten pages. Found while collapsing them under UX-013.

## Fix

The single `/cloud/[resource]` list page now passes `toStore(() => filtered)` to
`useResourceActions`, so the hook tracks the list reactively instead of snapshotting it empty.
Because there is now one list page rather than ten, there is one place for this to be right.

Landed as part of UX-013. Not committed yet.

## Repro

Needs a Kubernetes cluster, which this machine does not have — every `/cloud/*` route renders
`CloudConnectionGuard` here, so this was found and fixed by code read. It must be exercised
against a real cluster before being closed.

## Verification

**2026-11-07 — NOT closed. Code read only; this needs a cluster and still has none.**

Checked while verifying UX-013, on commit `d8ce6e3`. Nothing was clicked:
`src/routes/cloud/+layout.svelte:108` only renders `{@render children()}` inside
`{#if $cloudStore.connection.isConnected}`, so the list page cannot be mounted on this
machine at all.

What I confirmed by reading:

- The fix is present. `src/routes/cloud/[resource]/+page.svelte:123-127` passes
  `resources: toStore(() => filtered)` (`toStore` imported from `svelte/store` at line 4),
  not the bare getter.
- The root cause is where the issue says it is. `useResourceActions.ts:46-52` handles a
  getter by building a fake store whose `subscribe` calls `fn(resources())` **once** and
  returns a no-op unsubscribe — so `derived([selectedIndexStore, resourcesStore], …)` is
  frozen at the pre-load value, and `get(currentResource)` in `handleKeydown` (line ~97)
  reads `null`. `toStore` returns a real reactive readable, so the transient subscribe that
  `get()` performs now reads the current `filtered`.
- Every old call site is gone: `useResourceActions` has exactly one caller in `src/` today
  (plus the `useK8sKeyboard.ts:79` passthrough, which forwards whatever it is given). So
  the call-site fix covers everything that exists.
- **The trap itself is still in the hook.** `UseResourceActionsOptions.resources` still
  accepts `(() => ICloudResource[])` and still mishandles it silently. The next caller that
  passes a getter — the natural thing to write in Svelte 5 — gets the same dead shortcuts
  with no type error and no lint error. Worth a follow-up to delete the getter branch from
  the union, but it is not a live bug while there is one caller.

**Cannot be closed without a cluster.** Leaving `status: fixed`. To close it, run the steps
below against a real cluster:

1. `/cloud/pods` — `j` onto a row, then `d`, `y`, `l`; each must act on THAT row.
2. `/cloud/deployments` — same, plus `s` opening the Scale dialog for the selected row.
3. Press `d` immediately after the page loads, before the list visibly fills — it must act
   on the row under the cursor, not on nothing and not on a stale row.
4. Type a search query so `filtered` shrinks, then press `d` — it must act on the filtered
   row at that index, not the unfiltered one.
