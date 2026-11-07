---
id: UX-019
title: The Settings button in the /sdk page header does nothing
status: verified
severity: minor
kind: bug
area: sdk
mode: tauri
found: 2026-11-07
---

## Use case

From SDK Management, open the SDK settings — the primary (filled) button in the page header
is the most prominent control on the page.

## Steps

1. Open `/sdk`.
2. Wait for the stat strip to fill (Managers 3/5 · Services 3/5 · SDKs 8/11 · Status Online).
3. Click `Settings` — the blue primary button at the top right of the page header (not the
   gear icon at the bottom of the app sidebar, which does work).

## Expected

Something happens: navigation to an SDK settings page, or a settings dialog, or at minimum a
toast. A primary-variant button is the strongest affordance on the page.

## Actual

Nothing. After the click: `location.pathname` is still `/sdk`, no `[role="dialog"]` is in the
DOM, no toast, no console error. The button has no handler at all —
[src/routes/sdk/+page.svelte:267](../../../src/routes/sdk/+page.svelte#L267):

```svelte
<Button size="sm">
  <Settings class="mr-2 h-4 w-4" />
  Settings
</Button>
```

## Not a regression

Pre-existing. The same `<Button>` with no `onclick` is in the pre-UX-003 revision
(`git show HEAD:src/routes/sdk/+page.svelte`, the hand-rolled header); UX-003 only moved it
into `PageHeader`'s `actions` snippet and added `size="sm"`. Found while re-testing UX-003's
"the old buttons still work" claim.

## Expected fix

Either wire it (`goto('/settings/...')` or the SDK settings dialog) or delete it. If there is
no SDK settings destination yet, deleting it is the smaller diff — the gear in the app sidebar
already reaches `/settings/general`.

## Evidence

`.claude/qa/shots/ux/UX-019-before-sdk-settings-dead.png` (taken right after the click —
page unchanged, no dialog).

## Repro

Manual — steps above. Programmatic check:

```js
// node, with .claude/qa/scripts/cdp.mjs
const app = await connect();
await app.nav('/sdk', 3000);
await app.clickButton('Settings'); // scoped to <main>, asserts tagName === 'BUTTON'
await app.wait(2000);
console.log(await app.eval(() => ({ path: location.pathname, dialogs: document.querySelectorAll('[role="dialog"]').length })));
// -> { path: '/sdk', dialogs: 0 }
```

## Fix

<!-- coder: root cause, files changed, checks run, other QA ids this also fixes -->

## Verification

2026-11-07 — tauri, dev instance 2. Enumerated every button in the `/sdk` page shell:

```
Toggle Sidebar, Home, SDK Manager, Toggle theme, Toggle Sidebar,
SDK Manager / Manage your development environment, Overview, SDK Managers, Services, Versions,
Refresh
```

The PageHeader's only action is **Refresh**, which works (it invalidates the `sdk` query key).
There is no unwired primary button in that header. The dead `Settings` button was removed upstream
in commit d0fccf8, not by this wave — closing on that basis.

Shot: `.claude/qa/shots/UX-014-after-sdk-overview-loaded.png`.
