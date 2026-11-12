---
id: QA-140
title: Default Install/Dev/Build/Test/Lint actions are generated for scripts (or whole package.json files) that do not exist
status: verified
severity: major
kind: bug
area: projects/actions
mode: tauri
found: 2026-11-12
---

Same family as QA-128 (Expo), but on plain Node and non-Node projects; reproduced on release v0.14.0.

## Steps

Open `/projects/<id>?tab=actions`, compare the default cards with the real `package.json` scripts, then Run the broken default.

## Actual (observed, release v0.14.0)

| project | default | reality | run result |
| --- | --- | --- | --- |
| #14 aipairs | Test `pnpm test` | no `test` script (has `typecheck`, `format:check`, never offered) | failed exit 1, `Command "test" not found` |
| #4 Devstrail | Test `pnpm test` | no `test` script | not run |
| #12 pocket_portal | `npm run dev/build/test/lint` | package.json has only `release*`/`sideload` scripts; also lists `npm run release`, `release:apps`, `release:server` as runnable actions | `npm run lint` failed exit 1, `Missing script: "lint"` |
| #19 plugins | `npm install/run dev/build/test/lint` | no package.json at all | `npm run lint` failed exit -4058 ENOENT package.json |
| #15 agentic-ai | `pnpm build/test/lint` | package.json only has `docker:up*` scripts | not run |
| #10 clean-portfolio | `npm install/run build/test/lint` | no root package.json (pnpm workspace) | not run |
| #13 portal_web | none wrong (test/lint exist) | | |

A Portal default pointing at a missing script is a Portal bug: defaults should be derived from `scripts` in the manifest
(and offer the real ones, e.g. `typecheck`, `format:check`), not from the framework name.

Secondary: the red error toast on a failed run dumps the whole stderr as one run-on paragraph (newlines collapsed,
`'test' is not recognized as an internal or external command` noise from a cmd wrapper); see screenshot.

## Evidence

`.claude/qa/shots/QA-140-before-test-default-missing-script.png`, `.claude/qa/shots/sweep-actions-aipairs-test-run-release.png`.

## Repro

`node .claude/qa/scripts/qa-actions-run.mjs 14 "pnpm test" QA-140-before-repro 60` and `node .claude/qa/scripts/qa-catalog-all.mjs 4 12 19 15 10`

## Fix

Root cause: `buildDefaultActions` derived Node defaults from the package-manager/framework name, never from what package.json defines. Now `ProjectAutomationProfile.scripts` (`undefined` = unprobed, `null` = no package.json) is filled by `resolveProjectCatalog`/`resolveDirectoryCatalog` (reads package.json via `coder_read_file`); Node defaults are offered only when the script exists (Expo keeps its own build/lint), `null` -> no npm defaults at all, a real `dev` script beats the metadata start command (aipairs `next start`), and the CI workflow already requires `test`, so it drops out. Review follow-ups: `null` only when the read error says file-missing (BOM stripped, bad JSON / IPC error -> unprobed, via `invokeClient`); npm-init placeholder test script excluded; start-only -> `<pm> start`; unknown workflow refs in pipeline.yml are warnings on normal load, hard error only in `reloadAndValidatePortalPipelineFile` (`strict` param). Tests: vitest actions 39 pass.
Files: src/lib/domains/actions/{defaults,resolver,types}.ts, resolver.test.ts. Checks: vitest actions (33 pass, 3 new), prettier, eslint. Not run: live app / full pnpm check. Covers QA-140 and QA-154. Not done: QA-140 offering real scripts like `typecheck`/`format:check` as extra actions, and the failed-run toast formatting (panel, off-limits).


## Verification

2026-11-12, mode tauri (dev build :9223). Catalogs via qa-v140.mjs: aipairs, Devstrail, four_thousand offer only actions with real scripts (no CI workflow, aipairs Dev Server = pnpm dev, Lint present only where defined); plugins (19) and clean-portfolio (10) show 'No actions available'; agentic-ai (15) and pocket_portal (12) only Install default plus file actions. Shot QA-154-after-catalog-14.png / -4.png.
