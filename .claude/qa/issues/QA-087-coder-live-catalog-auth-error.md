---
id: QA-087
title: Coder page logs "Failed to load live catalog: Invalid or missing agent-platform API token"
status: open
severity: minor
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

## Notes

Found while verifying the tool-argument repair (not caused by it). Unknown what the live
catalog drives in the UI; worth identifying before deciding whether this is cosmetic.
