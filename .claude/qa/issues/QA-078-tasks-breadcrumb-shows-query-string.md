---
id: QA-078
title: Breadcrumb gains a "?project=none&view=board" segment after switching project or view
status: verified
severity: polish
kind: bug
area: tasks / breadcrumb
mode: tauri
found: 2026-11-06
---

## Steps
1. `/tasks` -> breadcrumb "Home > Tasks".
2. Click a project in the sidebar (or the List toggle). Selection goes into the URL via `goto('?project=..&view=..')`.

## Actual
Breadcrumb becomes `Home > Tasks > ?project=none&view=board` (also `?project=none&view=archive`). Archive too.

## Expected
Breadcrumb ignores the query string.

## Evidence
`shots/sweep-tasks-revamp-multiselect-long.png` (top bar). Likely the breadcrumb builder in the app shell splits `url.pathname + search`; fix there or navigate with a pathname-relative URL.

## Verification
2026-11-06, mode: tauri. Board, list and archive views (`?project=all&view=...`) show breadcrumb `Home > Tasks` with no query-string segment (`shots/QA-076-after-colscroll-1024x600.png`, `sweep-tasks-v2-archive.png`).
