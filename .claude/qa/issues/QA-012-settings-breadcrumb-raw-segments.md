---
id: QA-012
title: Settings breadcrumbs show raw URL segments ("Ides", "Github", "Ai", "Frameworks languages")
status: verified
severity: polish
kind: copy
area: shell/breadcrumbs
mode: tauri
found: 2026-11-06
---

## Use case

Read the breadcrumb on a Settings sub-page.

## Steps

Open `/settings/ides`, `/settings/github`, `/settings/ai`, `/settings/ai/training`, `/settings/frameworks-languages`.

## Expected

Names as in the Settings nav: `IDEs`, `GitHub`, `AI`, `AI > Training`, `Frameworks & Languages`.

## Actual

`Ides`, `Github`, `Ai`, `Ai > Training`, `Frameworks languages`: the QA-004 humanizer capitalises the segment only. The pages themselves are titled correctly. Needs a label map for acronyms and brands (`Github` also appears under GitHub routes elsewhere).

## Evidence

`.claude/qa/shots/QA-012-before-settings-crumb-ai-training.png`

## Repro

`node .claude/qa/scripts/QA-012.mjs`

## Fix

Root cause: QA-004 humanizer only capitalised the segment. Added `SEGMENT_LABELS` (ides, framework-ides, github, ai, sdk, frameworks-languages) in the same humanizer in `src/lib/domains/shared/stores/breadcrumbStore.ts`. Checks: pnpm check 0 errors; eslint clean.

## Verification

2026-11-06, tauri. QA-012.mjs: IDEs, GitHub, AI, AI > Training, Frameworks <!-- qa-tester --> Languages; Package managers and Autonomy fine. After-shot: shots/QA-012-after-settings-crumb-ai-training.png
