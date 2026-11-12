---
id: QA-139
title: Next.js pipeline template hard-codes npm commands and a Docker step that do not exist in the project
status: wontfix
severity: major
kind: bug
area: projects/pipelines
mode: tauri
found: 2026-11-12
---

## Steps

1. `/projects/14/pipelines/new` (aipairs: pnpm project, scripts `lint`, `typecheck`, `build`; no `test`; no Dockerfile).
2. All Templates > Next.js Full Pipeline.

## Expected

Steps mirror the project: pnpm, `pnpm typecheck`, `pnpm lint`, `pnpm build`; no test step (no script); no Docker
step without a Dockerfile; nothing deploy-flavoured preselected.

## Actual

Template steps (from `pipelineTemplateService.ts`): `npm ci`, `npm run type-check` (script is `typecheck`, would
fail), `npm test` (no such script), `npm run lint`, `npm run build`, then **Build Docker Image** (`DOCKER_COMMAND`;
no Dockerfile exists). `npm ci` on a pnpm project would wipe `node_modules` and reinstall from the other lockfile.
The description says "build, test, and deploy". Templates are filtered only by framework, never by the project's
package manager, scripts or files. (Steps were removed in the builder before saving; nothing was run.)

## Evidence

`.claude/qa/shots/QA-139-before-nextjs-template-in-builder.png`,
`.claude/qa/shots/sweep-pipelines-aipairs-builder-edited-release.png`.

## Repro

`node .claude/qa/scripts/qa-pipe-tpl.mjs 14 "Next.js Full Pipeline" QA-139-before-template-repro`

## Fix

<!-- coder -->

## Verification

<!-- qa-tester -->

## Decision (owner, 2026-11-12)

wontfix: the old template builder is being retired (QA-137). Stack-aware commands belong in the Actions defaults (QA-140).
