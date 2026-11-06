---
id: QA-067
title: Every page has two h1 (shell "Portal Desktop" + page heading); /ai and /coder also nest a second <main>; /terminal /ai /coder /cloud have no page heading at all
status: verified
severity: minor
kind: a11y
area: shell/routes
mode: tauri
found: 2026-11-06
---

## Use case

Screen-reader / outline navigation: know what page you are on.

## Steps

1. `node .claude/qa/scripts/QA-067.mjs`.

## Expected

One h1 per page (the page title); the sidebar brand is not an h1. One `<main>` landmark.

## Actual

- Every route: first h1 is the sidebar brand "Portal Desktop", the real heading is a second h1 (`/utilities` has three: "Portal Desktop", "Utilities", "Disk Utility").
- `/sdk`: the shell h1 is "SDK Manager" and the page heading second.
- `/terminal`, `/ai`, `/coder`, `/cloud`: the only h1 is "Portal Desktop".
- `/ai`, `/coder`: `main=2` (nested `<main>`; same as QA-014 on settings).

## Evidence

`.claude/qa/shots/QA-067-before-two-h1-nested-main-ai.png`; QA-067.mjs output.

## Repro

`node .claude/qa/scripts/QA-067.mjs`

## Fix

Sidebar brand `<h1>` -> `<span>` (`app-shell.svelte`); `/utilities` layout section header h1 -> div; added `sr-only` h1 to `/terminal`, `ai/+layout` (covers /ai and /coder) and `cloud/+layout`; nested `<main>` -> `<div>` in `CoderPanel.svelte` and `ai/chat/+page.svelte`. QA-067.mjs: exactly one h1 and one main on all 17 routes.

## Verification

<!-- qa-tester -->
