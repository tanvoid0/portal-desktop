---
id: QA-154
title: Default Test / Lint actions and the "CI" workflow are offered (and run) even when package.json has no such script
status: verified
severity: minor
kind: bug
area: actions/defaults
mode: tauri
found: 2026-11-12
---

## Use case

Owner opens Actions for a Node project and presses Run on the shown defaults.

## Steps

1. aipairs (id 14, no `test`), Devstrail (id 4, no `test`) or four_thousand (id 176, no `test`/`lint`) -> Actions.
2. Run `Test`, or the `CI` workflow (install -> test -> build).

## Expected

Defaults whose script is absent from package.json are hidden or marked "not defined" (QA-128 handled Expo only); CI skips missing steps.

## Actual

`Test` (`pnpm test`) and `Lint` (`npm run lint` on four_thousand) always appear; running them fails with `ERR_PNPM_NO_SCRIPT Missing script: test` and the "Run failed / Pipeline" card. The CI workflow is doomed at its test step on all three projects (the Generate agent itself noted "ci calls npm run test but package.json has no test script"). Related: aipairs `Dev Server` maps to `pnpm run start` (`next start`, needs a build) because the project's metadata start command wins; Devstrail gets `pnpm dev`.

## Evidence

`.claude/qa/shots/QA-154-before-default-test-lint-no-script.png`; run-history rows `failed pnpm --ignore-workspace test exit 1` on aipairs and Devstrail. `defaults.ts` `buildDefaultActions`, Node branch.

## Repro

`CDP_PORT=9223 node .claude/qa/scripts/qa-actions-list.mjs 14` then `qa-actions-run.mjs 14 Test <shot> 60`.

## Fix

Root cause: `buildDefaultActions` derived Node defaults from the package-manager/framework name, never from what package.json defines. Now `ProjectAutomationProfile.scripts` (`undefined` = unprobed, `null` = no package.json) is filled by `resolveProjectCatalog`/`resolveDirectoryCatalog` (reads package.json via `coder_read_file`); Node defaults are offered only when the script exists (Expo keeps its own build/lint), `null` -> no npm defaults at all, a real `dev` script beats the metadata start command (aipairs `next start`), and the CI workflow already requires `test`, so it drops out. Review follow-ups: `null` only when the read error says file-missing (BOM stripped, bad JSON / IPC error -> unprobed, via `invokeClient`); npm-init placeholder test script excluded; start-only -> `<pm> start`; unknown workflow refs in pipeline.yml are warnings on normal load, hard error only in `reloadAndValidatePortalPipelineFile` (`strict` param). Tests: vitest actions 39 pass.
Files: src/lib/domains/actions/{defaults,resolver,types}.ts, resolver.test.ts. Checks: vitest actions (33 pass, 3 new), prettier, eslint. Not run: live app / full pnpm check. Covers QA-140 and QA-154. Not done: QA-140 offering real scripts like `typecheck`/`format:check` as extra actions, and the failed-run toast formatting (panel, off-limits).


## Verification

2026-11-12, mode tauri (dev build :9223). Same checks as QA-140: no default Test on aipairs/Devstrail/four_thousand, no CI workflow, aipairs Dev Server = pnpm dev. Shots QA-154-after-catalog-14.png, QA-154-after-catalog-4.png.
