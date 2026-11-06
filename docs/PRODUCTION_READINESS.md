# Production Readiness Review — Portal Desktop

Date: 2026-07-22, status refreshed 2026-11-05
Scope: Rust/Tauri backend, SvelteKit frontend, build/release/CI/testing.
Method: three parallel deep reviews (backend security, frontend, build/CI). Findings verified against source; `file:line` cited throughout.

The original review ranked what stood between `v0.6.0` and a defensible production release. The status column below is current as of the `fix/p0-shell-exec-csp` branch; paths were updated for the `portal-core` split.

## Status (2026-11-05)

| #     | Item                                   | Status                                                                                                                                                                          |
| ----- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P0 1  | Debug telemetry exfiltrates cloud data | **Fixed** in `4377b38`, with a blocking CI grep guard                                                                                                                           |
| P0 2  | Shell execution exposed to the webview | **Fixed** on `fix/p0-shell-exec-csp` (see below); daemon exposure open, see note                                                                                                |
| P0 3  | CSP defeats XSS containment            | **Fixed**: `unsafe-eval` in `4377b38`; `unsafe-inline` and `connect-src` on this branch                                                                                         |
| P0 4  | Credential master key derivable        | **Fixed** in `4377b38`: random key in the OS keychain                                                                                                                           |
| P0 5  | Unsanitised markdown XSS               | **Fixed** in `4377b38`: DOMPurify plus a link-scheme allowlist                                                                                                                  |
| P1 6  | No OS code signing                     | Open                                                                                                                                                                            |
| P1 7  | CI gates non-blocking                  | Partly fixed: check and lint block, clippy still `continue-on-error`. Frontend checks were red on `main` from the SvelteKit 3 upgrade (2026-10-25); this branch makes them pass |
| P1 8  | One broad capability                   | Open, and lower value than it looked (see #8)                                                                                                                                   |
| P1 9  | Thin test coverage                     | Open; this branch adds tests for the Coder allowlist and the terminal shell scope                                                                                               |
| P1 10 | `derive_key` unwrap                    | Open                                                                                                                                                                            |
| P1 11 | No global frontend error handling      | Open                                                                                                                                                                            |

---

## Verdict

Portal Desktop is a **feature-rich, well-organized app with a solid release pipeline** (semver sync, 4-platform matrix, minisign-signed auto-updater, composite CI actions). The blockers are **not architecture** — they are a handful of security holes and one piece of leftover debug code that exfiltrates data.

**Do not ship a public release until the P0 items below are fixed.** They are small diffs with large blast radius.

With `fix/p0-shell-exec-csp` merged, all five P0s are closed. The one exec exposure left is the loopback daemon (see #2), and it needs the capability work already in progress.

| Area                              | State                                         |
| --------------------------------- | --------------------------------------------- |
| Release pipeline / updater wiring | Good                                          |
| App architecture / structure      | Good                                          |
| Security posture                  | P0s closed; daemon RPC exposure open (see #2) |
| OS code signing                   | Missing                                       |
| Test coverage                     | Thin                                          |
| Repo hygiene                      | Minor cleanup                                 |

---

## P0 — Blockers (fix before any release)

### 1. Leftover AI-debug telemetry exfiltrates cloud data. **Fixed in `4377b38`.**

`src/lib/domains/cloud/stores/cloudStore.ts`, `providers/gcp/GCPProvider.ts`, `components/CloudConnectionGuard.svelte`
36 `fetch("http://127.0.0.1:7704/ingest/...")` calls inside `// #region agent log` blocks POST cluster IDs, provider types, and error messages on every cloud-connect attempt. Debug instrumentation that must never ship.
**Fix:** delete all `#region agent log` blocks. Add a lint/grep guard in CI so they cannot return.
**Done:** blocks deleted; `.github/workflows/ci.yml` step "No debug telemetry" fails the build if the pattern returns.

### 2. Arbitrary shell command execution exposed to the webview. **Fixed on `fix/p0-shell-exec-csp`.**

Original finding: frontend-invokable commands passed unsanitised strings to `cmd /C` / `sh -c`, or ran any binary in any directory.

**What containment can and cannot do here.** The terminal is a feature: `create_terminal_process` plus `send_terminal_input` give the main webview a real shell by design, and pipelines, scripts and actions run user-authored commands. So no per-command gate can stop a _compromised_ webview from running a shell, and Tauri capability scoping cannot either (one window holds every page, see #8). The containment boundary is keeping script out of the webview: the CSP (#3) and the markdown sanitiser (#5). What this item can do is remove every exec entry the webview does not need, and scope the ones it does. Per command:

| Command (current path)                                                                                                                                          | Needed by the webview?                                                                                                           | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `execute_command_in_directory` (was `src-tauri/src/domains/projects/commands.rs:259`): any binary, any args, any directory                                      | No. Its one caller was a fallback for project build/test that the actions catalog already runs through `execute_script`.         | **Removed.** The project page now falls back to the terminal, as it already did for `start`.                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `execute_command` (was `src-tauri/src/domains/terminal/commands.rs:35` → `portal-core/src/domains/terminal/manager.rs:535`): `cmd /C` / `sh -c` of a raw string | No. `TerminalService.executeInContext` had no callers; the SDK page called it with the wrong argument shape, so it never worked. | **Removed**, with its frontend wrappers. The SDK page now installs managers through the existing `install_sdk_manager(manager_name)`, which resolves the install workflow from the backend's own manager config instead of running a `curl … \| bash` string sent by the page.                                                                                                                                                                                                                                                                       |
| `create_terminal_process` (`portal-core/src/domains/terminal/manager.rs:59`)                                                                                    | Yes: it is the terminal.                                                                                                         | **Scoped.** Known shells map to their canonical executable as before; a name it did not recognise used to be spawned verbatim (any program on disk). Now `allowed_shell` (`manager.rs:704`) accepts it only if the OS lists it as a shell: `/etc/shells` on Unix, `zsh`/`fish` by bare name on Windows.                                                                                                                                                                                                                                              |
| `execute_script` (`src-tauri/src/domains/scripts/commands.rs:43` → `portal-core/src/domains/scripts/services/script_execution_service.rs:97`)                   | Yes: actions, automation and the scripts page run user-authored commands through it.                                             | Unchanged, and now the only raw-shell entry besides the terminal. An allowlist cannot cover user-authored commands.                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Coder `run_command` (`portal-core/src/domains/coder/tools.rs:458`): not a webview command, runs model tool calls                                                | n/a                                                                                                                              | Already gated by mode plus per-call approval, but the "accept & remember" matcher defeated the gate: a remembered `git` approved `git log && curl … \| sh`, any argument anywhere matched, and a one-letter token matched every rule starting with that letter. **Fixed** in `permissions.rs:119` (`command_allowed_by`): an allow rule must be the command's leading words, and a command carrying `; & \| \` $ < >` or a newline always goes back to the user. Deny rules keep the loose match, so they still catch a command anywhere in a chain. |

**Still open, a sibling path.** `src-tauri/src/daemon.rs` serves every `#[rpc]` command, `execute_script` and the terminal included, at `POST http://127.0.0.1:18420/rpc/{command}` with no token. It binds loopback and refuses browser `Origin`s, so no web page and no other machine reaches it, but any local process does, including one running as a different user on a shared machine. The capability tiers and pairing on `feat/coder-claude-cli-engine` ("capability gate, pairing routes") are the fix; until they land, the exec commands should not be reachable without a capability, and the daemon must not bind beyond loopback.

### 3. CSP defeats XSS containment. **Fixed** (`unsafe-eval` in `4377b38`, the rest on `fix/p0-shell-exec-csp`).

`src-tauri/tauri.conf.json:23`
Was `script-src 'unsafe-inline' 'unsafe-eval'` plus a `connect-src` allowing `https:`, `wss:` and every localhost port.

Now:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data: https:; font-src 'self' data:;
connect-src 'self' ipc: http://ipc.localhost;
```

- **`script-src 'self'`.** SvelteKit's static build has one inline bootstrap script per HTML file. Tauri hashes it at build time and adds the hash, plus nonces for its own scripts, to `script-src` (`tauri-codegen` `inject_script_hashes`, `tauri` `manager::set_csp`). Once a hash or nonce is present, Chromium ignores `'unsafe-inline'`, so it was already inert in release builds; dropping it changes nothing that works and removes the fallback.
- **`connect-src` is IPC only.** The webview makes no network requests of its own. AI chat reaches agent-platform from Rust (`portal-core/src/domains/ai/providers/agent_platform_provider.rs`, `reqwest`), and the updater checks GitHub releases from Rust (`tauri-plugin-updater`). CSP governs neither, so neither origin needs to be in `connect-src`, and a user-configured agent-platform base URL works without one. `ipc: http://ipc.localhost` is Tauri's own IPC channel; without it Tauri silently falls back to the slower `postMessage` path, which is what the old policy was doing.
- **`style-src 'unsafe-inline'` stays.** Svelte writes inline styles and Tauri adds no style nonces for this build, so it is still needed.
- **`img-src https:` stays** for remote avatars and icons. With script blocked it is no longer an exfiltration channel on its own.
- **Dev builds.** On desktop `tauri dev` the webview loads the Vite server directly and Tauri applies no CSP (it only sets the header on embedded assets), so the policy is checked against a release-mode build, not `pnpm tauri:dev`.
- **Checked** on a `tauri build --debug` binary (embedded assets, so the CSP is live) over WebView2's DevTools protocol: no violations on Overview, Terminal, Coder or AI chat; an injected inline script, a `fetch` to `https://example.com` and a direct `fetch` to `127.0.0.1:18410` are all refused; IPC works; the terminal runs a oneshot and an interactive shell.

### 4. Credential master key is publicly derivable. **Fixed in `4377b38`.**

`src-tauri/portal-core/src/domains/credentials/services/credential_service.rs:240` (was `src-tauri/src/domains/credentials/services/credential_service.rs:249`)
AES-256 master key was `SHA256("<well-known-appdata-path>-portal-credential-master-key-v1")`. Anyone with the DB file could recompute it and decrypt every stored credential.
**Done:** the key is 32 random bytes stored in the OS keychain (Windows Credential Manager, macOS Keychain, Secret Service).

### 5. Unsanitized markdown → XSS. **Fixed in `4377b38`.**

`src/lib/components/ui/chat-markdown/ChatMarkdown.svelte`, `renderMarkdown.ts`
`{@html marked.parse(...)}` had no sanitiser, and the link renderer allowed `javascript:`/`data:` hrefs.
**Done:** output goes through DOMPurify before `{@html}`; links are limited to `http(s)` and `mailto`.

---

## P1 — High (fix before calling it production-grade)

### 6. No OS-level code signing. **Open.**

`src-tauri/tauri.conf.json`, `.github/actions/build-release`
Updater minisign is configured, but there is **no Windows Authenticode** (`bundle.windows.certificateThumbprint`) and **no macOS signing/notarization** (`bundle.macOS.signingIdentity`, no `APPLE_*` secrets). Every install and auto-update hits SmartScreen/Gatekeeper warnings.
**Fix:** add Authenticode cert + Apple Developer ID signing/notarization secrets and bundle config. This is table stakes for distributing a desktop app.

### 7. CI quality gates are non-blocking. **Partly fixed.**

`.github/workflows/ci.yml`
`pnpm check` (types), `pnpm lint`, and `cargo clippy` all `continue-on-error: true`. Type/lint/clippy regressions merge into `main` freely.
**Fix:** clear existing debt, then remove `continue-on-error`; or freeze current violations to an allowlist so only _new_ ones gate.
**Now:** check and lint block; clippy (`ci.yml:141`) still does not. The SvelteKit 3 upgrade (2026-10-25) left the frontend gates red on `main`: `tsconfig.json` still extended `.svelte-kit/tsconfig.json`, which SvelteKit 3 no longer writes, so a fresh clone could not type-check or even `vite build` (local clones only worked off a stale copy). `eslint.config.js` imported the deleted `svelte.config.js`, and the vitest browser project passed `playwright` uncalled. `fix/p0-shell-exec-csp` fixes all three and the 12 `page.params` type errors they had been hiding, so `pnpm check`, `pnpm lint` and `pnpm vitest run` pass from a clean clone. `kit.alias` is deprecated in favour of subpath imports; moving `$lib`/`@` to `package.json#imports` is the follow-up.

### 8. Single broad capability for ~400 commands. **Open, lower value than it looked.**

`src-tauri/capabilities/default.json:6`
One `default` capability on the `main` window covers all ~400 registered commands. No least-privilege scoping.
**Fix:** split capabilities per window/feature; expose the minimum command set each surface needs.
**Reassessed 2026-11-05.** A capability scopes commands per window or webview, and per origin. The app has one window and every page (terminal included) lives in it, so a split cannot give the terminal page more than the chat page. Remote origins already get no IPC: Tauri 2 requires an ACL entry for any command invoked from a non-local origin (`tauri` `webview/mod.rs`, the `!is_local` check), so a page navigated away to the web cannot call app commands even without an app manifest. Splitting starts to pay once there is a second window or embedded remote content; until then the containment is #3 and #5.

### 9. Near-zero automated test coverage

9 `*.test.ts` across 953 source files; 0 Svelte component tests (browser vitest project is configured but unused); Rust tests in ~22 of 365 files; 1 real e2e test.
**Fix:** treat current tests as a smoke floor. Add coverage on the paths that can lose or leak data: credential encryption, updater, command executor.

### 10. `derive_key` panics on zero iterations. **Open.**

`src-tauri/portal-core/src/domains/credentials/services/encryption_service.rs:97`
`NonZeroU32::new(iterations).unwrap()` aborts the thread on `0`.
**Fix:** return `CredentialError` instead of `unwrap()`.

### 11. No global frontend error handling. **Open.**

No `+error.svelte` anywhere, no `hooks.client.ts` `handleError`, no `unhandledrejection` listener. Uncaught load/render errors and rejected promises blank the view or fail silently. The existing `error-boundary.svelte` only reacts to a `component-error` event that nothing dispatches — it catches nothing.
**Fix:** add a root `+error.svelte`, a `hooks.client.ts handleError`, and an `unhandledrejection` listener wired to the toast system. Remove or actually wire the fake error boundary.

---

## P2 — Medium

| #   | Issue                                                                                                                                                           | Location                                                                             | Fix                                                                              |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| 12  | Secret DB unprotected on Windows — `set_file_permissions` is a no-op on non-unix, so the SQLite DB (encrypted creds) gets default ACLs on the primary target OS | `src-tauri/src/database.rs:116`                                                      | Apply Windows ACL (icacls / `SetNamedSecurityInfo`) current-user-only            |
| 13  | Command timeout defined but never enforced — hung child blocks indefinitely                                                                                     | `src-tauri/src/command_executor.rs:27,246`                                           | Wrap in `tokio::time::timeout`, kill child on expiry                             |
| 14  | Legacy DB import from CWD-relative `./data/portal_desktop.db` — a planted DB can be adopted on first run                                                        | `src-tauri/src/database.rs:85`                                                       | Resolve from a fixed absolute path only, or drop the legacy migration            |
| 15  | No `[profile.release]` hardening — ships debug symbols, no strip/lto                                                                                            | `src-tauri/Cargo.toml`                                                               | Add `strip=true`, `lto=true`, `panic="abort"`, `opt-level="s"`                   |
| 16  | 109 `unwrap()`/`expect()`/`panic!` across 32 backend files; a panic in a Tauri command aborts the task                                                          | `version_fetcher.rs` (14), `cli_service.rs` (12), `script_execution_service.rs` (9)… | Audit those reachable from user input / network responses; convert to `AppError` |
| 17  | Dead server auth gate — `hooks.server.ts` never runs under `adapter-static`/Tauri; false sense of security                                                      | `src/hooks.server.ts:8`                                                              | Delete; rely on client `DeviceAuthGuard`                                         |
| 18  | Broken `ai:test` script points at missing `test-ollama-integration.js`; ollama scripts contradict the agent-platform architecture in README                     | `package.json:26-30`                                                                 | Remove ollama scripts (`ai:test`, `ai:setup`, `ollama:*`)                        |
| 19  | README `npm run install` errors — the script is `run:install`                                                                                                   | `README.md:40`                                                                       | Fix docs to `npm run run:install`                                                |
| 20  | Permissive CSP `connect-src` (duplicate of P0 #3 from the build angle)                                                                                          | `tauri.conf.json:19`                                                                 | Scope to real hosts                                                              |

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

_Note for maintainers (2026-07-22): the P0 telemetry endpoint and the credential-key derivation are the two items to fix today — one leaks live data, the other makes credential encryption cosmetic. Everything else can be scheduled._ Both were fixed in `4377b38`.
