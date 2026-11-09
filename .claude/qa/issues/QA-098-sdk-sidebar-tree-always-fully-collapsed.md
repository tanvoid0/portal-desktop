---
id: QA-098
title: SDK sidebar tree opens fully collapsed, and the selected SDK's category is not expanded
status: verified
severity: major
kind: bug
area: sdk (SDKSidebar)
mode: tauri
found: 2026-11-07
---

## Use case

Open an SDK page and see where you are in the SDK tree; or land on `/sdk` and pick an SDK from the
sidebar without having to guess which category it is in.

## Steps

1. Load `/sdk/python` directly. Wait for the sidebar to finish ("11 of 16 SDKs" in the footer).
2. Read the state of the six category sections.
3. Repeat on `/sdk`.

## Expected

- The selected SDK's category (Language & Runtime for python) is open with Python highlighted —
  `SDKSidebar.svelte:264-275` says "Keep the currently selected SDK's category expanded".
- With nothing selected, `SDKSidebar.svelte:277-288` says it should open `language` and `manager`
  by default so the tree "shows something".

## Actual

Every category is collapsed, on both routes, after a full load:

```
direct load /sdk/python after 12s:
 {"sections":["SDK Managers=false","Language & Runtime=false","Database Server=false",
              "Web Server=false","Container Platform=false","AI SDKs=false"],
  "pythonRowVisible":false,"loading":false}
on /sdk:
 {"sections":[… all =false …],"pythonRowVisible":false,"loading":false}
```

So no SDK is visible anywhere in the tree until the user expands a category by hand, and the
highlight for the SDK they are actually looking at is hidden inside a closed section. Expanding by
hand does work (`Language & Runtime` → aria-expanded=true, Python/Node.js/Java/Rust/Go appear —
`.claude/qa/shots/sdk-sidebar-category-expanded.png`), and the four Navigation items highlight
correctly on `/sdk`, `/sdk/managers`, `/sdk/managers/nvm`, `/sdk/services`, `/sdk/versions`.

Likely cause: both `$effect`s read `categories`, which is empty while the SDK queries are still
pending. The first one sets `lastSelectedItemId = selectedItem` on that first empty pass
([SDKSidebar.svelte:275](../../../src/lib/domains/sdk/components/SDKSidebar.svelte#L275)) and then
short-circuits on every later run (`if (selectedItem === lastSelectedItemId) return`), so it never
runs again once the data arrives; the default-open effect is skipped whenever `selectedItem` is
truthy.

## Evidence

`.claude/qa/shots/sdk-sidebar-python-direct-load.png`,
`.claude/qa/shots/sdk-sidebar-python-client-nav.png`,
`.claude/qa/shots/sdk-overview-loaded-1280.png` (all six chevrons closed),
`.claude/qa/shots/sdk-sidebar-category-expanded.png` (after a manual click).

## Repro

`node .claude/qa/scripts/sdk-sidebar5.mjs`

## Fix

Two distinct bugs, both in scope:

1. The "keep selected category expanded" effect matched the exact mechanism this issue
   already named: it committed `lastSelectedItemId = selectedItem` on the first (empty
   `categories`) pass even when no category matched, short-circuiting the next run once real
   data arrived. Fixed by only committing `lastSelectedItemId` once a category actually
   matched (`src/lib/domains/sdk/components/SDKSidebar.svelte`).
2. On `/sdk` itself (and `/sdk/managers`, `/sdk/services`, `/sdk/versions`), `+layout.svelte`
   computed `selectedSDK` as the last path segment — for the bare nav routes that's the literal
   word `"sdk"`/`"managers"`/etc., not `undefined`. That truthy-but-matching-nothing value made
   `selectedItem` truthy, which blocked the *other* effect ("nothing selected → open `language`
   and `manager` by default") from ever running on these routes. Fixed by excluding the known
   nav-route keywords when deriving `selectedSDK` (`src/routes/sdk/+layout.svelte`).

Checks: `pnpm check && pnpm lint` (0 errors). Not re-verified live (see QA-097's note on why);
qa-tester should rerun `node .claude/qa/scripts/sdk-sidebar5.mjs`.

## Verification

2026-11-09, mode: tauri. `node .claude/qa/scripts/QA-098.mjs` — direct load of `/sdk/python`:
Python's sidebar row highlighted and visible without any manual expand
(`.claude/qa/shots/QA-098-direct-load-sdk-python.png` — Node.js/Python/Java/Rust/Go all shown
under the "SDK Manager" group, Python selected). Direct load of `/sdk` (nothing selected):
"SDK Managers" and "Language & Runtime" both expanded by default with full contents (Conda,
Fast Node Manager, NVM, Pyenv, Rustup / Python, Node.js, Java, Rust, Go), Database Server /
Web Server / Container Platform / AI SDKs correctly stay collapsed
(`.claude/qa/shots/QA-098-direct-load-sdk-overview.png`). Both bugs named in the Fix are
confirmed fixed, independent of QA-097 (the main content pane in the same screenshot is still
stuck on "Loading SDK data..." — that's QA-097, a different code path, reopened separately).
Verified.
