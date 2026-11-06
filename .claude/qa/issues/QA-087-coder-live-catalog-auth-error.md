---
id: QA-087
title: Coder page logs "Failed to load live catalog: Invalid or missing agent-platform API token"
status: fixed
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


## Verified working — 2026-11-06

The model dropdown populates and switches. Driven over CDP on the running app:

```
OPTIONS: Your models
  gemma4:e4b Q4_K_M ollama 8.0B gemma4 LIVE 8.95 GB
  nomic-embed-text:latest F16 ollama 137M nomic-bert LIVE 261.6 MB
  qwen2.5-coder:7b Q4_K_M ollama 7.6B qwen2 LIVE 4.36 GB
  qwen3:8b Q4_K_M ollama 8.2B qwen3 LIVE 4.87 GB
pref before: {"backendProvider":"ollama","model":"qwen2.5-coder:7b"}
pref after:  {"backendProvider":"ollama","model":"gemma4:e4b"}
```

No catalog error in the console on load.

## What was actually changed, and what was not

**Changed:** `agent_platform_provider.rs` no longer throws away the daemon's 401 body.
Both 401 sites collapsed every rejection into "Invalid or missing agent-platform API
token", while every *other* status branch in the same function already included the body.
That one line is why this issue, and an earlier session, both went looking at the token.
`auth_failure_detail` now reports which check refused: `AUTH_REQUIRED` (no header),
`TOKEN_INVALID` / `TOKEN_EXPIRED` (bad secret), `APP_ID_REQUIRED` / `APP_ID_UNKNOWN` (token
fine, no AI entitlement), with the raw body as fallback. Two unit tests cover it.

**Not proven: what restored the auth.** The dropdown was dead when this was filed and works
now, but the code change only affects the error *message*. The live settings file
(`%APPDATA%\portal_desktopi-settings.json`) now holds a 64-character `api_key`; the most
likely explanation is that it was empty before — which matches the daemon's own
`AUTH_REQUIRED` meaning of "the header was missing" — and that saving the provider config
from Settings -> AI (done while switching models for the Part D measurement) populated it.
The before-state was overwritten before anyone thought to record it. If this recurs, the
new message will name the real code instead of guessing.

## Two traps found on the way

- **A stale duplicate settings file.** `%APPDATA%\portal-desktopi-settings.json`
  (hyphen) exists alongside `%APPDATA%\portal_desktopi-settings.json` (underscore), from
  an older app identifier, with a *different* `api_key` and a different `model`
  (`gemma4:latest` vs the live value). Only the underscore one is read —
  `app_paths::config_dir()` is the single source, used by both the catalog command and the
  coder's `PlatformConfig::resolve`. Reading the wrong one makes the config look wrong when
  it is not.
- **The Settings "Default model alias" field genuinely does not drive the coder, and that is
  by design.** The coder's model is the per-user `portal.ai.chatCatalog` localStorage pref;
  the provider row's `model` is only the fallback when no pref exists
  (`coderSession.initProviderDefaults`, `platform_config.rs` `default_model`). Not a bug.
  The bug was that the pref could not be changed, because the only writer of it
  (`ChatCatalogSelectors.persistSelection`) runs on a *successful* catalog load.
