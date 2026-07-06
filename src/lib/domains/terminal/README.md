# Terminal Workspace

Unified interactive terminal for Portal Desktop — WaveTerm-style widget rail with Warp-style command blocks and AI assistance.

## Architecture

```
/terminal
  └── TerminalWorkspace
        ├── TabContainer / TabBar (multi-tab sessions)
        ├── TerminalSession (per tab)
        │     ├── XtermPane (interactive PTY via xterm.js)
        │     └── CommandInputBar (optional structured input + /ai)
        └── Widget rail (collapsible)
              ├── CommandBlocksPanel (OSC 133 capture)
              ├── AIAssistantPanel
              ├── NotesPanel
              ├── SessionLauncher (projects / containers)
              └── CommandHistoryPanel
```

### Shared core

| Module | Purpose |
|--------|---------|
| `composables/useXtermSession.ts` | xterm lifecycle, OSC8 links, session persistence |
| `composables/useTerminalProcess.ts` | PTY create/input/resize/kill |
| `components/core/XtermPane.svelte` | xterm canvas (interactive or one-shot) |
| `components/core/TerminalSession.svelte` | xterm + command input + optional widget slot |
| `stores/commandBlockStore.ts` | Unified command output capture |

### Embedded terminals

- **Project pages:** `ProjectTerminal.svelte` — scoped tabs via `resourceName: "project"`
- **Script runs:** `EmbeddedTerminal.svelte` — one-shot mode via `XtermPane mode="oneshot"`

## Keyboard shortcuts

- `Ctrl+T` — New tab
- `Ctrl+W` — Close tab
- `Ctrl+Tab` / `Ctrl+Shift+Tab` — Switch tabs
- `Ctrl+1-9` — Switch to tab by number
- `Ctrl+Space` — Toggle AI mode in command input bar
- `Ctrl+K` — Command palette

## Deep links

| URL param | Behavior |
|-----------|----------|
| `?command=` | Pre-fill and run command in active tab |
| `?container={id}` | Open Docker exec terminal tab |
| `?project={id}` | Focus existing project-scoped tab |

## Readonly command capture

For cross-domain readonly execution:

```typescript
import { TerminalService, commandBlockStore } from "$lib/domains/terminal";

const result = await TerminalService.executeInContext(
  { tabId, workingDirectory: cwd, environment: {} },
  "kubectl get pods",
);
// Results are automatically captured in commandBlockStore for the tab
```

## Backend

Tauri PTY (`src-tauri/src/domains/terminal/`), OSC 133 shell integration, SQLite persistence for history/sessions/notes.
