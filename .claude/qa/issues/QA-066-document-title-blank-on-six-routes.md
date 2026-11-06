---
id: QA-066
title: document.title is empty on /coder, /ai, /cloud, /logs, /sdk and /terminal, so the OS window title goes blank
status: verified
severity: minor
kind: bug
area: shell/routes
mode: tauri
found: 2026-11-06
---

## Use case

See which page the app is on from the window title / taskbar / alt-tab.

## Steps

1. Run `node .claude/qa/scripts/QA-067.mjs` (prints `document.title` for 17 routes), or open each route and read the title.

## Expected

Every route sets `<svelte:head><title>X - Portal Desktop</title>` like /projects ("Projects - Portal Desktop").

## Actual

`title=""` on `/terminal`, `/ai`, `/coder`, `/cloud`, `/logs`, `/sdk`. (`/terminal` was not in the lead's list; it is blank too.)

## Evidence

`.claude/qa/shots/QA-066-before-blank-title-coder.png`; QA-067.mjs output.

## Repro

`node .claude/qa/scripts/QA-067.mjs`

## Fix

`app-shell.svelte`: one fallback `<svelte:head><title>{last breadcrumb} - Portal Desktop</title>` for every route; routes that set their own `<title>` still win (verified /projects). QA-067.mjs shows a title on all 17 routes.

## Verification

<!-- qa-tester -->
