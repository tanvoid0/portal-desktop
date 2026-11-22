---
id: QA-180
title: "Utilities disk tiles: duplicated label, truncated drive name, percentage text wraps mid-phrase"
status: verified
severity: polish
kind: ux
area: utilities/disk
mode: tauri
found: 2026-11-22
---

## Use case

Read how full each drive is.

## Steps

1. /utilities/disk at 1280x800 or 1024x600.

## Expected

`C:\` once; `I:\ Samsung SSD 990 PRO` readable; `45% of 2.6 TB` on one line.

## Actual

`C:\ C:\` (drive letter repeated as the label), `I:\ Samsung SSD 990 ...` truncated, `65% of 977` / `GB` splits across two lines. The page header here (icon, big title, long subtitle) also differs from every other page's PageHeader.

## Evidence

`.claude/qa/shots/QA-180-before-disk-tiles.png`.

## Repro

manual - steps above

## Fix

Root cause: DashboardTab printed mountPoint twice when the drive has no name, truncated the mount, and the 'N% of X' span could wrap. File: src/lib/domains/disk/components/DashboardTab.svelte (hide name when equal to mount, mount shrink-0, nowrap on percentage, used/free truncates instead). Header differing from PageHeader (DiskUtilityPage) left as is: design call. Checks: pnpm check, prettier, eslint. Not visually rerun.

## Verification

2026-11-22, tauri. C:\ shown once, '45% of 2.6 TB' on one line (QA-180-after-utilities-disk.png). Trade-off: 'N used - M free' now truncates ('1.2 TB used - 1.4 ...') and 'I:\ Samsung SSD 990 ...' still truncates.
