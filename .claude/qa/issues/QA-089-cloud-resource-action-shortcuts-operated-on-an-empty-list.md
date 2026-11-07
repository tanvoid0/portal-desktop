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

<!-- qa-tester: with a cluster connected, open /cloud/pods, press j to move the cursor onto a
     row, then d / y / l and confirm each acts on THAT row. Repeat on /cloud/deployments
     including s (scale). Also confirm an action pressed immediately after the page loads —
     before the list has visibly filled — does not act on the wrong row. -->
