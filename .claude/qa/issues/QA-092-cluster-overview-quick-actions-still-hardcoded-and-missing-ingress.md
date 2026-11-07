---
id: QA-092
title: Cluster overview Quick Actions is still a hand-written list — no Ingress tile, and one tile links to the page you are on
status: open
severity: minor
kind: ux
area: cloud
mode: code read (no Kubernetes cluster on this machine)
found: 2026-11-07
---

## What I could and could not test

No Kubernetes cluster here, so `/cloud/workloads` renders `CloudConnectionGuard` and the
Quick Actions card never appears (`src/routes/cloud/workloads/+page.svelte:195` also gates
it behind `connection.isConnected`). Found by code read while verifying UX-013. Needs a
cluster to be seen.

## Use case

From the cluster overview, jump to any resource list.

## Steps

1. Connect a cluster, go to `/cloud/workloads`.
2. Read the Quick Actions card at the bottom.

## Expected

One tile per resource — the same ten the sidebar and the command palette now generate from
`RESOURCE_DESCRIPTORS` — and no tile that goes nowhere.

## Actual

`src/routes/cloud/workloads/+page.svelte:311-392` is ten hand-written `Button`s with
hand-copied emoji and labels:

- **Ingress is missing.** Nine of the ten descriptors have a tile; Ingress has none. The
  sidebar (Networking group) and the palette (`Go to Ingress`) both have it, so the
  overview is the one place that already drifted.
- **"View Workloads" navigates to `/cloud/workloads`** — the page the button is on. It
  was already like this before the refactor.
- The grid is a third copy of a list that UX-013 collapsed everywhere else. The emoji and
  labels are duplicated from `resourceDescriptors.ts` and will drift again the next time a
  resource type is added.

UX-013's fix repointed these ten `goto` targets to the new `/cloud/<slug>` URLs (correctly
— every target resolves) but left the list hardcoded, so the root cause UX-013 names
survives on this one screen.

## Expected shape of the fix

`{#each RESOURCE_DESCRIPTORS as d}` with `d.icon` / `d.label` / `/cloud/${d.slug}`, and
drop the self-referential Workloads tile.

## Evidence

Code read only — `src/routes/cloud/workloads/+page.svelte:311-392` against
`src/lib/domains/cloud/core/resourceDescriptors.ts`. No screenshot: the card cannot render
without a cluster.

## Repro

Manual — steps above, needs a cluster.

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->
