---
id: UX-003
title: Nineteen files hand-roll a text-3xl page header, so the shell compaction misses them
status: verified
severity: major
kind: ux
area: sdk / cloud / deployments
mode: tauri
found: 2026-11-06
---

## Use case

Any SDK, cloud or deployment page — get past the title to the content.

## Problem

`PageHeader` is being compacted to a single ~40px row. Nineteen files never used it: they
render their own `<h1 class="text-3xl font-bold">` plus a `<p class="text-muted-foreground">`
description inside a `flex items-center justify-between` row, so they keep the old
80–100px header forever.

```
src/routes/sdk/+page.svelte
src/routes/sdk/manager/+page.svelte
src/routes/sdk/manager/[name]/+page.svelte
src/routes/sdk/services/+page.svelte
src/routes/sdk/software-installer/+page.svelte
src/routes/sdk/versions/+page.svelte
src/routes/sdk/[sdk]/+page.svelte
src/routes/sdk/[sdk]/models/+page.svelte
src/routes/sdk/[sdk]/package-manager/+page.svelte
src/routes/sdk/[sdk]/projects/+page.svelte
src/routes/sdk/[sdk]/service/+page.svelte
src/routes/sdk/[sdk]/version/+page.svelte
src/routes/cloud/workloads/daemonsets/+page.svelte
src/routes/cloud/workloads/daemonsets/[daemonset]/+page.svelte
src/routes/cloud/workloads/statefulsets/+page.svelte
src/routes/cloud/workloads/statefulsets/[statefulset]/+page.svelte
src/routes/cloud/workloads/pods/[pod]/+page.svelte
src/lib/domains/deployments/components/DeploymentDashboard.svelte
src/lib/domains/sdk/components/FlyEnvStyleDashboard.svelte
```

Measured at 1280x800:

- `/sdk` — header block **100px**, h1 `SDK Management` + "Manage your development environment
  and SDK versions" ([src/routes/sdk/+page.svelte](../../../src/routes/sdk/+page.svelte)).
- `/deployments` — header **125px**, and directly under it a second full `<Card>` holding one
  status line ("Docker · Running · v29.9.2" / "Docker is ready. Container operations are
  available.") for another **100px**
  ([src/lib/domains/deployments/components/DeploymentDashboard.svelte](../../../src/lib/domains/deployments/components/DeploymentDashboard.svelte)).
  First content row lands at y=165 with nothing said.

The descriptions are also near-worthless: "SDK Management / Manage your development
environment and SDK versions", "DaemonSets / Manage daemon workloads that run on every node".

## Expected

One ~40px header row, consistent with `/projects` and `/documents`.

## Actual

80–100px per page, 19 pages, plus a 100px status Card on `/deployments`.

## Fix

1. Replace each block with
   `<PageHeader title=… description=…>{#snippet actions()}…{/snippet}</PageHeader>`.
2. On `/deployments`, delete the Docker status Card and move its content into the PageHeader
   as `<Badge>Running</Badge>` + `v29.9.2` beside the title. "Docker is ready. Container
   operations are available." adds nothing the Running badge does not.

Net: ~60px per page × 19, plus 100px on `/deployments`.

## Evidence

`.claude/qa/shots/ux/ux-sdk.png`, `.claude/qa/shots/ux/ux-deployments.png`.

## Repro

Manual — `rg -l 'h1 class="[^"]*text-3xl' src/routes src/lib -g "*.svelte"` lists all 19;
open `/sdk` and `/deployments` to see the cost.

## Fix

<!-- coder -->

Root cause: eighteen files rendered their own `<h1 class="text-3xl">` + description row instead of
`PageHeader`, so the wave-1 header compaction skipped them. All eighteen now call `PageHeader`,
with their old right-hand buttons/badges moved into the `actions` snippet.

SDK (12): `sdk/+page.svelte`, `sdk/manager/+page.svelte`, `sdk/manager/[name]/+page.svelte`,
`sdk/services/+page.svelte`, `sdk/software-installer/+page.svelte`, `sdk/versions/+page.svelte`,
`sdk/[sdk]/+page.svelte`, `sdk/[sdk]/{models,package-manager,projects,service,version}/+page.svelte`.
Cloud (5): `cloud/workloads/daemonsets/+page.svelte` + `[daemonset]`,
`cloud/workloads/statefulsets/+page.svelte` + `[statefulset]`, `cloud/workloads/pods/[pod]/+page.svelte`.
Plus `src/lib/domains/deployments/components/DeploymentDashboard.svelte`.

Where a page had a leading `Back` button or a Devicon, it stays on the same row as the header
(`flex items-center gap-2` + `min-w-0 flex-1` wrapper), so the row is still ~40px; Devicons went
`h-12` → `h-8`.

`/deployments` Docker Card: `DockerStatusBanner` now renders **only when Docker is not running**
(`{#if status && !status.running}`) — the 100px "Docker is ready" Alert is gone, and the health
moved into the header as `badge="Running · v29.9.2"`. Fixed at the shared component, so
`SessionLauncher.svelte` (the other caller) gets the same win and keeps the Start/Retry actions
when Docker is down.

Measured, y of the first content row relative to `<main>`: `/deployments` **212 → 80**,
`/sdk` **268 → 140**, `/sdk/manager` **112 → 80**. Evidence: `shots/ux/UX-003-before-*.png`,
`shots/ux/UX-003-after-*.png`.

Not done: `FlyEnvStyleDashboard.svelte` (19th file) — excluded pending UX-017's delete decision.

Checks: `pnpm check` exit 0 / 0 ERRORS; `eslint src` 0 errors, `prettier --check src` clean.

## Verification

<!-- qa-tester -->
**2026-11-07 · tauri (real backend, CDP :9222) · 1280x800 and 1024x600, dark + light**

Visited all thirteen converted pages that render on this machine: `/sdk`, `/sdk/manager`,
`/sdk/manager/fnm`, `/sdk/services`, `/sdk/versions`, `/sdk/software-installer`,
`/sdk/nodejs` and its `models | package-manager | projects | service | version` children,
and `/deployments`. The five cloud files (daemonsets, statefulsets, their detail routes,
pods detail) stop at the cluster guard — code read only.

- **Title, description and actions survived everywhere.** Every old `h1`/`<p>` pair diffed
  against the new `title=`/`description=`: all present, only wording trims ("Manage your
  development environment and SDK versions" → "Development environment and SDK versions").
  The action-label inventory is identical old vs new across all 18 files (`Refresh`,
  `Settings`, `New Deployment`, `Install`, `Back`, `Start/Stop Service`), and `Back`
  counts are unchanged (2 → 2) on all six files that had one.
- **Buttons work:** `/sdk` Refresh refetches; `/sdk/nodejs/version` Back → `/sdk/nodejs`;
  `/deployments` → Deployments tab → `New Deployment` → `/deployments/new` (h1 "Create New
  Deployment"). `/sdk`'s `Settings` button still has no `onclick` — it was dead before
  this change too (`src/routes/sdk/+page.svelte`, both revisions), so not a regression;
  raised in the sweep report instead.
- **Leading Back / Devicon stay on the header row:** `/sdk/manager/fnm` renders Devicon +
  "Fast Node Manager" + description + "Not Installed" badge on one 32px row;
  `/sdk/nodejs/*` render Back + PageHeader on one row.
- **`/deployments`:** header is "Local Docker" + `Running · v29.9.2` badge + description +
  Refresh; the 100px "Docker is ready" Card is gone; the first content row (tab bar) is at
  y=80.
- 1024x600: header row 32–36px on all thirteen, `main.scrollWidth === clientWidth`
  everywhere, every header button and badge inside the content edge. Console errors: none.

Evidence: `.claude/qa/shots/ux/UX-003-after-deployments-verify-2.png`,
`.claude/qa/shots/ux/UX-003-after-sdk-manager-detail.png`,
`.claude/qa/shots/ux/UX-003-after-sdk-nodejs.png`,
`.claude/qa/shots/ux/UX-003-after-deployments-light.png`.

Could not reproduce, so raised in the sweep report rather than on the board: the three cloud
detail headers (`pods/[pod]`, `daemonsets/[daemonset]`, `statefulsets/[statefulset]`) used
`variant={status === 'running' ? 'default' : 'secondary'}` on the status Badge, while
`PageHeader`'s `badge` prop is always `variant="secondary"` — running and non-running now
look the same. No cluster to see it on.
