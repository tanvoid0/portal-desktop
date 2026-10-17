# Local Docker Deploy — Design

Status: **draft / phase 1 in progress**
Audience: Portal Desktop contributors
Related code: `src-tauri/portal-core/src/domains/deployments/`, `src/routes/deployments/`

---

## 1. Goal

Deploy a project **on this PC** with one action and reach it at a real URL
(`http://my-app.localhost`) instead of hunting for a free host port. Inspired by
[oblien/openship](https://github.com/oblien/openship)'s self-hosted mode, minus
the cloud parts.

What we copy from openship, and what we drop:

| openship stage                                                | we keep                               | we drop (cloud-only)      |
| ------------------------------------------------------------- | ------------------------------------- | ------------------------- |
| **Detect** stack from project files, `openship.json` override | ✅ `portal.yaml` override             | —                         |
| **Build** a Docker image, snapshot config                     | ✅ build + release snapshot           | —                         |
| **Run** on loopback, never a public port                      | ✅ run on a private Docker network    | public exposure           |
| **Route + TLS** via an edge (OpenResty + Let's Encrypt)       | ✅ local reverse proxy by hostname    | Let's Encrypt, public DNS |
| **Redeploy** on GitHub webhook push                           | ⏳ later: local git-watch / one-click | public webhook endpoint   |

Dropped outright: remote-server-over-SSH, multi-tenant login, managed cloud.

The reframe: openship is **not** cloud-only — its desktop/self-hosted mode
already deploys locally via Docker. We are adopting that mode's shape, not
fighting its design.

---

## 2. Where we start

Already in the tree (`deployments` domain):

- `DockerService` — shells out to `docker` (build / run / stop / rm / logs /
  stats / images / status). Keep it; it is fine for a desktop app.
- `DeploymentService` — sqlite-backed (sea-orm) orchestration, Docker + CLI types.
- `generate_dockerfile` — 4 hardcoded templates (node/rust/python/go).
- Deployments persist as one JSON blob in `deployments.data_json`, so **adding
  fields to the `Deployment` struct needs no migration** — only `#[serde(default)]`.
- `projects/discovery/scan.rs` — stack markers (`MARKERS`, `stack_for`) we reuse.
- An axum WS daemon + xterm (for log streaming later).

Two real bugs to fix on the way through (see §7).

---

## 3. The `portal.yaml` contract

A single optional file committed to the **target app's** repo. Absent → Portal
infers everything (§4). Present → it overrides detection, field by field. This is
our `openship.json`.

```yaml
# portal.yaml — how Portal builds and runs this app locally.
# Every field is optional; Portal fills the gaps by detection.

name: my-app # image + hostname base. Default: project dir name.
stack:
  node # node|python|go|rust|java|php|ruby|docker|compose
  # Default: auto-detected from manifests.

build:
  dockerfile: ./Dockerfile # Used as-is if it exists. Else Portal generates one.
  context: . # Build context, relative to the project dir.
  args: # --build-arg
    NODE_ENV: production

run:
  port:
    3000 # The port the app LISTENS on INSIDE the container.
    # This is the field that fixes the hardcoded-3000 bug.
  command: npm start # Optional; overrides the image CMD.
  env:
    DATABASE_URL: postgres://localhost/app
  volumes:
    - ./data:/app/data # host:container. Host side is relative to project dir.

host:
  my-app.localhost # The URL Portal routes to this app.
  # Default: <name>.localhost
```

Resolution order for every value: **`portal.yaml` → detection → built-in default.**

Secrets do **not** go in `portal.yaml` (it is committed). They stay in the OS
keychain (`keyring`) and are injected as env at run time, same as today.

---

## 4. Detection (the Detect stage)

New focused module `deployments/services/detect.rs`. Input: a project path.
Output:

```rust
pub struct DetectedApp {
    pub stack: String,            // canonical: "node" | "python" | ...
    pub container_port: u16,      // the LISTEN port (see table)
    pub dockerfile: Option<String>, // existing Dockerfile path, if any
    pub is_compose: bool,         // docker-compose.yml present
}
```

Logic (all cheap file reads, no DB — mirrors `scan.rs::stack_for`):

1. If `portal.yaml` exists, load it; its fields win.
2. Stack ← first matching manifest (`package.json`→node, `go.mod`→go, …).
   Reuse the marker list from `scan.rs`.
3. Container port, in order:
   - `portal.yaml run.port`, else
   - `EXPOSE <n>` parsed from an existing Dockerfile, else
   - stack default from the table below.

| stack      | manifest                              | base image (generated Dockerfile)                                                                                              | default port             |
| ---------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------ |
| node       | `package.json`                        | `node:lts`                                                                                                                     | 3000                     |
| **nextjs** | `package.json` with a `next` dep      | `node:22-alpine`, **build-aware** (`install → build → start`), using the project's package manager (pnpm/yarn/npm by lockfile) | 3000                     |
| python     | `requirements.txt` / `pyproject.toml` | `python:3.12`                                                                                                                  | 8000                     |
| go         | `go.mod`                              | `golang:1-alpine` (multi-stage)                                                                                                | 8080                     |
| rust       | `Cargo.toml`                          | `rust:1` (multi-stage)                                                                                                         | 8080                     |
| java       | `pom.xml` / `build.gradle`            | `eclipse-temurin:21`                                                                                                           | 8080                     |
| php        | `composer.json`                       | `php:8-apache`                                                                                                                 | 80                       |
| ruby       | `Gemfile`                             | `ruby:3`                                                                                                                       | 3000                     |
| docker     | `Dockerfile`                          | use it verbatim                                                                                                                | from `EXPOSE`, else 8080 |

`generate_dockerfile` is extended to take the detected port (so `EXPOSE`
matches reality) and to cover the new stacks. Existing templates stay.

**Framework detection.** A plain manifest is not always enough: a Node project
that depends on `next` needs a `next build` before `next start`, so the
generic `npm start` template would fail. Detection reads `package.json` and
narrows `node` → `nextjs`, which gets a build-aware, package-manager-aware
Dockerfile. This is the extension point for other frameworks (Vite, Nuxt, …)
as they come up.

---

## 5. Routing

> **Reality check (verified 2026-10-10).** The Traefik label-discovery approach
> below **does not work on Docker Desktop for Windows**: mounting the Docker
> socket into the Traefik container yields an unusable API (a container's own
> `curl --unix-socket` → `HTTP 000`; Traefik's provider → `400`), so no routes
> register and every request 404s. Therefore:
>
> - **Published host port is the default, reliable routing** — every deploy
>   publishes a free host port and gets `http://localhost:<port>`. No socket, no
>   proxy, works everywhere.
> - **Traefik `*.localhost` routing is an opt-in bonus**, attached only when a
>   health-check confirms the proxy's docker provider is actually up
>   (`proxy_provider_healthy`). On Windows it stays off and the deploy logs say
>   so, instead of handing back a dead URL.
> - To use hostname routing on Windows later: point Traefik at the Docker
>   Desktop TCP endpoint (`tcp://host.docker.internal:2375`, with "Expose daemon
>   on tcp://localhost:2375" enabled) or run a socket-proxy sidecar.

openship fronts apps with an edge so you get a URL, not a port. Where the
provider works (Linux hosts), we do the same with **Traefik** as a long-lived
container that auto-discovers app containers by **Docker labels** — no config
files, no routing code of our own.

Why Traefik over a hand-rolled proxy or Caddy:

- **Label discovery**: Traefik's Docker provider watches the daemon and wires
  routes from container labels. We emit labels at `docker run`; nothing else.
- **No cert-store friction**: default is `http://<name>.localhost`, which every
  browser resolves to loopback with zero hosts-file edits and zero TLS trust
  prompts. (Caddy's auto-HTTPS is nice but needs its root CA installed into the
  OS trust store — friction we don't want by default. HTTPS is a later opt-in.)

### Mechanism

1. **Bootstrap** (once, lazily, when the first Docker deploy starts): ensure a
   `portal` Docker network and a `portal-proxy` Traefik container exist.

   ```
   docker network create portal        # idempotent; ignore "already exists"
   docker run -d --name portal-proxy --network portal \
     -p 80:80 \
     -v /var/run/docker.sock:/var/run/docker.sock:ro \
     traefik:v3 \
     --providers.docker=true \
     --providers.docker.exposedbydefault=false \
     --entrypoints.web.address=:80
   ```

   On Windows the socket mount is `//./pipe/docker_engine://./pipe/docker_engine`.

2. **Per app**: `run_container` joins the `portal` network and emits labels:

   ```
   --network portal
   --label traefik.enable=true
   --label traefik.http.routers.<name>.rule=Host(`<host>`)
   --label traefik.http.services.<name>.loadbalancer.server.port=<container_port>
   ```

   No host-port publish needed — Traefik reaches the app over the shared
   network on its real listen port. (We can still publish a port for apps the
   user wants to hit directly; optional.)

### Smart port handling (implemented)

Nothing is published onto a host port that is already taken:

- **Proxy**: `ensure_proxy` prefers `:80` but probes first; if `:80` is busy it
  binds the next free port and returns it, so the app URL becomes
  `http://<host>:<port>`.
- **Per app**: a host port is published **only** when the user sets
  `exposed_port` (direct access). That port is probed with
  `find_free_port(requested)`; if busy, the next free port is used and the
  deployment records the port actually published. With no `exposed_port`, the
  app is reachable only through the proxy, so there is no collision to resolve.

Probe is a `std::net::TcpListener` bind on `0.0.0.0` that is dropped
immediately — no new dependency. There is a small TOCTOU window between the
probe and `docker run` publishing the port; acceptable for a single-user
desktop, and `docker run` still errors loudly if it loses the race.

3. The deployment stores `host`; the UI shows it as a clickable link and we
   open it with `tauri-plugin-opener` after a successful start.

This is the Docker-local twin of the `/cloud/ingress` concept already built for
Kubernetes — same mental model, different backend.

---

## 6. Releases and rollback (phase 2)

openship snapshots config per build so rollback is repeatable. Today a
`Deployment` is mutable and reuses one image name — no history.

Add to the `Deployment` struct (serializes into `data_json`, no migration):

```rust
#[serde(default)] pub container_port: Option<u16>,
#[serde(default)] pub host: Option<String>,
#[serde(default)] pub releases: Vec<Release>,
#[serde(default)] pub active_release: Option<String>,

pub struct Release {
    pub id: String,            // also the image tag suffix
    pub image_tag: String,     // my-app:<id> — immutable, never reused
    pub container_port: u16,
    pub env: HashMap<String,String>,
    pub created_at: String,
}
```

- Each build tags `my-app:<release-id>` (timestamp/uuid), **not** a mutable tag.
- Deploy = build a release, point `active_release` at it, run that image.
- **Rollback** = set `active_release` to a prior release and run its (still
  present) image. No rebuild.

---

## 7. Bugs to fix along the way

In `start_deployment` (`deployment_service.rs`):

```rust
let ports = if let Some(port) = deployment.exposed_port {
    vec![(port, 3000)]          // BUG: container port hardcoded to 3000
} else { vec![(3000, 3000)] };
let volumes = vec![(deployment.project_id.clone(), "/app".to_string())];
                                 // BUG: host path is a UUID, not a directory
```

- **Hardcoded container port 3000** contradicts our own generated Dockerfiles
  (`EXPOSE 8080`/`8000`). Fixed by §4's detected `container_port`.
- **Volume host path is `project_id`** (a UUID string), bind-mounting garbage
  over `/app` and clobbering the code the image already `COPY`'d. Drop the
  implicit volume; take volumes only from `portal.yaml run.volumes`.

---

## 8. Monitoring (phase 3)

Reuse what exists: `list_container_stats` already returns CPU/mem/net/io;
`get_container_logs` does a one-shot tail. For live view:

- Stream `docker logs -f <id>` to a WS channel on the existing axum daemon →
  render in xterm.
- Replace the polling `get_container_status` loop with `docker events` (push).

No new subsystem — just wiring.

---

## 9. Phasing

- **Phase 1 (now):** `portal.yaml` parse · `detect.rs` (stack + port) ·
  `generate_dockerfile` uses detected port + new stacks · **fix both §7 bugs** ·
  Traefik bootstrap + labels + `portal` network · `host` field + open URL.
  → first real "deploy on my PC and click the link" path.
- **Phase 2:** immutable releases + rollback (§6).
- **Phase 3:** live logs over WS + `docker events` status (§8).
- **Phase 4 (YAGNI-gated, build at first real need):** compose mode (repo has
  `docker-compose.yml` → `docker compose up -d`, group by compose label the UI
  already reads); push-to-deploy via local git-watch or a one-click redeploy.

### Hardening applied after review

- Restart works: `start` re-`docker start`s an existing container instead of a
  fresh `docker run` (which collided on the container name after a stop).
- `ensure_proxy` recovers a wedged proxy: if the stopped proxy can't restart
  (its old port got taken), it is removed and recreated on a free port.
- Proxy is non-fatal for direct-port deploys: if Traefik can't come up, a
  deployment with an `exposed_port` still runs (direct URL), logging a warning;
  only hostname-only deploys hard-fail.
- Publish retry: a `docker run -p` refused by Docker (Windows reserved range or
  a lost race) advances to the next free port and retries, bounded.
- The reachable URL (with any non-80 proxy port) is persisted on the deployment
  and is what the UI links to.

### Known limitation (deferred)

Two running deployments that resolve to the **same** `<name>.localhost` produce
two Traefik routers with an identical `Host` rule; Traefik then routes by
router-name order, so the second deploy may silently be shadowed by the first.
Fix when multiple concurrent deployments per project name become real: give
each a unique host (short suffix) or stop the prior one first. Single
deployment per name — the common case — is unaffected.

### Explicitly deferred / won't

- **`bollard`** (Docker API client) instead of shelling `docker` — correct
  long-term engine, but a rewrite of working code. Add when text parsing bites.
- **MCP endpoint / SDK** — off-path for local deploy; the AI Coder already exists.
- **Remote SSH deploy, Let's Encrypt, multi-tenant** — cloud concerns, out of scope.

---

## 10. Open questions

- HTTPS locally: ship `http://*.localhost` only, or offer a Caddy/`mkcert`
  opt-in later for apps that need secure-context browser APIs?
- The `DeploymentCard` URL link assumes the proxy is on `:80`. When the proxy
  falls back to another port, the authoritative URL (with port) is in the
  deployment logs ("Available at …"); fold the real port into the card link
  in a follow-up (needs the proxy port surfaced to the frontend).
- Windows Docker socket in the Traefik mount: we use
  `/var/run/docker.sock` (works with Docker Desktop in Linux-container mode on
  all three OSes). Windows-containers mode would need the named pipe — out of
  scope for now.
