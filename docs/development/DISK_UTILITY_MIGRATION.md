# Disk Utility → Portal Desktop Migration

Plan to fold the standalone **portal_disk_utility** app into **portal_desktop** and
retire it as a separate binary.

## Why

- One app to ship, update, and sign instead of two.
- Disk cleanup is a natural "Utility" inside the desktop shell (there is already a
  `utilities` route group and a domain-per-feature backend).
- The AI verification the disk utility does by hand duplicates the desktop's `ai`
  domain (Ollama / Anthropic / OpenAI / Gemini providers) — merging lets us drop the
  duplicate HTTP client.

## Compatibility assessment

| Layer | disk utility | desktop | Portable? |
|-------|--------------|---------|-----------|
| Shell | Tauri 2 | Tauri 2 | ✅ same |
| Backend | plain Rust modules | domain-per-feature (`src-tauri/src/domains/*`) | ✅ move under a domain |
| Frontend | **React 18** + own Tailwind | **Svelte 5 / SvelteKit** + shadcn-svelte + bits-ui | ❌ must be rewritten |
| Local DB | `rusqlite` (bundled), own `portal.db` | `sea-orm` / `sqlx` | ⚠️ keep rusqlite for the disk domain (MVP) or port to sea-orm entities |
| AI verify | bespoke `reqwest::blocking` client → Agent Platform | full `ai` domain (async providers) | ⚠️ rewire later; keep as-is for MVP |

**Verdict: worth doing.** Backend ports mechanically; the real work is a React→Svelte
UI rewrite.

## Backend (this scaffold)

New domain `src-tauri/src/domains/disk/` — a straight move of the 8 Rust modules with
their `crate::` paths re-pointed under the domain, plus a `commands.rs` holding the
`#[tauri::command]` fns and shared state that used to live in the disk utility's
`lib.rs`.

```
domains/disk/
  mod.rs          module wiring
  commands.rs     tauri commands + ScanSummary/CachedScan/ScanControl/VerifyControl
  scan.rs         parallel two-pass walk (jwalk)
  classify.rs     heuristic cleanup proposals
  projects.rs     project-aware (node_modules/target/…) scan
  quarantine.rs   move-to-Recycle-Bin (trash) — the only destructive path, reversible
  verify.rs       advisory multi-agent review (Agent Platform)
  disk.rs         mounted-volume capacity (sysinfo)
  locations.rs    suggested scan roots
  db.rs           audit log + protected paths + scan cache (rusqlite)
```

Wiring in `lib.rs`:
- `app.manage(Arc::new(disk::db::Db::open(app_data_dir.join("disk_utility.db"))?))`
- `app.manage(disk::commands::ScanControl::default())`
- `app.manage(disk::commands::VerifyControl::default())`
- register the 17 commands in `generate_handler!`.

Cargo deps to add to `src-tauri/Cargo.toml`:
- `jwalk = "0.8"`
- `trash = "5"`
- `rusqlite = { version = "0.32", features = ["bundled"] }`
- add `"blocking"` to the existing `reqwest` features (or remove once verify is rewired to the `ai` domain).
- `sysinfo` already present (0.30) — the `Disks` API is compatible.

### Commands surfaced
`scan_directory`, `scan_projects`, `cancel_scan`, `get_cached_scan`,
`remove_cached_scan`, `quarantine_paths`, `get_audit_log`, `list_protected`,
`add_protected`, `remove_protected`, `list_locations`, `disk_usage`,
`verify_proposals`, `cancel_verify`, `list_ai_teams`, `provision_ai_team`,
`open_recycle_bin`.

Emitted events: `scan://progress`, `quarantine://progress`, `verify://progress`.

## Frontend (follow-up — the bulk of the work)

Rebuild the React UI as Svelte routes under `src/routes/utilities/disk/`:
- Dashboard: `disk_usage` capacity bars.
- Cleanup tab: location picker (`list_locations`), scan + progress, proposals table
  (`@tanstack/table-core` already a dep), quarantine confirm.
- Projects tab: `scan_projects` grouped by project.
- Audit log: `get_audit_log`.
- Settings: protected paths, AI verify config (rewire onto the `ai` domain).

No component copy-paste is possible (React → Svelte 5). Reuse desktop's existing
shadcn-svelte primitives, Tauri `invoke` + `listen` for the progress events.

## Estimated effort

- Backend move + wire: ~1 day.
- Rewire verify onto the `ai` domain: ~0.5 day (optional for MVP).
- Frontend rewrite: ~3–5 days.
- Retire standalone repo + add nav entry under Utilities.

## Risk

Low correctness risk — quarantine goes to the Recycle Bin (reversible) and every
action is audited. Main cost is UI rewrite time, plus a slightly larger binary while
both rusqlite and sqlx are linked (removable by porting `db.rs` to sea-orm later).

## Status

- ✅ Backend domain (`src-tauri/src/domains/disk/`) — moved, wired, compiles.
- ✅ Svelte UI (`src/lib/domains/disk/`, route `/utilities/disk`, nav entry) — passes `svelte-check`.
- ✅ AI verify rewired onto the `ai` domain (`verify_ai.rs`): `verify_proposals` now calls
  `AIService::generate_with_system` with whatever provider the user enabled, parses the same
  trailing-JSON verdicts, and drives the existing UI panel with two synthetic progress frames.
  The old Agent-Platform path (`verify.rs`) is retained behind `#[allow(dead_code)]` as a fallback;
  `list_ai_teams` / `provision_ai_team` still target it (Settings marks those legacy).

Remaining: single-call generation isn't mid-flight cancellable (Stop only pre-empts before start);
theming still uses hardcoded `neutral-*` palette rather than the desktop CSS-variable tokens.

## Cutover

1. Land backend domain + commands (compiles, no UI yet).
2. Build Svelte UI incrementally behind the Utilities nav.
3. Rewire AI verify onto the `ai` domain.
4. Feature-parity check against the standalone app.
5. Archive `portal_disk_utility` repo; drop from workspace.
