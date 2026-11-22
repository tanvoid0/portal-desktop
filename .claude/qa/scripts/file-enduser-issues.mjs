// One-shot: files the end-user sweep issues QA-160..167 and copies their before-shots.
import { copyFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const Q = fileURLToPath(new URL('../', import.meta.url));
const I = [
  [160, 'vault-unlock-dialog-opens-cold-with-no-reason', 'Vault unlock dialog appears on app load over unrelated pages and never says why', 'minor', 'ux', 'vault/VaultGate (all routes)', 'sweep-credentials-dark-1280', 'vault-prompt-no-reason',
`## Use case

Open Portal and start working (projects, tasks, devices, settings).

## Steps

1. Cold-load any route (/projects, /tasks, /settings, /devices, /github) with the vault locked.
2. Dismiss with Not now, then navigate to GitHub.

## Expected

If the app must ask for the master password, the dialog says what needs it. Pages that do not need secrets do not prompt.

## Actual

A modal "Vault locked / Enter your master password" covers Projects, Tasks, Settings and Devices with no stated reason, so it reads like a random lock screen. After Not now it returns on /github. On /credentials the same state also shows inline ("Vault is locked - Unlock") so that page asks twice. The body copy says you will be asked again, but not by what. Lens: apple-design / make-interfaces-feel-better (explain why a permission is requested, at the moment of need).`],
  [161, 'coder-header-eats-viewport-at-1024x600', 'Coder page at 1024x600: header stack takes ~320px of 600 and pushes the chat down (QA-023 regressed)', 'major', 'ux', 'ai/coder', 'sweep-coder-light-1024x600', 'coder-header-1024x600',
`## Use case

Use the AI coder in a small window.

## Steps

1. Open /ai/coder at 1024x600.

## Expected

Chat and composer own most of the window.

## Actual

Home/Code tabs, engine tabs (Portal, Claude Code, Codex, Cursor, Overview), Agent Platform + provider + model selects, panel icons, git delta and Commit each wrap onto their own row. "What should we build?" is half hidden above the composer; the right panel and "New session" row add more chrome. At 1280x800 the same controls form two nav rows (Home/Code and Portal/Claude Code/...) so it is unclear which tab set is primary; "Git changes +1268 -1352" appears three times (header, panel, composer chip) and the panel label truncates to "Git chan...". Lens: impeccable critique (hierarchy), emil-design-eng (one control, one place). Also see sweep-coder-light-1280.png.`],
  [162, 'credentials-duplicate-cards-and-fake-masked-value', 'Credentials: two identical "GitHub (tanvoid0)" cards, fake "encr****" value, card actions clip at the edge', 'minor', 'ux', 'credentials', 'sweep-credentials-1280', 'duplicate-credential-cards',
`## Use case

Find which stored credential is which.

## Steps

1. Open /credentials.

## Expected

Entries are distinguishable; locked values show an obvious placeholder.

## Actual

Two cards share name, type and description; only "Last used" differs (on one only). The Value box reads "encr****", which looks like real content rather than a locked placeholder. The eye/pencil/trash icons sit against the card's right edge (trash touches it); delete is adjacent to edit with no labels, and the user cannot tell which card to delete. Lens: apple-design (clarity), make-interfaces-feel-better (alignment).`],
  [163, 'devices-capabilities-raw-ids-no-explanation', 'Devices: pairing capabilities are raw ids (n8n.run, agents.history) and the red Sensitive badge explains nothing', 'minor', 'copy', 'devices', 'sweep-devices-after-dismiss-dark-1280', 'raw-capability-ids',
`## Use case

Choose what a phone may do before pairing.

## Steps

1. Open /devices.

## Expected

Each permission has a human label or hint; sensitive ones say what they allow.

## Actual

17 toggles labelled with code ids (coder.exec, github.write, terminal.exec; remote.input on a device row). Six carry a red Sensitive pill but no tooltip or description of the risk. The block sits above the paired-devices list, so only two devices are visible at 1280x800 and none at 1024x600. Device rows mix absolute ("Expired 13/11/2026") and relative ("Expired 1d ago") dates, and expired rows say "no capabilities", which reads like a bug rather than an expired grant. The "Advanced" switch top-right is unexplained, and "Takes effect after restarting" has no restart action. Lens: impeccable clarify, apple-design.`],
  [164, 'sdk-sidebar-all-home-icons-triple-loading', 'SDK Manager sidebar uses the Home icon for Node/Python/Java/Rust/Go and shows "Loading" three times', 'polish', 'ux', 'sdk', 'sweep-sdk-1280', 'sdk-home-icons',
`## Steps

1. Open /sdk.

## Actual

Every language entry in the app sidebar has the house icon, identical to Overview, so they read as home links. While loading the page shows "Loading SDKs..." in the rail, "Loading SDK data..." in the body and em-dash stats, plus a doubled title (SDK Manager / SDK Management). Lens: make-interfaces-feel-better (icon meaning), emil-design-eng.`],
  [165, 'github-cards-clickable-without-affordance', 'GitHub page: Repositories and Issues cards are links but look like static info; scope chips are raw OAuth scopes', 'polish', 'ux', 'github', 'sweep-github-dark-1280', 'github-cards',
`## Steps

1. Open /github while connected.

## Actual

Account, Repositories and Issues render as same-looking cards. Clicking Repositories goes to /github/repos but no arrow, button or hover cue says so. The Account card shows raw scopes "read:user" and "repo". Disconnect (top right) does not say what it removes. Lens: apple-design (affordance), impeccable clarify.`],
  [166, 'ai-chat-delete-all-icon-next-to-new', 'AI chat: "Delete all conversations" is a lone trash icon beside "+", and most chats are titled "Hello"', 'minor', 'ux', 'ai', 'sweep-ai-1280', 'ai-chat-delete-all',
`## Steps

1. Open /ai (Home tab).

## Actual

The Chats header has three icon-only buttons: new folder, new conversation, trash. The trash's only label is a tooltip ("Delete all conversations"), where users expect delete-selected. I did not click it (real data), so whether it confirms is untested. Five of eight chats are titled "Hello", distinguished only by count or date. Agent Platform selects read "Loading..." at first paint. Lens: apple-design (destructive actions separated and labelled), emil-design-eng.`],
  [167, 'cloud-quick-actions-clip-and-duplicate-nav', 'Cloud overview: "View X" quick-action buttons duplicate the left rail and clip their text', 'polish', 'ux', 'cloud', 'sweep-cloud-1280', 'cloud-quick-actions',
`## Steps

1. Open /cloud at 1280x800.

## Actual

Ten "View Pods/Services/..." buttons repeat the rail beside them one-for-one. "View StatefulSets", "View DaemonSets" and "View ConfigMaps" run to the button edges with no padding. Emoji icons sit beside a different icon style elsewhere. The console logs "Metrics API not available" with no on-screen message. Lens: impeccable critique (redundancy), make-interfaces-feel-better.`],
];
for (const [id, slug, title, sev, kind, area, src, name, body] of I) {
  const shot = `QA-${id}-before-${name}.png`;
  copyFileSync(`${Q}shots/${src}.png`, `${Q}shots/${shot}`);
  writeFileSync(`${Q}issues/QA-${id}-${slug}.md`, `---
id: QA-${id}
title: ${JSON.stringify(title)}
status: open
severity: ${sev}
kind: ${kind}
area: ${area}
mode: tauri
found: 2026-11-22
---

${body}

## Evidence

\`.claude/qa/shots/${shot}\`

## Repro

manual - steps above

## Fix

<!-- coder: root cause, files changed, checks run -->

## Verification

<!-- qa-tester -->
`);
}
