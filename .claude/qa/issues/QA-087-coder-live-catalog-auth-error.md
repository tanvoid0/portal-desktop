---
id: QA-087
title: Coder page logs "Failed to load live catalog: Invalid or missing agent-platform API token"
status: open
severity: major
kind: bug
area: coder / agent-platform auth
mode: tauri
found: 2026-11-06
---

## Steps

1. AI -> Code (`/ai/coder`) with `agent-platformd` running on `http://127.0.0.1:18410`.
2. Read the webview console.

## Actual

```
Failed to load live catalog: Authentication error: Invalid or missing agent-platform API token
```

The daemon's `/health` reports `auth: { required: true, mode: "bearer" }`, and Portal's
AgentPlatform provider row has no `api_key`. The coder's own `/api/v1/coder/chat/stream`
turns work regardless, so whatever the "live catalog" feeds is the only thing degraded.

## Expected

Either the catalog request carries the same credential the chat stream uses, or the failure
is reported in the UI rather than only as a console error — right now a user cannot tell
that anything is missing.

## Why this is major, not cosmetic

The live catalog is what populates the coder's **model** dropdown
(`ProviderModelSelector`, rendered from `CoderPanel.svelte:471` with `showModel={!isCli}`).
With the catalog request failing, the selector shows `Loading...` forever and **the model
cannot be changed from the UI at all.**

The coder's model comes from `localStorage` key `portal.ai.chatCatalog`
(`chatCatalogPrefs.ts:1`, read by `coderSession.initProviderDefaults`), not from the
AgentPlatform provider row — Settings -> AI -> "Default model alias" said `gemma4:latest`
while every coder thread was created with `gemma4:e4b`. So the only way to switch model
today is to edit that localStorage value by hand. Found while measuring model accuracy for
Part D of the test plan; switching from `gemma4:e4b` to `qwen3:8b` had to be done that way.

## Notes

Not caused by the tool-argument repair (`2ede7cf`). Two separable problems: the catalog
request needs the credential the chat stream already uses, and the Settings "Default model
alias" field does not drive the coder.
