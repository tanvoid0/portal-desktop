---
id: QA-093
title: Deployments list lost the Restart action, but the `?` help still advertises `r`
status: open
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

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
