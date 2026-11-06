---
id: QA-075
title: No skip-to-content link: 31 Tab presses to reach the first control of the page
status: verified
severity: minor
kind: a11y
area: shell
mode: tauri
found: 2026-11-06
---

## Use case

Use any page with the keyboard only.

## Steps

1. `/projects`, click empty space at the top, press Tab until focus lands inside `<main>`.

## Expected

A "Skip to content" link first, or sidebar sections that are one tab stop.

## Actual

31 Tab stops (toggle, back/forward/refresh/home, breadcrumbs, every nav link, every submenu toggle, footer) before the page's own controls (search, filters, New Project). Repeats on every route. Focus ring itself is visible on all stops.

## Evidence

`.claude/qa/shots/QA-075-before-no-skip-link-focus-deep-in-sidebar.png` (12 Tabs in, still in the sidebar).

## Repro

`node .claude/qa/scripts/QA-075.mjs`

## Fix

`app-shell.svelte`: "Skip to content" link as first tab stop (sr-only until focused), targets `<main id="main-content" tabindex="-1">`. Verified: first Tab = Skip to content, Enter focuses main, next Tab lands on the page's first control. QA-075.mjs still prints 31 because it clicks into the header first, which moves the focus start past the link.

## Verification

<!-- qa-tester -->
