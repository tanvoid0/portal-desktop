# Production Readiness Review — Portal Desktop

Date: 2026-07-22
Scope: Rust/Tauri backend, SvelteKit frontend, build/release/CI/testing.
Method: three parallel deep reviews (backend security, frontend, build/CI). Findings verified against source; `file:line` cited throughout.

This is a review doc only — no code changed. It ranks what stands between the current `v0.6.0` and a defensible production release.

---

## Verdict

Portal Desktop is a **feature-rich, well-organized app with a solid release pipeline** (semver sync, 4-platform matrix, minisign-signed auto-updater, composite CI actions). The blockers are **not architecture** — they are a handful of security holes and one piece of leftover debug code that exfiltrates data.

**Do not ship a public release until the P0 items below are fixed.** They are small diffs with large blast radius.

| Area | State |
|------|-------|
| Release pipeline / updater wiring | Good |
| App architecture / structure | Good |
| Security posture | **Blocking issues** |
| OS code signing | Missing |
| Test coverage | Thin |
| Repo hygiene | Minor cleanup |

---

## P0 — Blockers (fix before any release)

### 1. Leftover AI-debug telemetry exfiltrates cloud data
`src/lib/domains/cloud/stores/cloudStore.ts`, `providers/gcp/GCPProvider.ts`, `components/CloudConnectionGuard.svelte`
36 `fetch("http://127.0.0.1:7704/ingest/...")` calls inside `// #region agent log` blocks POST cluster IDs, provider types, and error messages on every cloud-connect attempt. Debug instrumentation that must never ship.
**Fix:** delete all `#region agent log` blocks. Add a lint/grep guard in CI so they cannot return.

### 2. Arbitrary shell command execution exposed to the webview
`src-tauri/src/domains/terminal/manager.rs:546` (`execute_command`), `coder/tools.rs:143`, `scripts/services/script_execution_service.rs`, `projects/commands.rs:259` (`execute_command_in_directory`)
Frontend-invokable commands pass unsanitized strings straight to `cmd /C <string>` / `sh -c <string>`, or run any binary in any directory. Full RCE surface reachable from the webview.
**Fix:** gate execution behind explicit per-invocation user confirmation and/or an allowlist; never pass raw strings to `-c`/`/C`. Constrain `execute_command_in_directory` to registered project dirs.

### 3. CSP defeats XSS containment
`src-tauri/tauri.conf.json:23`
`script-src 'unsafe-inline' 'unsafe-eval'` + `connect-src` allowing arbitrary `http:`/`https:`/`ws:`. Any injected script can reach the shell commands (P0 #2) and exfiltrate anywhere.
**Fix:** drop `unsafe-inline`/`unsafe-eval`; scope `connect-src` to the agent-platform origin (`http://127.0.0.1:18410`) + GitHub release host.

### 4. Credential master key is publicly derivable
`src-tauri/src/domains/credentials/services/credential_service.rs:249`
AES-256 master key = `SHA256("<well-known-appdata-path>-portal-credential-master-key-v1")`. Anyone with the DB file recomputes it and decrypts every stored credential. "Encrypted at rest" is effectively plaintext.
**Fix:** derive from a user password (PBKDF2 already present) or the OS keychain — not a public path.

### 5. Unsanitized markdown → XSS
`src/lib/components/ui/chat-markdown/ChatMarkdown.svelte:125`, `renderMarkdown.ts:151,179`
`{@html marked.parse(...)}` with no sanitizer (marked v5+ dropped its built-in). Raw HTML like `<img src=x onerror=...>` in AI/chat content executes. Link renderer also allows `javascript:`/`data:` hrefs.
**Fix:** pipe `marked.parse()` output through DOMPurify before `{@html}`; allowlist `http(s)`/`mailto` schemes in the link renderer.

---

## P1 — High (fix before calling it production-grade)

### 6. No OS-level code signing
`src-tauri/tauri.conf.json`, `.github/actions/build-release`
Updater minisign is configured, but there is **no Windows Authenticode** (`bundle.windows.certificateThumbprint`) and **no macOS signing/notarization** (`bundle.macOS.signingIdentity`, no `APPLE_*` secrets). Every install and auto-update hits SmartScreen/Gatekeeper warnings.
**Fix:** add Authenticode cert + Apple Developer ID signing/notarization secrets and bundle config. This is table stakes for distributing a desktop app.

### 7. CI quality gates are non-blocking
`.github/workflows/ci.yml:49,53,100`
`pnpm check` (types), `pnpm lint`, and `cargo clippy` all `continue-on-error: true`. Type/lint/clippy regressions merge into `main` freely.
**Fix:** clear existing debt, then remove `continue-on-error`; or freeze current violations to an allowlist so only *new* ones gate.

### 8. Single broad capability for ~400 commands
`src-tauri/capabilities/default.json:6`
One `default` capability on the `main` window covers all ~400 registered commands. No least-privilege scoping.
**Fix:** split capabilities per window/feature; expose the minimum command set each surface needs.

### 9. Near-zero automated test coverage
9 `*.test.ts` across 953 source files; 0 Svelte component tests (browser vitest project is configured but unused); Rust tests in ~22 of 365 files; 1 real e2e test.
**Fix:** treat current tests as a smoke floor. Add coverage on the paths that can lose or leak data: credential encryption, updater, command executor.

### 10. `derive_key` panics on zero iterations
`src-tauri/src/domains/credentials/services/encryption_service.rs:97`
`NonZeroU32::new(iterations).unwrap()` aborts the thread on `0`.
**Fix:** return `CredentialError` instead of `unwrap()`.

### 11. No global frontend error handling
No `+error.svelte` anywhere, no `hooks.client.ts` `handleError`, no `unhandledrejection` listener. Uncaught load/render errors and rejected promises blank the view or fail silently. The existing `error-boundary.svelte` only reacts to a `component-error` event that nothing dispatches — it catches nothing.
**Fix:** add a root `+error.svelte`, a `hooks.client.ts handleError`, and an `unhandledrejection` listener wired to the toast system. Remove or actually wire the fake error boundary.

---

## P2 — Medium

| # | Issue | Location | Fix |
|---|-------|----------|-----|
| 12 | Secret DB unprotected on Windows — `set_file_permissions` is a no-op on non-unix, so the SQLite DB (encrypted creds) gets default ACLs on the primary target OS | `src-tauri/src/database.rs:116` | Apply Windows ACL (icacls / `SetNamedSecurityInfo`) current-user-only |
| 13 | Command timeout defined but never enforced — hung child blocks indefinitely | `src-tauri/src/command_executor.rs:27,246` | Wrap in `tokio::time::timeout`, kill child on expiry |
| 14 | Legacy DB import from CWD-relative `./data/portal_desktop.db` — a planted DB can be adopted on first run | `src-tauri/src/database.rs:85` | Resolve from a fixed absolute path only, or drop the legacy migration |
| 15 | No `[profile.release]` hardening — ships debug symbols, no strip/lto | `src-tauri/Cargo.toml` | Add `strip=true`, `lto=true`, `panic="abort"`, `opt-level="s"` |
| 16 | 109 `unwrap()`/`expect()`/`panic!` across 32 backend files; a panic in a Tauri command aborts the task | `version_fetcher.rs` (14), `cli_service.rs` (12), `script_execution_service.rs` (9)… | Audit those reachable from user input / network responses; convert to `AppError` |
| 17 | Dead server auth gate — `hooks.server.ts` never runs under `adapter-static`/Tauri; false sense of security | `src/hooks.server.ts:8` | Delete; rely on client `DeviceAuthGuard` |
| 18 | Broken `ai:test` script points at missing `test-ollama-integration.js`; ollama scripts contradict the agent-platform architecture in README | `package.json:26-30` | Remove ollama scripts (`ai:test`, `ai:setup`, `ollama:*`) |
| 19 | README `npm run install` errors — the script is `run:install` | `README.md:40` | Fix docs to `npm run run:install` |
| 20 | Permissive CSP `connect-src` (duplicate of P0 #3 from the build angle) | `tauri.conf.json:19` | Scope to real hosts |

---

## P3 — Low / hygiene

- **Repo bloat tracked in git:** `appimagelauncher-*.rpm` (2.9 MB, already matches a `.gitignore` rule but predates it), a literal `~/` directory (`~/.sdkman-native/sdk`), empty `portal_desktop_linux_workspace`. → `git rm` all three; add `/~/` to `.gitignore`.
- **Dead build files:** `Dockerfile.macos` (self-documents "NOT functional"), `docker-compose.yml` `build-macos` service, host-locked `Dockerfile.windows`. GitHub Actions does the real builds. → delete, keep one doc note.
- **`ws` dependency unused** — no `from "ws"` import; all WebSocket usage is browser-native. → remove from `package.json`.
- **286 `console.*` across 95 files** despite a `logger.ts` — noisy/leaky. → route through `logger`, strip via `esbuild.drop`.
- **Silent auto-update** (`dialog: false`) — signature is the only gate; a compromised release auto-installs. → consider `dialog: true` + changelog prompt.
- **Frontend-controlled `workspace_root`** for file IO (`coder/tools.rs:303`) rejects `..` but accepts any absolute path. → constrain to registered project dirs.
- **`reqwest` pulls `blocking`** feature in an async app. → drop unless a sync path needs it.
- **`unstale docs`:** `PRODUCTION_READY.md` documents an `<ErrorBoundary>` that doesn't exist; `IMPROVEMENTS.md` lists the same unwrap/mutex debt still open. → reconcile.
- **Large files** (refactor candidates, not blockers): `projects/[id]/+page.svelte` (1525), `sdk/+page.svelte` (1500), `coderSession.svelte.ts` (1446), `IdeSettings.svelte` (1316).

---

## Suggested sequencing

**Sprint 1 — unblock release (all P0):** delete telemetry, gate/allowlist shell commands, tighten CSP, fix credential key derivation, add DOMPurify. Small diffs, days not weeks.

**Sprint 2 — production-grade (P1):** code signing (longest lead time — start cert procurement now), make CI gates blocking, capability scoping, error handling, encryption/updater tests, fix the `derive_key` panic.

**Sprint 3 — harden & clean (P2/P3):** Windows ACLs, command timeouts, release profile, unwrap audit, repo cleanup, dead-file removal.

**What NOT to do:** the `IMPROVEMENTS.md` plan proposes a persistent job queue, DeploymentService DB migration, and full Svelte 5 migration. None block production. Defer until the security and signing work lands.

---

*Note for maintainers: the P0 telemetry endpoint and the credential-key derivation are the two items to fix today — one leaks live data, the other makes credential encryption cosmetic. Everything else can be scheduled.*
