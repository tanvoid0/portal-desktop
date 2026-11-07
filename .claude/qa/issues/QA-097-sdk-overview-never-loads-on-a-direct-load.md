---
id: QA-097
title: /sdk overview is stuck on "Loading SDK data..." forever when loaded directly
status: open
severity: blocker
kind: bug
area: sdk
mode: tauri
found: 2026-11-07
---

## Use case

Open the SDK overview — the landing page of the whole SDK area, and now the only place the
Software Installer button lives.

## Steps

1. Navigate straight to `/sdk` (fresh load or reload — this is what the breadcrumb, the sidebar
   "SDK Manager" link and a window restore all do).
2. Wait.

## Expected

The four counters fill in and Quick Navigation renders, as it does when you arrive by in-app
navigation.

## Actual

It never finishes. Polled for 45s:

```
t=5s  {"loading":true,"stats":"Managers installed | — | Services running | — | SDKs installed | — | Status | — | Loading SDK data...","quick":false}
t=10s {"loading":true, …}
t=20s {"loading":true, …}
t=30s {"loading":true, …}
t=45s {"loading":true, …}
```

No console errors. The SDK sidebar on the same page finishes and shows "11 of 16 SDKs", and
`/sdk/managers` loads its cards in ~8s, so `sdkConfigsQuery` and `sdkManagersQuery` both do
resolve — only the overview's `pending` gate never clears.

Arriving at `/sdk` by clicking the SDK sidebar's "Overview" from `/sdk/managers` renders fine
(counters `3/5`, `3/5`, `8/11`, `Online`, Quick Navigation present), so the hang is specific to a
cold load of the route.

`pending = configsQuery.isPending || managersQuery.isPending`
([src/routes/sdk/+page.svelte:35](../../../src/routes/sdk/+page.svelte#L35)) — one of the two
queries stays `isPending` on a cold mount of this route.

## Evidence

`.claude/qa/shots/sdk-overview-after-wait.png`,
`.claude/qa/shots/sdk-overview-hard-load-stuck.png`,
`.claude/qa/shots/sdk-sidebar-category-expanded.png` (sidebar loaded, overview still loading),
versus `.claude/qa/shots/UX-014-after-sdk-overview-loaded.png` (same route, reached by in-app nav).

## Repro

`node .claude/qa/scripts/sdk-overview-wait.mjs` — prints the loading flag at 5/10/20/30/45s.
`node .claude/qa/scripts/sdk-overview-bug2.mjs` — shows it renders when reached by in-app nav.

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->
