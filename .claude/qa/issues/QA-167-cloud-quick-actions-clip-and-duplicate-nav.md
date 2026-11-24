---
id: QA-167
title: "Cloud overview: \"View X\" quick-action buttons duplicate the left rail and clip their text"
status: verified
severity: polish
kind: ux
area: cloud
mode: tauri
found: 2026-11-22
---

## Steps

1. Open /cloud at 1280x800.

## Actual

Ten "View Pods/Services/..." buttons repeat the rail beside them one-for-one. "View StatefulSets", "View DaemonSets" and "View ConfigMaps" run to the button edges with no padding. Emoji icons sit beside a different icon style elsewhere. The console logs "Metrics API not available" with no on-screen message. Lens: impeccable critique (redundancy), make-interfaces-feel-better.

## Evidence

`.claude/qa/shots/QA-167-before-cloud-quick-actions.png`

## Repro

manual - steps above

## Fix

Buttons had no horizontal padding and a 4-col grid too narrow for long labels. Added px-4 and lg:grid-cols-3 2xl:grid-cols-4. File: routes/cloud/workloads/+page.svelte. Left as product call: removing the buttons as rail duplicates, emoji icons, on-screen 'Metrics API not available' message. Checks: pnpm check (0 errors), eslint+prettier on touched files.

Second pass: removed the whole Quick Actions card (10 'View X' buttons duplicated the rail and the stats grid carried no unique info; emoji icons go with it). Metrics failure now shown on-screen as 'Resource usage unavailable: <error>' (metricsError state) instead of console-only. File: routes/cloud/workloads/+page.svelte. Checks: pnpm check 0 errors, eslint+prettier. Not verified live in CDP.

## Verification

<!-- qa-tester -->

2026-11-22 (round 2), tauri. Round 2: Quick Actions card gone from /cloud/workloads; on-screen 'Resource usage unavailable' shown when the metrics call fails (QA-167-after2-cloud-workloads.png).
