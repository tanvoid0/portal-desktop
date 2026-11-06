---
id: QA-002
title: Audit folder input placeholder shows "D:\production" with a doubled backslash
status: verified
severity: polish
kind: copy
area: projects/audit
mode: browser
found: 2026-11-05
---

## Use case
Add a folder to scan; the placeholder is the hint for the expected path format.

## Steps
1. Open `/projects/audit`.

## Expected
Placeholder `D:\production`.

## Actual
Placeholder reads `D:\production`. A Svelte attribute string does not process escapes, so the TS-style `\` is literal.

## Evidence
`.claude/qa/shots/sweep-projects-audit-empty-1280.png`. `src/lib/domains/projects/components/ProjectAudit.svelte:491` `placeholder="D:\production"`.

## Repro
manual — steps above

## Fix
Root cause: a double backslash in a plain Svelte attribute renders literally. Fix: placeholder is now a JS string expression with an escaped backslash, `ProjectAudit.svelte:491`. Checks: pnpm check 0 errors; prettier clean.

## Verification

2026-11-05, browser mode (HMR build), 1280x800 and 1024x600. Placeholder attribute now reads D:\production. Verified.
