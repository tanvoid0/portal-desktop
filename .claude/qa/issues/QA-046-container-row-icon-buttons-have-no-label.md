---
id: QA-046
title: Container rows: Stop/Start/Remove icon buttons have no accessible name or tooltip
status: verified
severity: minor
kind: a11y
area: deployments
mode: tauri
found: 2026-11-06
---

## Use case

Stop, start or remove a container from the Local Docker list (compose stack rows).

## Steps

1. Open `/deployments`, Containers tab, expand the compose stack.
2. Inspect the square (stop) and red trash (remove) buttons on a container row.

## Expected

`aria-label` and tooltip ("Stop container", "Remove container"), like the Expand chevron next to them. Remove is a destructive red button with only an icon.

## Actual

No aria-label, no title, no text (`ContainerRow.svelte` lines ~118-128). Screen readers and keyboard users hear "button". `ContainerCard.svelte` has text labels, so the two views are also inconsistent. (Remove does ask for confirmation. Not clicked.)

## Evidence

`.claude/qa/shots/sweep-deployments-container-rows.png`; script prints 2 unnamed buttons.

## Repro

`node .claude/qa/scripts/QA-046.mjs`

## Fix

Root cause: icon-only buttons unnamed. Added aria-label (with container name) and title to Stop/Start/Remove.
Files: lib/domains/deployments/components/ContainerRow.svelte. Checks: pnpm check, eslint, prettier clean.

## Verification

2026-11-06, tauri. QA-046.mjs prints 0 unnamed buttons; labels 'Stop container postgres', 'Remove container postgres'. Nothing clicked. After: shots/QA-046-after-containers.png. Result: gone.
