---
id: UX-004
title: TabsList is forced to w-full on 13 pages, floating three labels across 1170px
status: verified
severity: minor
kind: ux
area: automation / sdk / projects / settings / deployments
mode: tauri
found: 2026-11-06
---

## Use case

Switch tabs on any page that has them.

## Problem

The vendored `TabsTrigger` already carries `flex-1`
([src/lib/components/ui/tabs/tabs-trigger.svelte:17](../../../src/lib/components/ui/tabs/tabs-trigger.svelte#L17)),
and `TabsList` already defaults to `w-fit`
([src/lib/components/ui/tabs/tabs-list.svelte:12](../../../src/lib/components/ui/tabs/tabs-list.svelte#L12)).
Thirteen callers override that default with `w-full` or `grid w-full grid-cols-N`, so each
trigger gets an equal slice of the whole content width and the label sits alone in the middle
of a 300–400px cell.

Measured on `/projects/1`: three tabs (`Overview`, `Actions`, `Terminal`) spread over 1172px
— 390px per label. On `/deployments`: two tabs over 1182px — 590px per label. The tab bar
reads as a segmented control the width of the page, which is why it looks like chrome rather
than a control.

| file:line |
| --- |
| [src/routes/automation/+layout.svelte:27](../../../src/routes/automation/+layout.svelte#L27) (`h-auto w-full justify-start` — `justify-start` is dead, `flex-1` wins) |
| [src/routes/projects/[id]/+page.svelte:460](../../../src/routes/projects/[id]/+page.svelte#L460) |
| [src/routes/sdk/+page.svelte:358](../../../src/routes/sdk/+page.svelte#L358) |
| [src/routes/sdk/ai/ollama/+page.svelte:559](../../../src/routes/sdk/ai/ollama/+page.svelte#L559) |
| [src/routes/settings/frameworks-languages/+page.svelte:43](../../../src/routes/settings/frameworks-languages/+page.svelte#L43) |
| [src/lib/domains/deployments/components/DeploymentDashboard.svelte:321](../../../src/lib/domains/deployments/components/DeploymentDashboard.svelte#L321) and `:351` |
| [src/lib/domains/ai/components/navigation/AITabBar.svelte:26](../../../src/lib/domains/ai/components/navigation/AITabBar.svelte#L26) |
| [src/lib/domains/sdk/components/ServiceConfigEditor.svelte:287](../../../src/lib/domains/sdk/components/ServiceConfigEditor.svelte#L287) |
| [src/lib/domains/sdk/components/VersionList.svelte:140](../../../src/lib/domains/sdk/components/VersionList.svelte#L140) |
| [src/lib/domains/settings/components/LearningSettings.svelte:540](../../../src/lib/domains/settings/components/LearningSettings.svelte#L540) |
| [src/lib/domains/utilities/components/EnvironmentVariablesPage.svelte:322](../../../src/lib/domains/utilities/components/EnvironmentVariablesPage.svelte#L322) |
| [src/lib/components/ThemeDisplay.svelte:55](../../../src/lib/components/ThemeDisplay.svelte#L55) |

## Expected

Tabs hug their labels and sit on the left, the shadcn default.

## Actual

Labels float centred in equal slices of the full content width.

## Fix

Drop `w-full` / `grid w-full grid-cols-N` from every `TabsList` above — the vendored default
is already `inline-flex w-fit`, so no replacement class is needed. The one that needs the
mobile escape hatch is `EnvironmentVariablesPage.svelte:322` (`w-full sm:w-auto`); make it
`w-fit` outright or keep the breakpoint there only.

Do **not** edit the files under `src/lib/components/ui/tabs/` — they are vendored.

## Evidence

`.claude/qa/shots/ux/ux-projects-1.png` (3 tabs / 1172px),
`.claude/qa/shots/ux/ux-deployments.png` (2 tabs / 1182px),
`.claude/qa/shots/ux/ux-automation-blocks.png` (4 tabs / 1124px).

## Repro

Manual — open `/projects/1` and look at the tab bar.

## Fix

<!-- coder -->

Root cause: thirteen callers overrode the vendored `w-fit` default on `TabsList`. The override is
removed at every site; no replacement class was needed.

`ThemeDisplay.svelte:55`, `AITabBar.svelte:26` (kept `h-auto gap-1 p-1`, dropped
`w-full justify-stretch`), `DeploymentDashboard.svelte:321` and `:351`, `ServiceConfigEditor.svelte:287`,
`VersionList.svelte:140`, `LearningSettings.svelte:540`, `EnvironmentVariablesPage.svelte:322`
(`w-full sm:w-auto` dropped outright), `automation/+layout.svelte:27` (kept `h-auto overflow-x-auto`),
`projects/[id]/+page.svelte:460` (the `hasModules` grid-cols expression went with it),
`sdk/+page.svelte:358`, `sdk/ai/ollama/+page.svelte:559`, `settings/frameworks-languages/+page.svelte:43`.

Measured tablist widths at 1280x800 (main = 1232px): `/projects/4` 3 tabs **1172 → 307px**,
`/deployments` 2 tabs **1182 → 266px**, `/automation/blocks` 4 tabs **1124 → 292px**,
`/utilities/environment` 173px, `/settings/frameworks-languages` 304px.
Evidence: `shots/ux/UX-004-after-projects-detail-tabs.png`.

Checks: `pnpm check` exit 0 / 0 ERRORS; `eslint src` 0 errors, `prettier --check src` clean.

## Verification

<!-- qa-tester -->
**2026-11-07 · tauri (real backend, CDP :9222) · 1280x800 and 1024x600**

`rg` over `src` finds no `TabsList` still carrying `w-full` or `grid-cols-N`. Walked ten
of the thirteen sites live at both viewports and clicked the second tab on each:

| route | tabs | width @ main 1232 | @ main 976 | switch |
| --- | --- | --- | --- | --- |
| /projects/4 | 3 | 307 (was 1172) | 307 | ok |
| /sdk | 4 | 363 | 363 | ok |
| /sdk/ai/ollama | 5 | 368 | 368 | ok |
| /deployments | 2 | 266 (was 1182) | 266 | ok |
| /ai/chat (AITabBar) | 2 | 157 | 157 | ok |
| /sdk/nodejs/version (VersionList) | 2 | 209 | 209 | ok |
| /settings/learning | 3 | 412 | 412 | ok |
| /utilities/environment | 2 | 173 | 173 | ok |
| /settings/theme (ThemeDisplay) | 3 | 256 | 256 | ok |
| /automation/blocks | 4 + 2 | 292 / 158 | 292 / 158 | ok |

Every list has `scrollWidth === clientWidth` at 1024x600 — they hug their labels and do not
clip — and `aria-selected` moves to the clicked tab on all ten. Three sites are code-only:
`ServiceConfigEditor.svelte` (its dialog did not open from `/sdk/services` → `Manage` in
this session) and `settings/frameworks-languages/+page.svelte`, which UX-015 deleted.
Console errors: none.

Evidence: `.claude/qa/shots/ux/UX-004-after-deployments-tab-switch.png`.
