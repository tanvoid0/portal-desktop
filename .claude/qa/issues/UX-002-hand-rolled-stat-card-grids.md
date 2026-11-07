---
id: UX-002
title: Thirteen pages hand-roll a four-Card stat grid instead of using PageStats
status: verified
severity: major
kind: ux
area: dashboard / cloud / sdk / credentials / deployments
mode: tauri
found: 2026-11-06
---

## Use case

Glance at a list page and get on with the list.

## Problem

`src/lib/components/shell/page-stats.svelte` exists and is being compacted into an inline
strip. It has **exactly one caller**. Thirteen other pages render their own
`grid … md:grid-cols-4` of full `<Card>`s for four integers, so that compaction never
reaches them.

Measured cost per page: **118–150px** of vertical space for 2–6 numbers.

| file | what it counts |
| --- | --- |
| [src/routes/+page.svelte](../../../src/routes/+page.svelte) | tasks / projects / services / completion |
| [src/routes/sdk/+page.svelte:300-353](../../../src/routes/sdk/+page.svelte#L300) | managers / services / sdks / status — all four rendered `—` on load |
| [src/routes/cloud/workloads/+page.svelte](../../../src/routes/cloud/workloads/+page.svelte) | cluster totals |
| cloud/workloads/{services,deployments,jobs,cronjobs,daemonsets,statefulsets}/+page.svelte | total / running / pending / failed — **byte-identical in all six** |
| [src/lib/domains/credentials/components/CredentialVault.svelte](../../../src/lib/domains/credentials/components/CredentialVault.svelte) | total 2 / active 2 / expired 0 / types 1 |
| [src/lib/domains/deployments/components/ContainerOverview.svelte](../../../src/lib/domains/deployments/components/ContainerOverview.svelte) | container counts |
| [src/lib/domains/deployments/components/DeploymentDashboard.svelte](../../../src/lib/domains/deployments/components/DeploymentDashboard.svelte) | deployment counts |
| [src/lib/domains/sdk/components/FlyEnvStyleDashboard.svelte](../../../src/lib/domains/sdk/components/FlyEnvStyleDashboard.svelte) | sdk counts |

Worst case seen: `/credentials` holds **two** credentials and spends 118px saying
"Total 2 / Active 2 / Expired 0 / Types 1" — the same fact four times, directly above a list
where you can count them. The 3-column card grid below then leaves a third of the row empty.

## Expected

One compact strip, the way `/projects` now does it:
`Total 184 · Active this week 43 · Need attention 17 · Starred 0` in a 40px row.

## Actual

Four 118–150px Cards per page, on thirteen pages.

## Fix

Replace each grid with `<PageStats items={[…]} />`. Where a stat restates the visible list
length (credentials "Total", cloud "Total"), **drop it** — keep only the numbers that are not
derivable from the rows on screen (failed, expired, need-attention).

Net: ~100px per page × 13 pages.

## Evidence

`.claude/qa/shots/ux/ux-credentials.png`, `.claude/qa/shots/ux/ux-sdk.png`,
`.claude/qa/shots/ux/ux-home-2.png`.

## Repro

Manual — open `/credentials` and `/sdk` at 1280x800; measure the stat grid with devtools.
Or `rg -l "md:grid-cols-4" src/routes src/lib -g "*.svelte"`.

## Fix

<!-- coder -->

Root cause: twelve pages rendered their own `md:grid-cols-4` of full `<Card>`s, so the compacted
`PageStats` strip never reached them. Each grid is now a single `<PageStats stats={[…]} />` row,
with the `Total` stat dropped wherever it restates the list on screen.

Files: `src/routes/+page.svelte`, `src/routes/sdk/+page.svelte`,
`src/routes/cloud/workloads/+page.svelte` (11 cards → one strip, 11 now-unused `~icons` imports
removed), `src/routes/cloud/workloads/{services,deployments,jobs,cronjobs,daemonsets,statefulsets}/+page.svelte`,
`src/lib/domains/credentials/components/CredentialVault.svelte`,
`src/lib/domains/deployments/components/ContainerOverview.svelte`,
`src/lib/domains/deployments/components/DeploymentDashboard.svelte`.

`FlyEnvStyleDashboard.svelte` deliberately left alone — it is pending the delete decision in UX-017.

Measured: `/credentials` first credential card **308px → 204px** below `<main>`; `/sdk` tab bar
**268px → 140px**. Evidence: `shots/ux/UX-002-before-credentials.png`,
`shots/ux/UX-002-after-credentials.png`.

Checks: `pnpm check` exit 0 / 0 ERRORS; `eslint src` 0 errors, `prettier --check src` clean.

## Verification

<!-- qa-tester -->
**2026-11-07 · tauri (real backend, CDP :9222) · 1280x800 and 1024x600, dark + light**

Four strips render on this machine. `/cloud/workloads` and the six cloud list pages stop at
the "Cluster Connection Required" guard (no Kubernetes cluster here —
`shots/ux/UX-002-after-cloud-workloads-guard.png`), so those seven were verified by code
read only.

- **Numbers are right.** `/`: Tasks 19 / Completed 8 / Projects 183 / Running services 2 /
  Completion 42% — matches the sidebar badges (Tasks 19, Projects 183, SDK Manager 2), and
  8/19 = 42%. `/credentials`: Active 2 / Expired 0 / Types 1 against the two `active`
  github cards on screen. `/sdk`: Managers 3/5 matches the five managers on `/sdk/manager`
  (Conda, NVM, Rustup installed; fnm, Pyenv not). `/deployments` Containers: CPU 31.5% /
  Memory 8.8 GiB / Network; Deployments tab: Running 0 / Stopped 0 / Building 0 beside
  "No Deployments Found".
- **Wraps, does not overflow.** At 1024x600 every strip has `scrollWidth === clientWidth`,
  one row, 36px tall, and `main` does not scroll horizontally (`/` 5 stats in 926px,
  `/credentials` 3, `/deployments` 3, `/sdk` 4). The 11-stat `/cloud/workloads` strip is
  the only one that would have to wrap, and it cannot be rendered here.
- **The dropped `Total` is not missed** on the pages I could see: `/credentials` shows two
  cards, `/deployments` keeps "19 CONTAINERS · Running 17 · Stopped 2" in the Status
  breakdown donut immediately below the strip.
- **Hover `title`: partly dropped, judged acceptable, not filed.** The cloud pages carry
  `description` ("N running"), but `/`, `/credentials`, `/sdk` and both deployments strips
  have no `title` on any stat — the old card sub-lines ("N installed", "aggregate usage",
  "in use", "running containers") were deleted rather than moved. Nothing is lost: the new
  values encode it (`Managers 3/5` says more than "3 installed") and the labels already
  read "CPU (running)" / "Memory (running)".
- `/sdk` shows `—` for all four stats for ~2s while the query settles, then fills
  (3/5, 3/5, 8/11, Online) — the same pending placeholder the old cards had.
- Console errors: none on any of the four pages.

Evidence: `.claude/qa/shots/ux/UX-002-after-credentials-strip.png`,
`.claude/qa/shots/ux/UX-002-after-sdk-settled.png`,
`.claude/qa/shots/ux/UX-002-after-deployments-containers.png`,
`.claude/qa/shots/ux/UX-002-after-home-1024x600.png`.

Verified for the six pages that render here. The seven cloud strips are code-read only and
want a re-check the next time a cluster is available.
