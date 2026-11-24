---
id: QA-164
title: "SDK Manager sidebar uses the Home icon for Node/Python/Java/Rust/Go and shows \"Loading\" three times"
status: verified
severity: polish
kind: ux
area: sdk
mode: tauri
found: 2026-11-22
---

## Steps

1. Open /sdk.

## Actual

Every language entry in the app sidebar has the house icon, identical to Overview, so they read as home links. While loading the page shows "Loading SDKs..." in the rail, "Loading SDK data..." in the body and em-dash stats, plus a doubled title (SDK Manager / SDK Management). Lens: make-interfaces-feel-better (icon meaning), emil-design-eng.

## Evidence

`.claude/qa/shots/QA-164-before-sdk-home-icons.png`

## Repro

manual - steps above

## Fix

Sidebar submenu icons in main-nav.ts ('node-js','python','coffee','rust','go') matched no case in nav-icon.svelte so fell through to the Home icon. Now devicon-* names, and nav-icon renders Devicon for them. SDKSidebar footer no longer repeats 'Loading SDKs...' (rail body + page body keep one each). Files: config/main-nav.ts, components/shell/nav-icon.svelte, sdk/components/SDKSidebar.svelte. Not changed: doubled title SDK Manager / SDK Management (copy call). Checks: pnpm check (0 errors), eslint+prettier on touched files.

Second pass: doubled title was src/routes/sdk/+page.svelte PageHeader 'SDK Management' vs nav/sidebar 'SDK Manager'; now 'SDK Manager' (matches nav). Checks: pnpm check 0 errors, eslint+prettier on touched file.

## Verification

<!-- qa-tester -->

2026-11-22 (round 2), tauri. Round 2: page title reads 'SDK Manager' (QA-164-after2-sdk.png).
