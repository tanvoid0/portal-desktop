---
id: QA-165
title: "GitHub page: Repositories and Issues cards are links but look like static info; scope chips are raw OAuth scopes"
status: verified
severity: polish
kind: ux
area: github
mode: tauri
found: 2026-11-22
---

## Steps

1. Open /github while connected.

## Actual

Account, Repositories and Issues render as same-looking cards. Clicking Repositories goes to /github/repos but no arrow, button or hover cue says so. The Account card shows raw scopes "read:user" and "repo". Disconnect (top right) does not say what it removes. Lens: apple-design (affordance), impeccable clarify.

## Evidence

`.claude/qa/shots/QA-165-before-github-cards.png`

## Repro

manual - steps above

## Fix

Cards were plain Cards with onclick. Added hover background and an 'Open repositories/issues' arrow line; scopes show friendly label (raw scope in title); Disconnect has a title saying it removes the stored token only. File: routes/github/+page.svelte. Cards still not keyboard-focusable (pre-existing). Checks: pnpm check (0 errors), eslint+prettier on touched files.

## Verification

<!-- qa-tester -->
