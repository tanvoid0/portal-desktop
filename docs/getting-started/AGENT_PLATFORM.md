# Agent Platform (required)

Portal Desktop routes **all AI features** through [agent-platform](https://github.com/tanvoid0/agent-platform/) — chat, tasks, documents, disk analysis, and the Coder agent. The desktop app is a client; you must run agent-platform separately.

## Architecture

```
┌─────────────────────┐         HTTP / SSE          ┌──────────────────────────┐
│  portal-desktop     │  ─────────────────────────► │  agent-platform          │
│  (Tauri + Svelte)   │   /v1/catalog, /v1/chat   │  https://github.com/     │
│                     │   /api/v1/coder/chat/…    │  tanvoid0/agent-platform   │
└─────────────────────┘                             └──────────────────────────┘
                                                              │
                                                              ▼
                                                    Ollama, Gemini, LM Studio, …
                                                    (configured at /config)
```

Portal Desktop does **not** embed LLM providers. It connects to agent-platform for:

- Provider/model catalog (`GET /v1/catalog`)
- AI chat and streaming
- Coder agent turns (`POST /api/v1/coder/chat/stream`)
- Smart titles, commit messages, and other background LLM calls

## Quick start

### 1. Clone and run agent-platform

```bash
git clone https://github.com/tanvoid0/agent-platform.git
cd agent-platform
cp .env.example .env
pnpm install
pnpm dev:server
```

Default API: `http://127.0.0.1:18410`  
OpenAPI spec: `http://127.0.0.1:18410/openapi.json`

There is no browser UI on the platform any more — the `/config`, `/tokens` and
`/docs` pages went with the Python server. Configuration is the agent-platform
desktop app (`pnpm dev` in that repo) or the REST routes below.

See the [agent-platform README](https://github.com/tanvoid0/agent-platform#quick-start) for Docker and workspace token setup.

### 2. Configure providers on the platform

Either open the agent-platform desktop app and use its **Providers** screen, or
drive the same routes directly:

1. `GET /api/v1/llm-proxy/env` / `POST /api/v1/llm-proxy/env` — API keys and base URLs
2. `GET /api/v1/llm-proxy/config-yaml` / `POST /api/v1/llm-proxy/config-yaml` — provider and alias definitions
3. `GET /api/v1/llm-proxy/ui/providers` — what is configured and reachable
4. If authentication is enabled, mint a workspace token with the master key:

```bash
curl -X POST http://127.0.0.1:18410/api/v1/workspaces/1/api-tokens/ \
  -H "Authorization: Bearer $AGENT_PLATFORM_MASTER_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "portal-desktop"}'
```

The response carries the `agp_…` token once; it is not retrievable afterwards.

### 3. Connect Portal Desktop

1. Start Portal Desktop (`pnpm tauri:dev` or install from [GitHub Releases](https://github.com/tanvoid0/portal-desktop/releases))
2. Go to **AI → Providers**
3. Set:
   - **Base URL** — `http://127.0.0.1:18410` (default)
   - **API token** — workspace token from agent-platform (if required)
   - **Default model** — alias from the catalog (use **Load catalog**)
4. Click **Test connection**

### 4. Optional: web search for the Coder

The Coder's `web_search` tool runs through the platform's `GET /api/v1/search`,
so Portal needs no search provider of its own. The route always answers — with
`configured: false` and no results — until the platform has a key:

```bash
SEARCH_API_KEY=<google-custom-search-key> SEARCH_CX=<engine-id> agent-platformd
```

Without one the tool still appears and still answers, telling the model that
search is switched off rather than that the web had nothing. That distinction is
deliberate: an empty result list would read as "this does not exist".

## Environment variables (optional)

For development or CI when AI settings are not saved yet:

| Variable                                               | Default                  | Purpose               |
| ------------------------------------------------------ | ------------------------ | --------------------- |
| `AGENT_PLATFORM_BASE_URL` or `CODER_PLATFORM_BASE_URL` | `http://127.0.0.1:18410` | Platform API base URL |
| `CODER_MODEL`                                          | —                        | Default model alias   |

API tokens are **not** read from env vars — set them in **AI → Providers** in the app.

## Troubleshooting

| Symptom                            | Fix                                                                                                        |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| "Can't reach agent-platform"       | Start agent-platform (`pnpm dev:server` or Docker). Confirm `http://127.0.0.1:18410/health` returns `200`. |
| "Invalid catalog response"         | Update agent-platform to the latest version; catalog format is defined in its `/v1/catalog` API.           |
| "Invalid or missing API token"     | Add a workspace token in AI → Providers, or disable auth on the platform for local dev.                    |
| Coder agent fails                  | Ensure agent-platform exposes `/api/v1/coder/chat/stream` and your model supports tools if required.       |
| `web_search` says "not configured" | Set `SEARCH_API_KEY` and `SEARCH_CX` on the platform, not in Portal.                                       |

## Links

- **Repository:** https://github.com/tanvoid0/agent-platform
- **Client integration:** [docs/CLIENT_INTEGRATION.md](https://github.com/tanvoid0/agent-platform/blob/main/docs/CLIENT_INTEGRATION.md) (workspace tokens)
- **Portal releases:** https://github.com/tanvoid0/portal-desktop/releases
