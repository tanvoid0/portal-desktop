---
id: QA-058
title: Unknown URL renders a bare unstyled "404 Not Found" with no way back
status: verified
severity: minor
kind: ux
area: app-shell (found via cloud)
mode: tauri
found: 2026-11-06
---

## Use case

Owner follows a stale link or typo'd route (e.g. a bookmarked/deleted cloud resource URL).

## Steps

1. Open `/cloud/bogus` (any unmatched route).

## Expected

An error page in the app style (PageError/PageEmpty shell): what went wrong and a button back to a known page.

## Actual

Two lines of default-font text, "404" and "Not Found", top-left of the content pane. No heading, no action, no theme styling. There is no `src/routes/+error.svelte`, so SvelteKit's default is used. Console also logs two 404 resource errors.

## Evidence

`.claude/qa/shots/QA-058-before-unstyled-404.png`

## Repro

`node .claude/qa/scripts/QA-058.mjs`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

Root cause: no `+error.svelte`, so SvelteKit's default bare error page was used. Added `src/routes/+error.svelte` (renders inside the root layout's AppShell, so sidebar stays) using `PageError` plus Go back / Go home buttons. Checks: pnpm check 0 errors, eslint + prettier clean. Not rerun in app (told not to drive it).

Review follow-up: Go back falls back to `goto('/')` when `history.length <= 1`.

2026-11-06 (final round), tauri after rebuild. /cloud/bogus renders styled '404 Page not found - There is nothing at /cloud/bogus.' with Go back / Go home buttons and sidebar. shots/QA-058-after-unknown-route.png. (Console still logs one 404 resource line, harmless.)
