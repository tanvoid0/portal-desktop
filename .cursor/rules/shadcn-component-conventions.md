# shadcn Component Conventions

Use shadcn-svelte primitives from `$lib/components/ui` for interactive UI. See [SHADCN_COMPONENT_AUDIT.md](../../docs/development/SHADCN_COMPONENT_AUDIT.md) for per-file status.

## Required imports

| Control | Import |
|---------|--------|
| Actions | `{ Button } from "$lib/components/ui/button"` |
| Text fields | `{ Input }`, `{ Textarea }`, `{ Label }` |
| Selection | `import Select from "$lib/components/ui/select.svelte"` — **not** raw `<select>` |
| Toggles | `{ Checkbox }`, `{ Switch }` |
| Overlays | `{ Dialog, DialogContent, ... }`, `{ Sheet, ... }`, `{ AlertDialog, ... }` |
| Layout/data | `{ Card, ... }`, `{ Table, ... }`, `{ Badge }` |
| Command palettes | `Dialog` + `Command` — see `k8s-navigation/components/CommandPalette.svelte` |

## Do not

- Add `fixed inset-0 z-50` modal divs in domain or route code — use Dialog, Sheet, or AlertDialog
- Duplicate Button/Select with raw `<button>` / hand-built dropdowns
- Rewrite files under `src/lib/components/ui/<registry-component>/` (shadcn primitives)
- Use `@/` imports — use `$lib/`

## Approved exceptions

- Hidden `<input type="file">` for file pickers
- Semantic `<kbd>` for keyboard hints
- Layout-only divs (resizable panes, canvas areas)
- Thin delegate components that only compose shadcn children

## Simplified Select API

The wrapper at `$lib/components/ui/select.svelte` accepts:

- `options`: `string[]`, `{ value, label }[]`, readonly constant arrays, or TypeScript enums
- `bind:value`, `placeholder`, `disabled`, `required`, `error`
- `onSelect` / `onValueChange` callbacks

It is implemented on top of shadcn `Select.*` primitives.

## Page shell

Page chrome (`PageHeader`, `PageLoading`, etc.) lives in `$lib/components/shell` — orthogonal to shadcn primitives. See [page-shell-conventions.md](./page-shell-conventions.md).
