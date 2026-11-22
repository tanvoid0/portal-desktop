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

Capability ids are the backend registry names with no descriptions (no label/hint field in CapabilityInfo). Added title hints: Sensitive badge explains the risk, Advanced switch explains itself; expired/empty rows read 'No permissions granted' instead of 'no capabilities'. Files: devices/components/CapabilityChecklist.svelte, DeviceRow.svelte, routes/devices/+page.svelte. Left as product calls: human labels per capability (needs registry descriptions in Rust), moving the checklist below the device list, unifying absolute/relative dates, a restart action for 'Takes effect after restarting' (LanToggleCard). Checks: pnpm check (0 errors), eslint+prettier on touched files.

## Verification

<!-- qa-tester -->
