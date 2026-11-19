# Agent Platform → Portal Desktop migration plan

Status: **plan / proposed.** Implements the decision in agent-platform
[ADR 0020](https://github.com/tanvoid0/agent-platform/) (_Operator UI moves to
portal_desktop; server runs as its sidecar_). This is the portal-side counterpart
ADR 0020 asked for. Nothing here is built yet.

---

## 1. The question, answered first

**How much migrates: all the operator _UI_, none of the _brain_.**

Draw one line and the rest follows:

- **`agent-platformd` stays the brain.** It is the product — an OpenAI-compatible
  proxy + multi-agent orchestration API on `:18410`, headless, shipped in
  Docker/cloud and as a bare local provider. Every bit of _compute_ lives here:
  provider routing, BYOK, the model catalog, model-ops (build/train), run/process
  orchestration, teams, workflows, memory, media generation. **It never moves into
  portal, and portal never re-implements any of it.** That is exactly your "avoid
  duplicating AI functionality across apps" rule: the logic lives once, behind an
  HTTP contract, and _N_ apps call it.

- **Portal owns the operator UI.** The rich screens that today live in
  agent-platform's `desktop/crates/app` (iced) are rebuilt in portal's
  SvelteKit + shadcn against the existing `/api/v1` contract. The iced app retires.

- **agent-platform keeps only a thin status/config surface.** Enough to prove the
  server is alive, show the port/key, and point at Docker/cloud. Not a second
  operator UI — the "config built twice" is the thing this kills.

So "a little UI to configure / check status" = what _stays_ in agent-platform.
The full operator experience = what _moves_ to portal. The AI brain other apps
consume = `agent-platformd`, untouched.

```
        other apps ─┐
   portal_desktop ──┼──HTTP :18410──▶  agent-platformd   (the brain; one copy)
     (future app) ──┘                  ├ LLM proxy + BYOK
                                       ├ orchestration / runs / teams / workflows
                                       ├ model catalog + model-ops
                                       └ memory / studio / media
   portal owns the operator UI + spawns agent-platformd as its sidecar
   agent-platform keeps only: headless build + a thin status page
```

---

## 2. Where we already are (more done than it looks)

Portal is **already** a deep HTTP client of `:18410`. This is not a greenfield port.

Already in portal:

- **Core client**: `portal-core/src/domains/ai/` — `agent_platform_provider.rs`,
  `platform_admin.rs` (health/start/attach), `platform_config.rs`,
  `platform_project.rs`, `catalog.rs`, `context_usage.rs`, `ai_service.rs`,
  `ai_settings_service.rs`, `chat_title.rs`.
- **Frontend AI domain**: `src/lib/domains/ai/` — provider settings,
  `PlatformHealthBar`, `PlatformStatusBanner`, `PlatformOfflineBanner`, catalog,
  context usage, chat services + state (`aiPlatform.svelte.ts`).
- **Routes already present**: `/ai/{agents, chat, coder, history, logs, providers,
training}`.
- **Coder** domain with `platform_stream.rs` (SSE/ws from the platform).

So "chat, tasks, documents, Coder, disk analysis" already run through the platform
(per portal's own README). The migration is **the screens portal doesn't have yet**,
plus **flipping how the server is owned**.

### The one architectural reversal

`platform_admin.rs` today encodes the **pre-0020** stance, verbatim in its doc comment:

> Portal must never spawn `agent-platformd` itself. Starting it means starting the
> **Agent Platform app**, which owns the `agent-platformd` child — the same way
> Portal starts Docker Desktop and not `dockerd`.

ADR 0020 reverses this: **portal owns `agent-platformd` directly as a Tauri
`externalBin` sidecar** (spawn on launch, kill on close, attach-if-running), and
the iced app — the thing portal currently launches — is retired. There is no
`externalBin`/sidecar config in portal yet; this is net-new work (Phase 1).

---

## 3. What stays in `agent-platformd` (never migrates)

Keep. Portal calls it; portal does not own it.

| Stays server-side                                | Why                                                |
| ------------------------------------------------ | -------------------------------------------------- |
| LLM proxy `/v1/*`, BYOK                          | The whole point: one credential path, many clients |
| Orchestration: runs, processes, task DAG         | Core engine; portal renders it                     |
| Teams (rosters), Workflows, Plans                | Domain logic + persistence                         |
| Model catalog, provider capability normalization | One source of truth for every client               |
| Model-ops (build/train) — and `worker/` training | Heavy, out-of-bundle by design (ADR 0004/0020)     |
| Memory, Studio/media generation, Search          | Server features; portal is a viewport              |
| Accounts / billing (`/accounts`)                 | Hosted, not a desktop concern                      |

Also **not** merged (ADR 0020 "out of scope"): **agent-monitor** — it watches
Claude Code, is its own Node+SQLite+React app, and its independence is a feature.
May later dock into portal as a webview section; code stays its own.

---

## 4. What migrates to portal (the operator UI)

The iced app grew well past ADR 0020's "providers, model-ops, teams, runs." Full
screen inventory from `desktop/crates/app/src/screen.rs`:

Home, Workspace (Chat/Board/Docs/Code tabs), Assistant (E.V. voice + HUD + wake
word), Studio (media/ads), Processes, Coder, Projects, Teams, Workflows,
Plans/Todos, Agenda, Search, Downloads/model-download, Memory, and Settings
(Providers, Model-ops, Appearance, Performance, Status, API docs, Account) + Logs.

Mapped against what portal already has:

| iced screen                                           | Portal status                        | Action                                    |
| ----------------------------------------------------- | ------------------------------------ | ----------------------------------------- |
| Providers                                             | **have** (`/ai/providers`)           | polish to parity                          |
| Chat / Assistant (text)                               | **have** (`/ai/chat`, `/ai/history`) | parity                                    |
| Coder                                                 | **have** (`/ai/coder`)               | parity                                    |
| Logs / Status                                         | **partial** (health banners)         | add a Status/Logs surface                 |
| Model-ops                                             | gap                                  | port — the build/train console            |
| Processes / runs (plan graph, scoped chat, event log) | gap (`/ai/agents` may seed it)       | port — highest-value screen               |
| Teams (rosters + templates)                           | gap                                  | port                                      |
| Workflows                                             | gap                                  | port                                      |
| Plans / Todos, Agenda                                 | gap                                  | port (lower priority)                     |
| Studio (media/ad generation)                          | gap                                  | port (own phase; heavy)                   |
| Search, Downloads, Memory                             | gap                                  | port (low priority)                       |
| Assistant **voice** (E.V., wake word, HUD, STT/TTS)   | gap                                  | **decide** — see open questions           |
| Performance (resource modes, ADR 0010)                | gap                                  | port — small, useful                      |
| Account                                               | gap                                  | hosted `/accounts` webview, don't rebuild |

Rule for every port: **rebuild the view, reuse the contract.** Each screen is
Svelte + shadcn calling `/api/v1` (+ `/openapi.json` for types, via
`pnpm gen:types`-style generation). No server logic crosses the line.

---

## 5. Phases

Each phase ends on portal's gate: **`pnpm check && pnpm lint`** (+ `cargo test`
where Rust changed). Ship behind the existing platform-health gating so a
half-ported screen degrades to "server starting", never a blank.

### Phase 0 — Accept the direction (no code)

- Land this doc as portal's ADR 0020 counterpart.
- Update agent-platform README portfolio line + portal README "Required: run in a
  second terminal" (it stops being true after Phase 1).

### Phase 1 — Sidecar ownership flip _(the keystone; do first)_

- Bundle `agent-platformd` as a Tauri **`externalBin`** per target
  (`tauri.conf.json` `bundle.externalBin` + build step that drops the platform
  binary in).
- Rewrite `platform_admin.rs`: spawn/kill the **daemon** directly (not the iced
  app), keep **attach-if-running** and the **foreign-server** guard (a Docker
  port-forward on 18410 pointing at another DB must be reported, not adopted).
  Honor the per-install 256-bit key.
- Version pin: portal tags carry the `agent-platformd` version they bundle.
- **Gate:** launch portal with no second terminal; health goes green; killing
  portal reaps the child.
- After this, the "Docker Desktop analogy" in `platform_admin.rs` is wrong — the
  binary is now owned like `dockerd`, not like Docker Desktop.

#### Release/CI wiring (how the daemon actually ships)

The daemon is pulled from agent-platform's own cargo-dist releases, not rebuilt
here. `scripts/fetch-agent-platformd.mjs` downloads the pinned tag's
`agent-platform-server-<triple>.tar.xz|.zip`, verifies its `.sha256`, and drops
`agent-platformd-<triple>[.exe]` into `src-tauri/binaries/` (gitignored) where
Tauri's `externalBin` expects it.

- **Pinned tag:** `v0.7.6` (the constant in the fetch script). Bump it in lockstep
  with a deliberate daemon upgrade; portal tags carry the daemon version they ship.
- **Kept out of `tauri.conf.json`:** `bundle.externalBin` lives in
  `src-tauri/tauri.sidecar.conf.json`, merged only at build via
  `--config src-tauri/tauri.sidecar.conf.json`. A missing externalBin file fails
  `cargo check` / `tauri:dev`, so dev and `ci.yml` (which only `cargo check`s the
  base config) never need a fetched binary. Release CI
  (`.github/actions/build-release`) fetches per matrix target, then passes the
  `--config` to `tauri-action`. `ci.yml` and `release-smoke` are unchanged.
- **macOS arm64:** agent-platform ships **no `aarch64-apple-darwin` daemon**
  (GitHub pulled that repo's macOS-ARM runner; its `dist-workspace.toml` says
  Apple Silicon runs the x86_64 daemon under Rosetta 2). Portal's release matrix
  still builds a native arm64 **app**, but for that job the fetch script maps
  `aarch64-apple-darwin → x86_64-apple-darwin` and bundles the x86_64 daemon under
  the arm64 filename. **Ceiling:** the daemon runs under Rosetta 2 on Apple
  Silicon (present by default once any x86 binary has run). Revisit if
  agent-platform restores a native arm64 daemon asset — then the mapping in
  `DAEMON_TARGET` just drops away.

### Phase 2 — Settings parity (Providers, Model-ops, Status, Performance, API)

- Providers to full parity; add Model-ops console, a real Status/Logs page, the
  Performance/resource-mode knob (ADR 0010), and an API-docs/key panel.
- These are the "configure + check status" screens — the ones that justify the
  whole move for day-one users.

### Phase 3 — Runs & crew (Processes, Teams, Workflows)

- Processes/runs screen: plan-as-graph, the run-scoped chat, the event log, JSON
  export. This is the operator UI's center of gravity — highest value.
- Teams (rosters + templates), Workflows.

### Phase 4 — Retire iced

- Once Phases 2–3 cover the screens actually used, cut
  `desktop/crates/app` (iced), `desktop/crates/client`, `desktop/installer/` in
  agent-platform. **Keep** `desktop/crates/server` (→ `agent-platformd`),
  `desktop/crates/local-llm`, `worker/`.
- agent-platform's desktop shrinks to: headless build + a thin status page.
- **Do not cut before parity-on-used-screens** — iced and Svelte coexist only
  during the transition (the one doubling this whole effort removes).

### Phase 5 — The long tail (optional / own cadence)

- Studio (media/ads), Plans/Agenda, Search, Downloads, Memory.
- Assistant **voice** — only if the decision below says yes.

---

## 6. Open questions (your calls)

1. **Assistant voice (E.V.).** Wake word, HUD, STT/TTS, "ask from any screen" is a
   large, distinctive chunk of the iced app. Port it into portal, drop it, or keep
   it as a separate small companion that also talks to `:18410`? Recommendation:
   **defer** (Phase 5) — it is the least "operator UI", most "consumer app".
2. **Does portal become the only desktop face, or one of several?** Your framing
   ("other apps use the brain based on need") implies several. If so, nothing
   portal-specific should leak into the server during the port — keep the seam at
   `/api/v1`, resist adding portal-only endpoints.
3. **Headless-ahead vs pinned-sidecar drift.** Portal ships a pinned
   `agent-platformd`; the headless binary can run ahead for other API clients.
   Confirm the `/api/v1` versioning story so a newer headless server and an older
   bundled one don't surprise a shared `:18410`.
4. **Thin status page tech.** Does agent-platform's leftover status surface stay a
   tiny served HTML page, or lean on the existing `/accounts` stack? (Keep it
   boring — it exists only for "is it up".)
5. **Cross-repo release fetch.** Release CI downloads the daemon from the **public**
   `tanvoid0/agent-platform` repo using the workflow's own `GITHUB_TOKEN` (a public
   read needs no cross-repo scope). If agent-platform ever goes private, that token
   can't reach it — the fetch step would need a PAT secret with read access. Flag
   if visibility changes.

---

## 7. One-line summary

Move **every operator screen** into portal and let portal **own the daemon as a
sidecar**; keep **all AI compute** in `agent-platformd` behind `/api/v1` so portal
— and the next app — stay thin clients of one brain. Start with the sidecar flip
(Phase 1); it is what makes "no second terminal" true and everything after it a
view-only port.
