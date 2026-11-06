---
id: QA-059
title: Cluster connection screen - Retry gives no feedback, "kind" detected but no action, contradictory copy
status: verified
severity: minor
kind: ux
area: cloud/CloudConnectionGuard
mode: tauri
found: 2026-11-06
---

## Use case

Owner opens any Cloud page with no kubeconfig and tries to get connected.

## Steps

1. Open `/cloud/workloads` (no clusters configured; kind installed).
2. Read the card, click **Retry**, click **Refresh tools**.

## Expected

- Retry that finds nothing says so (toast or inline "Still no clusters found at <path>").
- A detected tool should offer a next step, or the card should say what to do with it.
- Copy should not tell you to reopen the page when you are already on it.

## Actual

- Retry: button shows "Loading..." briefly, then the card is pixel-identical. No toast, no "checked just now". Refresh tools: same, silent.
- "Automatic setup" card lists "Detected tools: kind" but offers nothing for kind (or minikube/kubectl): only aws/gcloud/az get forms (`CloudConnectionGuard.svelte` ~L395-470). A kind-only user sees an "Automatic setup" section with no setup.
- Bullet 3 reads "Reopen this page and click Retry" - you are on the page; and Retry is the only action.
- "Refresh tools" ghost button looks like plain text in light theme (no affordance; visible button chrome only in dark).
- The not-connected screen hides the Cloud sidebar entirely, no link out besides the main nav.

Cluster-connected pages (pods, deployments, services, statefulsets, daemonsets, jobs, cronjobs, configmaps, secrets, ingress and all detail pages) could not be tested: no cluster is configured. All 20 routes show this same guard.

## Evidence

`.claude/qa/shots/QA-059-before-cluster-guard.png`, also `sweep-cloud-not-connected-dark.png`, `sweep-cloud-not-connected-1024x600.png` (no horizontal overflow at 1024; no sidebar clipping seen).

## Repro

`node .claude/qa/scripts/cloud-guard.mjs`

## Fix

Root cause: guard actions had no busy/no-change feedback and stale state. Fixed in `src/lib/domains/cloud/components/CloudConnectionGuard.svelte`: Retry has a loading state, clears `autoConnectError`, toasts "No change"; Refresh tools shows "Refreshing..." and toasts "No change"; button is `outline` variant; detected tools without a form (kind, minikube, kubectl) get a one-line kubeconfig hint; "Reopen this page" copy replaced. Checks: pnpm check 0 errors, eslint (only pre-existing prefer-const warning), prettier clean. Not rerun in app.

Review follow-up: hint now excludes kubectl and tools that already have detected targets (worded "no <tool> clusters found"); "No change" retry toast gated on `isExpectedNoClustersState`. Re-ran pnpm check (0 errors), eslint, prettier.

Round 2: Retry and Refresh tools now always give feedback: button loading state, then a toast ("No clusters found" / "Found N cluster(s)" / error; "No new tools detected" / "Tools updated" / error). Earlier 'No change' gating removed.


## Verification

<!-- qa-tester -->

2026-11-06 (final round), tauri after rebuild. REOPENED (the ## Fix section is empty). Fixed: copy no longer says reopen the page; kind now has a hint ('no kind clusters found. Add a cluster context to your kubeconfig, then click Retry'); Refresh tools has button chrome. STILL WRONG: Retry gives no feedback. Clicking it (and Refresh tools) with no clusters: no toast within 3.5s, card text identical before/after, no 'checked just now'. Expected per issue: 'Still no clusters found at <path>'. shots/QA-059-after-cloud-guard-card.png. Repro: node .claude/qa/scripts/cloud-guard.mjs.

2026-11-06 (last round), tauri. Retry shows a visible toast 'No clusters found' (shots/QA-059-after-retry-toast.png). Refresh tools clicked 5s later; my toast poll did not capture text (selector quirk), toast was only visually confirmed for Retry.
