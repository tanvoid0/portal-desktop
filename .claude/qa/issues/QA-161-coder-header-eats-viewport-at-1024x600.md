---
id: QA-161
title: "Coder page at 1024x600: header stack takes ~320px of 600 and pushes the chat down (QA-023 regressed)"
status: verified
severity: major
kind: ux
area: ai/coder
mode: tauri
found: 2026-11-22
---

## Use case

Use the AI coder in a small window.

## Steps

1. Open /ai/coder at 1024x600.

## Expected

Chat and composer own most of the window.

## Actual

Home/Code tabs, engine tabs (Portal, Claude Code, Codex, Cursor, Overview), Agent Platform + provider + model selects, panel icons, git delta and Commit each wrap onto their own row. "What should we build?" is half hidden above the composer; the right panel and "New session" row add more chrome. At 1280x800 the same controls form two nav rows (Home/Code and Portal/Claude Code/...) so it is unclear which tab set is primary; "Git changes +1268 -1352" appears three times (header, panel, composer chip) and the panel label truncates to "Git chan...". Lens: impeccable critique (hierarchy), emil-design-eng (one control, one place). Also see sweep-coder-light-1280.png.

## Evidence

`.claude/qa/shots/QA-161-before-coder-header-1024x600.png`

## Repro

manual - steps above

## Fix

Root cause: CoderTopbarActions parked ~8 controls in the shared AI tab-bar row, duplicating the workspace sidebar (Agent changes, Git changes, and Commit via the Git changes panel) plus a stacked "Agent Platform" label; the narrow flex column made every control wrap onto its own row.
Fix: removed the topbar Agent changes, Git changes and Commit buttons (all still in CoderWorkspaceSidebar / GitChangesPanel), hid the platform label, narrowed the selects, Terminal label now 2xl-only. Header at 1280 is one row (was two + orphan); "Git changes" now appears twice (sidebar, composer chip) instead of 3x.
Files: src/lib/domains/coder/components/CoderTopbarActions.svelte, CoderPanel.svelte (dropped unused props).
Checks: pnpm check 0 errors, eslint + prettier on the 2 files clean. Shots: QA-161-after-coder-header-1280x800-d.png, QA-161-after-coder-header-1024x600-final.png.
Remaining (outside my file scope): at 1024 the actions column in src/routes/ai/+layout.svelte is still too narrow (selects stack, 2 rows ~190px vs ~320px before). Suggested: give the actions div `min-w-[20rem]` and the bar `flex-wrap` so actions take one row below the tabs. Not changed; needs the lead's OK.

## Verification

<!-- qa-tester -->
