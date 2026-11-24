---
id: QA-163
title: "Devices: pairing capabilities are raw ids (n8n.run, agents.history) and the red Sensitive badge explains nothing"
status: verified
severity: minor
kind: copy
area: devices
mode: tauri
found: 2026-11-22
---

## Use case

Choose what a phone may do before pairing.

## Steps

1. Open /devices.

## Expected

Each permission has a human label or hint; sensitive ones say what they allow.

## Actual

17 toggles labelled with code ids (coder.exec, github.write, terminal.exec; remote.input on a device row). Six carry a red Sensitive pill but no tooltip or description of the risk. The block sits above the paired-devices list, so only two devices are visible at 1280x800 and none at 1024x600. Device rows mix absolute ("Expired 13/11/2026") and relative ("Expired 1d ago") dates, and expired rows say "no capabilities", which reads like a bug rather than an expired grant. The "Advanced" switch top-right is unexplained, and "Takes effect after restarting" has no restart action. Lens: impeccable clarify, apple-design.

## Evidence

`.claude/qa/shots/QA-163-before-raw-capability-ids.png`

## Repro

manual - steps above

## Fix

Root cause: registry ids had no human copy, and several presentation details were left as product calls. Now: (1) `devices/utils/capabilityLabels.ts` maps ids to label + one-line hint (fallback = raw id); CapabilityChecklist shows label, hint, Sensitive badge, raw id in title. (2) `devices/utils/expiry.ts` `relativeExpiry` ("in 3d" / "3d ago"); DeviceRow expiry label and extend toast use it, absolute date in title. (3) PairDeviceCard (pairing + checklist) moved below Paired devices in routes/devices/+page.svelte; empty-state copy says "below". (4) LanToggleCard shows "Restart now" after toggling, via existing `relaunch()` from @tauri-apps/plugin-process (already used by updateService); no new Rust. Earlier hints (Sensitive tooltip, Advanced title, 'No permissions granted') kept. Checks: pnpm check 0 errors, prettier + eslint on devices (0 errors). Visual CDP check not completed (script hung); not verified in the running app. Earlier status was already verified for the first round; re-verify.

## Verification

<!-- qa-tester -->

2026-11-22 (round 2), tauri. Round 2: Paired devices list is above the Pair a phone card; pairing card shows labels with hints ('Quick actions - Run your saved actions', 'Run agents' Sensitive); 1d expiry reads relative ('Expired 1d ago') while older ones show the date (QA-163-after2-devices.png). Notes: paired-device chips still show raw ids (agents.exec, remote.input). The 'Restart now' button after the LAN toggle NOT verified (needs toggling the real LAN setting).
