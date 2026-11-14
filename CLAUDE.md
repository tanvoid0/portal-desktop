# Portal Desktop

Tauri 2 desktop app.

- **Backend:** Rust — `src-tauri/` (thin command layer) + `src-tauri/portal-core/` (domain logic, SeaORM migrations).
- **Frontend:** SvelteKit + **Svelte 5 (runes)** + TypeScript — `src/`. Tailwind 4 + **shadcn-svelte** UI.
- Frontend talks to the backend over Tauri `invoke`, never HTTP.

## Commands

```bash
pnpm check        # svelte-check (type + a11y) — the pre-push gate
pnpm lint         # prettier --check + eslint + shadcn-ui check
pnpm format       # prettier --write
pnpm tauri:dev    # run the full app (Rust + webview)
pnpm gen:types    # regenerate TS types from Rust (scripts/gen-types.mjs)
```

Rust: `cargo check` / `cargo test` / `cargo clippy` from `src-tauri/`.

**Before calling frontend work done: `pnpm check && pnpm lint`.** Both must pass.

---

## Frontend is Svelte 5 runes. Not Svelte 4. Not React.

This is the #1 source of wrong output. Match the existing code (see `src/lib/domains/projects/components/ProjectCard.svelte` for a clean reference).

**Props** — `interface Props` + `$props()`, with callback props. No `export let`, no `createEventDispatcher`:

```svelte
<script lang="ts">
  interface Props {
    project: Project;
    onEdit?: (p: Project) => void; // callbacks, not dispatched events
  }
  let { project, onEdit = () => {} }: Props = $props();
</script>
```

**State / derived:**

- `let open = $state(false);`
- `const label = $derived(expr);` — single expression.
- `const rows = $derived.by(() => { ... });` — when you need a block body.
- **NEVER `$derived(() => ...)`** — that stores the _function_, memoizes nothing, and re-runs at every read. Enforced by the `portal/no-derived-closure` eslint rule. Use `$derived.by` for a body, `$derived(expr)` for an expression.

**Other runes idioms:**

- `ref = $bindable(null)`, `bind:value`, `bind:open` for two-way binding.
- Children via snippets: `{@render children?.()}`, not `<slot>`.
- DOM events are plain attributes: `onclick`, `oninput` — **not** `on:click`.
- Side effects: `$effect(() => { ... })`. Don't load data at module top level (see data rules).
- Shared reactive state lives in `*.svelte.ts` files using runes, not `writable()` stores.

When unsure about a rune, check the Svelte 5 docs (links at the bottom) — do not guess Svelte 4 syntax.

**The Svelte 4 forms above are lint errors, not advice.** `portal/no-legacy-syntax` fails `pnpm lint` on `export let`, `on:click`, `$:`, `<slot>`, `<svelte:fragment>` and `<svelte:component>`; `no-restricted-imports` fails on `createEventDispatcher`. `writable()` for server data is the one convention still left to review, since it is not decidable from the syntax.

---

## UI: use shadcn-svelte primitives

Interactive controls come from `$lib/components/ui/*`. **Raw `<button>` / `<input>` / `<textarea>` / `<select>` / `<label>` in app code is a lint error** (`portal/no-raw-ui-elements`).

| Need       | Use                                                                                               |
| ---------- | ------------------------------------------------------------------------------------------------- |
| Button     | `{ Button } from '$lib/components/ui/button'`                                                     |
| Text field | `{ Input }`, `{ Textarea }`, `{ Label }`                                                          |
| Select     | `import Select from '$lib/components/ui/select.svelte'` (wrapper — takes `options`, `bind:value`) |
| Toggle     | `{ Checkbox }`, `{ Switch }`, `{ Toggle }`, `{ ToggleGroup }`                                     |
| Overlay    | `{ Dialog }`, `{ Sheet }`, `{ AlertDialog }` — never a hand-rolled `fixed inset-0 z-50` div       |
| Layout     | `{ Card }`, `{ Table }`, `{ Badge }`                                                              |

Rules:

- **Do not edit files under `src/lib/components/ui/`** — those are vendored shadcn-svelte primitives (`pnpm lint` checks them). Compose on top instead.
- Imports use `$lib/...`, never `@/...`.
- Icons: `@lucide/svelte` (not `lucide-svelte`). Tech/brand logos: `TechIcon` / `devicon`.
- Toasts: `import { toast } from '$lib/utils/toast'`.
- Confirm dialogs: `confirmAction()` or `AlertDialog`, never raw `confirm()`.
- Styling: Tailwind + semantic tokens (`bg-primary`, `text-muted-foreground`, `bg-destructive`), `cn()` from `$lib/utils`, `tv()` (tailwind-variants) for variant APIs.

**List / management pages** use the shared shell in `$lib/components/shell`: `PageHeader`, `PageStats`, `PageFilters`, `PageLoading`, `PageError`, `PageEmpty`. See `.cursor/rules/page-shell-conventions.md` for the standard page skeleton.

---

## Responsive: design for the narrow window and the ultrawide monitor, not just your laptop

This is a desktop window the user resizes and puts on any monitor — not a fixed-size canvas. Before calling a page or component done, check both ends:

- **No fixed-width columns/panels without a breakpoint.** A `grid-cols-[16rem_1fr]` or a bare `w-96`/`w-[220px]` rail that never stacks or collapses breaks on a narrow window. Use a responsive prefix (`md:grid-cols-[...]`, `grid-cols-1 md:grid-cols-[16rem_1fr]`) or route it through `ResponsivePanel` (`$lib/components/shell/responsive-panel.svelte`), which already collapses to a Sheet below 768px.
- **Content needs a max-width cap, or it stretches edge-to-edge on an ultrawide monitor.** Use `PageContainer` (`$lib/components/shell/page-container.svelte`) — its `full` variant is capped via `--content-max-width-full`; reach for `readable` on text-heavy pages. Don't skip `PageContainer` or use the bare Tailwind `container` class (Tailwind 4 has no breakpoint config for it here — it just resolves to 100%).
- **No raw `w-screen` / unclamped `100vw`.** If you need viewport-relative sizing, clamp it (`min(100vw-2rem, 46rem)`), matching the pattern already used in `CatalogModelSelect.svelte` / `ProjectCombobox.svelte`.

---

## Data flow (frontend ↔ Rust)

1. **Call:** a domain `api/*.ts` wraps the Tauri command:
   ```ts
   import { invokeClient } from '$lib/utils/invokeClient';
   const raw = await invokeClient.post<unknown>('get_all_projects');
   ```
2. **Normalize:** Rust returns snake_case; map it in `utils/normalize*.ts` to the TS shape.
3. **Cache:** server state goes through **TanStack Query** (`@tanstack/svelte-query`) — query keys in `src/lib/domains/shared/query/keys.ts`, client in `.../queryClient.ts`. **Do not add new `writable()` stores for server data.**
4. **Types:** shared types live in `types/`; Rust-generated ones in `generated/` (regenerate with `pnpm gen:types`).

Never load backend data at module import time — load in `onMount`/`$effect` or via a Query. Full detail: `.cursor/rules/data-loading-conventions.md`.

---

## Domain structure

Frontend is domain-driven: `src/lib/domains/<domain>/` with `components/`, `api/`, `services/`, `queries/`, `types/`, `utils/`, `generated/`, `state/` (`*.svelte.ts` runes), `stores/` (mutations only). Cross-domain primitives in `src/lib/components/`.

Backend mirrors it: `src-tauri/portal-core/src/domains/<domain>/`.

---

## Detailed conventions (read the relevant one before working in that area)

These are the source of truth — this file summarizes them:

- `.cursor/rules/shadcn-ui.mdc` + `.cursor/rules/shadcn-component-conventions.md` — UI mappings, Button variant patterns, allowed exceptions.
- `.cursor/rules/page-shell-conventions.md` — page skeleton, theme, navigation.
- `.cursor/rules/data-loading-conventions.md` — TanStack Query, TTL cache, the `getList`/`[]`-is-truthy trap.
- `docs/development/SHADCN_COMPONENT_AUDIT.md` — per-file migration status.

## The shared daemon (port 18420)

The **installed release's** daemon on loopback port 18420 is the one shared task-board backend for every project on this machine — not a per-session instance. Every session/agent must talk to that single running Portal, never assume it owns one. To bring it up, run `pnpm portal:ensure` (idempotent, lock-protected, safe to call from multiple sessions at once): it launches the installed release (`%LOCALAPPDATA%\Portal Desktop\portal_desktop.exe`, or `PORTAL_EXE`), never a build from this repo. `tauri-plugin-single-instance` is the hard backstop: one release process and one dev process at most.

## Dev and release never share data

Debug builds (`pnpm tauri:dev`, `cargo build`) run as `com.tanvoid0.portal-desktop.dev` — decided by `app_paths::DEV` (`cfg!(debug_assertions)`), not by a script, so nothing can forget it:

|                  | release (`pnpm tauri:build`, installed)      | dev (any debug build)                            |
| ---------------- | -------------------------------------------- | ------------------------------------------------ |
| data dir (DB)    | `%APPDATA%\com.tanvoid0.portal-desktop`      | `%APPDATA%\com.tanvoid0.portal-desktop.dev`      |
| config dir       | `%APPDATA%\portal_desktop`                   | `%APPDATA%\portal_desktop-dev`                   |
| WebView2 storage | `%LOCALAPPDATA%\com.tanvoid0.portal-desktop` | `%LOCALAPPDATA%\com.tanvoid0.portal-desktop.dev` |
| keychain entry   | `com.tanvoid0.portal-desktop`                | `com.tanvoid0.portal-desktop.dev`                |
| daemon port      | 18420 (the board)                            | 18421 (+N-1 for `dev-instance.mjs N`)            |

A dev build's first launch **copies** the release's DB, config and master key into its own — real data to work against; its migrations then run on the copy only. Reset dev data by deleting the two `.dev`/`-dev` dirs. The release only ever runs migrations it shipped with.

## When unsure about Svelte 5 or shadcn-svelte

Prefer fetching the real docs over guessing (training data skews Svelte 4 / React):

- Svelte 5 + SvelteKit, compressed for LLMs: https://svelte.dev/llms-small.txt (full: https://svelte.dev/llms-full.txt)
- shadcn-svelte components: https://shadcn-svelte.com/docs
