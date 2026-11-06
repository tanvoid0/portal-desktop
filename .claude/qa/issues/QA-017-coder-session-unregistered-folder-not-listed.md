---
id: QA-017
title: Coder session started in a folder that is not a registered project is counted but never listed
status: verified
severity: major
kind: bug
area: ai/coder
mode: tauri
found: 2026-11-06
---

## Use case

Run the coder (Claude Code CLI engine) on an arbitrary folder (path typed/picked via the workspace
selector, which explicitly allows paths that are not projects), then come back later and reopen that session.

## Steps

1. Open `/ai/coder`, set the workspace to a folder that is not a registered project
   (QA used `.claude/qa/fixtures/QA coder fixture`), provider `Claude Code (CLI)`.
2. Send a prompt (Ctrl+Enter). The run completes ("Worked for 11s", file listing rendered).
3. Reload `/ai/coder`. Type part of the prompt in "Search sessions...".

## Expected

The session appears in the Sessions list (under its folder), so it can be reopened.

## Actual

Header says "1 session", but the WORKSPACES tree only lists registered projects (183 collapsed folders).
There is no row for the fixture folder and no row for the session. The session is unreachable from the UI.
Same without the search: counts go 2 -> 3 sessions, none of the new one visible.

## Evidence

`.claude/qa/shots/QA-017-before-session-not-listed.png`; script output "QA coder fixture folder in list: false".
Likely cause: `ProjectWorkspaceList.svelte` builds the tree from `projectService.loadProjects()` only.

## Repro

`node .claude/qa/scripts/QA-017.mjs` (needs a coder session on that folder, created as in the steps).

## Fix

Root cause: the tree only expanded the active workspace and listed 183 collapsed project rows, so a session in an unregistered folder (and search matches) were buried/never shown.
Fix: `ProjectWorkspaceList.svelte` now shows an "Other folders" group (workspaces built from session workspace paths that are not projects) at the top, always expanded so sessions can be reopened/deleted; while searching, only workspaces with matching sessions are shown, expanded.
Nit: "Other folders"/"Projects" headings are hidden when searching leaves them no matching rows.
Review: sessions with empty workspace_root are grouped under "(unknown folder)"; "Projects" heading gated on matches; "No matching sessions" shown when a search matches nothing.
Checks: pnpm check 0 errors; eslint 0 errors, prettier clean on touched files. Not run in app (tester verifies).

## Verification

<!-- qa-tester -->

2026-11-06, tauri. Reran QA-017.mjs flow: fixture coder session ('list the files here', workspace .claude/qa/fixtures/QA coder fixture) is now listed (search shows 1 session under a project group) and reopenable; 'OTHER FOLDERS' group shows unregistered-folder sessions (test-project, 9). Deleted my fixture session via the row's 'Delete session' button; after reload search shows 0 sessions. Shots: shots/QA-017-after-other-folders.png, QA-017-after-search-fixture-session.png. Nit: while searching, empty 'OTHER FOLDERS' / 'PROJECTS' headings show with no rows. Delete happened with no confirmation: see QA-041.
