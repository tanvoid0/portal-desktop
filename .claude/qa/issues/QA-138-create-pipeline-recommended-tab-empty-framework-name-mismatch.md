---
id: QA-138
title: Create Pipeline "Recommended" tab is blank for a Next.js project (framework "Next.js" vs template "nextjs") and has no empty state
status: wontfix
severity: minor
kind: bug
area: projects/pipelines
mode: tauri
found: 2026-11-12
---

## Steps

1. `/projects/14/pipelines/new` (aipairs, a Next.js project). Default tab is **Recommended**; wait 8s.

## Expected

"Next.js Full Pipeline" recommended; or a message when nothing matches.

## Actual

The tab is completely blank (only the tab bar). "All Templates" lists Next.js Full Pipeline (framework `nextjs`).

## Evidence

`.claude/qa/shots/QA-138-before-recommended-tab-empty.png`. In code the page passes the display name
(`resolveFrameworks(project)[0].name` = "Next.js") to `getRecommendedTemplates`, which compares
`t.framework === projectFramework` against slugs like `nextjs`, `vue`, `spring-boot`; no match, so the list is
empty, and the section is hidden when empty with no fallback.

## Repro

`node .claude/qa/scripts/qa-pipe-new2.mjs 14`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

## Decision (owner, 2026-11-12)

wontfix: the old template builder is being retired (QA-137). Stack-aware commands belong in the Actions defaults (QA-140).
