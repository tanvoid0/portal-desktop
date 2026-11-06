---
id: QA-006
title: App freezes (CPU spin, unbounded memory) when the OS switches light/dark while theme is "system"
status: verified
severity: blocker
kind: bug
area: shell/theme
mode: tauri
found: 2026-11-06
---

## Use case

Owner keeps the theme on "System" (the default) and the OS flips between light and dark —
by hand, or on a schedule (Windows auto dark mode, night light tools).

## Steps

1. Real app, theme = System, OS in dark mode (app renders dark).
2. Switch the OS to light mode (or emulate it: CDP `Emulation.setEmulatedMedia` with
   `prefers-color-scheme: light`).

## Expected

App switches to the light theme and stays responsive.

## Actual

The webview freezes immediately. The renderer process spins a full core and its memory grows
~40 MB/s (seen: 160 MB → 5.9 GB in about two minutes). No JS ever runs again — CDP
`Runtime.evaluate` and even `Debugger.pause` get no reply. Only killing the app recovers.

Also blocks all QA automation: Playwright's `connectOverCDP` sends exactly this media
emulation on attach (it defaults `prefers-color-scheme` to light), so `app.mjs` hung the
app on every run.

## Evidence

- Renderer samples after the flip: `5569 MB → 5763 → 5837 → 5917` at 4 s intervals, CPU +1 core.
- Same app with no CDP attach: steady ~160 MB, idle. Plain reload and raw `Page.navigate`: fine.
- No before-screenshot: the frozen page can't be captured (CDP screenshot needs the renderer).

Root cause (`src/lib/domains/shared/stores/themeStore.ts:175`): `handleSystemThemeChange`
calls `refreshSystemTheme()` inside a `themeStore.subscribe(...)` callback, before that
subscription is torn down. `refreshSystemTheme()` → `update()` always produces a new object,
so the store notifies the still-live callback again, which refreshes again — Svelte's
subscriber queue never drains.

## Repro

`node .claude/qa/scripts/QA-006.mjs` — flips the scheme over raw CDP, prints PASS/FAIL.
After a FAIL the app is frozen: restart it.

## Fix

Fixed by the lead (one line, and it blocked the whole sweep). Root cause as above.
`src/lib/domains/shared/stores/themeStore.ts`: the media-query `change` listener now calls
`themeStore.refreshSystemTheme()` directly. That method already returns early unless the
preference is `system`, so the subscribe-wrapper was redundant as well as the loop.
Grepped the other `subscribe(...)` callbacks in `src/`: all read-only, none write back to
their own store.

Checked 2026-11-06 in tauri mode: `QA-006.mjs` → PASS (light/dark/light all applied, page
answers); `app.mjs settings/theme` now attaches in tauri mode with no console errors
(`shots/QA-006-after-playwright-attach.png`).

## Verification

<!-- qa-tester: date, mode, what was rerun, result -->

2026-11-06, tauri mode. `node .claude/qa/scripts/QA-006.mjs` -> PASS (light/dark/light, page answers each time). Playwright attach no longer hangs the app (every sweep script this session attached cleanly). Header theme menu Dark -> Light -> System, reload under Dark, and `page.emulateMedia` dark/light under System all applied and the app stayed responsive. Theme restored to System. No regression found. Related but separate: QA-008.
