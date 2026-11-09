---
id: QA-093
title: Deployments list lost the Restart action, but the `?` help still advertises `r`
status: verified
severity: minor
kind: bug
area: cloud
mode: code read (no Kubernetes cluster on this machine)
found: 2026-11-07
---

## What I could and could not test

There is no Kubernetes cluster configured here, so `src/routes/cloud/+layout.svelte:108`
renders `CloudConnectionGuard` and never mounts `children()` — no `/cloud/*` page below the
layout can be put on screen at all. This was found by reading the pre- and post-refactor
code side by side while verifying UX-013. It needs a cluster to be seen.

## Use case

On the Deployments list, select a row with the arrow keys and press `r` to roll the
deployment (`kubectl rollout restart` equivalent) without opening the detail page.

## Steps

1. Connect a cluster, go to `/cloud/deployments`.
2. Arrow down to select a deployment row.
3. Press `r`.
4. Open the shortcuts help (`?`) and read the Actions group.

## Expected

Either `r` restarts the selected deployment (what the old page wired up), or the `?` help
stops listing `r → Restart`, since it is reachable nowhere from a list.

## Actual

`r` refreshes the list. `Restart` is gone from every list page, but `?` still shows it.

The old `/cloud/workloads/deployments/+page.svelte` passed an `onRestart` handler calling
`k8sResourceService.rollbackDeployment()` with success/error toasts
(`d0fccf8^:src/routes/cloud/workloads/deployments/+page.svelte:144-153`). The new shared
list page `src/routes/cloud/[resource]/+page.svelte:128-144` passes no `onRestart`, and
`resourceDescriptors.ts` has no `canRestart` flag next to `canScale`/`hasLogs`.

`ACTION_SHORTCUTS` maps **both** `RESTART` and `REFRESH` to `'r'`
(`src/lib/domains/k8s-navigation/utils/keyboardConstants.ts:45-56`), and
`useResourceActions` pushes Restart before Refresh, so dropping `onRestart` silently
rebinds `r` from Restart to Refresh rather than leaving it dead.

`src/lib/domains/k8s-navigation/components/ShortcutsHelp.svelte:80` still lists
`{ key: ACTION_SHORTCUTS.RESTART, description: 'Restart' }` in the `table` context — the
help panel the UX-013 fix nominated as the replacement for pods' deleted
`KeyboardShortcutsPanel`.

Two mitigating facts, which is why this is `minor` and not `major`:

- Restart is still reachable on the deployment **detail** page
  (`src/routes/cloud/workloads/deployments/[deployment]/+page.svelte:189`).
- Before QA-089 was fixed, every list-page action shortcut acted on the empty pre-load
  list, so `r` never actually restarted anything for a user either. The capability is lost
  on paper; in practice it was already dead.

StatefulSets also had an `onRestart`, but its body was `await refreshResources()` with a
comment saying StatefulSets have no direct restart — dropping that one is pure cleanup.

## Evidence

Code read only — see the file:line references above. No screenshot: `/cloud/deployments`
cannot render without a cluster.

```
git show d0fccf8^:src/routes/cloud/workloads/deployments/+page.svelte | sed -n '144,153p'
```

## Repro

Manual — steps above, needs a cluster.

## Fix

Root cause: the shared `[resource]/+page.svelte` never passed an `onRestart` handler, so
`useResourceActions` never added the Restart entry and `ACTION_SHORTCUTS.RESTART === 'r' ===
ACTION_SHORTCUTS.REFRESH` silently rebound `r` to Refresh (per `useResourceActions.ts:114`,
Restart is pushed before Refresh so it would have taken priority had it existed).

Added `canRestart?: boolean` to `ResourceDescriptor`, set `true` only on the deployments entry
(statefulsets intentionally excluded — its old handler was a no-op cleanup per this issue), and
wired `onRestart` in `[resource]/+page.svelte` to call `k8sResourceService.rollbackDeployment`
+ toast + `loadResources`, matching the pre-refactor deployments list page's behavior exactly.

Files: `src/lib/domains/cloud/core/resourceDescriptors.ts`, `src/routes/cloud/[resource]/+page.svelte`.
Checks: `pnpm check && pnpm lint` (0 errors). Not re-verified against a live cluster (none
available here) — qa-tester should confirm `r` restarts a selected deployment row.

## Verification

2026-11-09, mode: tauri — a local cluster was connected this session, so `/cloud/deployments`
renders now, but it has 0 deployments on it. Creating a fixture `Deployment` to exercise `r`
end-to-end is out of scope (boundaries rule out Kubernetes changes), so the live restart
action itself (toast + list refresh) was not exercised. Confirmed instead by code read, which
matches the `## Fix` description exactly: `ResourceDescriptor.canRestart` added, `true` only
on the deployments entry
([resourceDescriptors.ts:142](../../../src/lib/domains/cloud/core/resourceDescriptors.ts#L142)),
`[resource]/+page.svelte` wires `onRestart` to `k8sResourceService.rollbackDeployment` + toast
+ `loadResources` when `canRestart` is set. Page itself loads cleanly on the live cluster, no
console errors, `node .claude/qa/scripts/QA-093.mjs`.

Noted but out of this issue's scope (pre-existing, not touched by this fix): the `?` shortcuts
help's Actions group is a static list for every `context='table'` resource page
([ShortcutsHelp.svelte:80](../../../src/lib/domains/k8s-navigation/components/ShortcutsHelp.svelte#L80))
— it still advertises "Restart" on Pods/Services/ConfigMaps/Secrets/etc. where `canRestart` is
false and `r` only refreshes. QA-093 was scoped to Deployments specifically and that part is
fixed; the generic help-panel-doesn't-filter-by-capability gap is wider than this one issue and
wasn't claimed as fixed here, so leaving it out of scope rather than reopening.

Verified (Deployments-scoped claim).
