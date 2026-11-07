---
id: UX-015
title: Settings has two ways into Languages / Package Managers / Frameworks, and 16 routes where 12 would do
status: verified
severity: minor
kind: ux
area: settings
mode: tauri
found: 2026-11-06
---

## Use case

Change a language or package-manager setting.

## Problem — the same three panels, two routes

[src/routes/settings/frameworks-languages/+page.svelte](../../../src/routes/settings/frameworks-languages/+page.svelte)
is a 91-line tab hub that renders `LanguagesSettings`, `PackageManagersSettings` and
`FrameworksSettings`. Those same three components are also each rendered by a 22-line
standalone route:

```
/settings/frameworks-languages?tab=languages        → LanguagesSettings
/settings/languages                                  → LanguagesSettings
/settings/frameworks-languages?tab=package-managers  → PackageManagersSettings
/settings/package-managers                           → PackageManagersSettings
/settings/frameworks-languages?tab=frameworks        → FrameworksSettings
/settings/frameworks                                 → FrameworksSettings
```

And the settings nav offers **both**:
[SettingsNavigation.svelte:132-155](../../../src/lib/domains/settings/components/SettingsNavigation.svelte#L132)
lists `Frameworks & Languages` (→ the hub) with three `subSections` pointing at the
standalone routes. So clicking the parent and clicking a child land you on different routes
showing the same panel, and the tab inside the hub does not track which child the nav thinks
is active.

Same shape elsewhere in settings:

- `/settings/ai` (13 lines) + `/settings/ai/training` (53) are a parent/child pair where the
  parent is a bare wrapper.
- `/settings/ides` (20 lines) wraps `IdeSettings.svelte`, which is **1187 lines** and holds
  both IDE configuration and framework→IDE mappings (its page renders `h2` "IDE
  Configuration" twice — the audit found two `<h2>IDE Configuration</h2>` on one screen).

## Fix

**Route map before:** 16 routes under `/settings`.

**After:** 12.

```
delete  /settings/frameworks-languages      (the hub)
keep    /settings/languages
keep    /settings/package-managers
keep    /settings/frameworks
```

1. Delete `src/routes/settings/frameworks-languages/+page.svelte` and redirect it to
   `/settings/languages` (`+page.ts` with `redirect(302, …)`), preserving `?tab=` →
   the matching route.
2. In `SettingsNavigation.svelte`, make `Frameworks & Languages` a non-clickable group
   label over its three children rather than a section with its own `path`. Drop
   `isFrameworksLanguagesSection` and the forced-open logic (`:63-92`) with it.
3. Do the same for `ai`: make it a group label over `Providers` and `Training Data`, and
   delete `/settings/ai/+page.svelte`, redirecting `/settings/ai` → `/settings/ai/providers`.

This is also the change that makes UX-005 (single-line nav items) read correctly: group
labels and leaves instead of fourteen equal two-line entries.

## Evidence

`.claude/qa/shots/ux/ux-settings-ides.png` — the audit reports `h2` list
`["IDE Configuration", "IDE Configuration", "Framework IDE Mappings"]` on one page.
Code paths quoted above.

## Repro

Open `/settings/frameworks-languages?tab=languages` and `/settings/languages` side by side.

## Fix

<!-- coder -->

Root cause: two routes rendered the same three panels and the settings nav offered both, with
two parent sections (`frameworks-languages`, `ai`) that were links *and* group headers.

- Deleted `src/routes/settings/frameworks-languages/+page.svelte`; new
  `src/routes/settings/frameworks-languages/+page.ts` redirects 307 to `/settings/languages`,
  `/settings/package-managers` or `/settings/frameworks` by `?tab=`.
- Moved `src/routes/settings/ai/+page.svelte` to `src/routes/settings/ai/providers/+page.svelte`;
  new `src/routes/settings/ai/+page.ts` redirects 307 to `/settings/ai/providers` (keeps
  `LearningSettings`' `goto('/settings/ai')` and any bookmark working).
  `src/lib/config/ai-nav.ts` `AI_PROVIDER_SETTINGS_PATH` now points at the real page, so the
  ~10 call sites that use it skip the hop.
- `SettingsNavigation.svelte`: sections with `subSections` render as non-clickable group labels
  over their leaves. Dropped the Collapsible, `submenuExpanded` / `isSubmenuExpanded` /
  `handleSubmenuOpenChange`, `isFrameworksLanguagesSection` / `isAiSection` forced-open logic,
  `handleCollapsedSectionClick`'s `?tab=` preservation, `SettingsSectionType`, the
  `activeSection` / `activeSubSection` derivations and the `currentSection` prop — active state
  is now a pathname match (`-212` lines). `+layout.svelte` lost its `currentSection` derivation.
- Side effect: `/settings/ides` no longer shows `IDE Configuration` twice (the route-level copy
  is gone; the one inside `IdeSettings.svelte` remains).

Checks: `pnpm check` exit 0, `0 ERRORS`; `pnpm lint` exit 0. Walked all 14 nav entries and all
retired paths live over CDP with `.claude/qa/scripts/ux-015-verify.mjs` — every entry lands on a
real page, `/settings/frameworks-languages[?tab=…]`, `/settings/ai`, `/settings`,
`/settings/framework-ides` and `/ai/providers` all redirect, none 404.

Also covers the heading half of UX-005 (per-page `<h2>` + description deleted from all 16
settings route pages); UX-005's nav sizing and badge row are a separate pass and stay open.

## Verification

<!-- qa-tester -->
**2026-11-07 · tauri (real backend, CDP :9222)**

Rerun with `node .claude/qa/scripts/ux-settings-nav.mjs` — the coder's
`ux-015-verify.mjs` can no longer run (Playwright's `connectOverCDP` now throws an
assertion on this app's attached `shared_worker` target); the new script drives the page
target directly over CDP and still navigates through the SvelteKit client router.

- **All 14 nav entries land on a real page** (General, Updates, Editor, Terminal, IDEs,
  Languages, Package Managers, Frameworks, GitHub, Theme, Learning, Providers, Training
  Data, Autonomy) — none 404, both by direct navigation and by clicking the nav button.
- **Active state is correct and singular:** exactly one nav button carries the `bg-accent`
  class per route (`/settings/general` → General, `/settings/languages` → Languages,
  `/settings/ai/training` → Training Data, `/settings/ides` → IDEs).
- **Group labels are not clickable:** "FRAMEWORKS & LANGUAGES" and "AI" are plain `<div>`s
  with no `button` / `a` / `role="button"` ancestor.
- **Retired paths redirect, none 404:** `/settings` → `/settings/general`,
  `/settings/frameworks-languages` → `/settings/languages`, `?tab=languages` →
  `/settings/languages`, `?tab=package-managers` → `/settings/package-managers`,
  `?tab=frameworks` → `/settings/frameworks`, `/settings/ai` → `/settings/ai/providers`,
  `/settings/framework-ides` → `/settings/ides`.
- **Every "Configure providers" entry point reaches the providers page:** LearningSettings'
  "Go to AI Providers" (the legacy `goto('/settings/ai')`, arrives via the 307), the
  `/utilities/disk` Rules tab link (`href=/settings/ai/providers`) and the coder sidebar
  footer "AI provider settings" icon button all end on `/settings/ai/providers` with h2
  "AI Provider Configuration". The task generator (`AIGenerationDialog.svelte`) and AI chat
  (`ChatCatalogSelectors.svelte`) use the same `AI_PROVIDER_SETTINGS_PATH` constant, which
  now points at the real page.
- **No double titles:** `/settings/ides` shows "IDE Configuration" once (h2 list is
  `["IDE Configuration", "Framework IDE Mappings"]`), `/settings/ai/providers` shows
  "AI Provider Configuration" once, and the other twelve have no `h2` at all — they are
  identified by the breadcrumb (`Home > Settings > Languages`, `… > AI > Providers`,
  `… > AI > Training`, all correct) plus the highlighted nav entry. The only `h1` is the
  layout's "Settings".
- **Collapsed rail:** forced `sidebar:domain:state=false` (its trigger is `md:hidden`, so
  the rail is otherwise unreachable above 768px). The rail renders **11 icon buttons for 11
  sections**, and the group icons navigate: index 5 (Frameworks & Languages) →
  `/settings/languages`, index 9 (AI) → `/settings/ai/providers`. Cookie restored
  afterwards. The rail buttons still have no accessible name (`tooltipContent` only) —
  identical to the pre-change markup, so not a regression from this fix.
- Console errors: none across all 14 routes and all 7 redirects.

Evidence: `.claude/qa/shots/ux/UX-015-after-settings-general.png`,
`.claude/qa/shots/ux/UX-015-after-settings-languages.png`,
`.claude/qa/shots/ux/UX-015-after-settings-rail-icons.png`.
