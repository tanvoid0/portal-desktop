# Terminal Workspace

Unified interactive terminal for Portal Desktop — Warp-style command blocks as the primary view, with AI assistance and a collapsible widget rail.

## Architecture

```
/terminal
  └── TerminalWorkspace
        ├── TabContainer / TabBar (multi-tab sessions)
        ├── TerminalSession (per tab)
        │     ├── BlocksView (Warp-style: one block per command; default view)
        │     ├── Terminal (raw xterm PTY; toggle for interactive apps)
        │     └── CommandInputBar (view toggle + command/AI input)
        └── Widget rail (collapsible)
              ├── CommandBlocksPanel (OSC 133 capture)
              ├── AIAssistantPanel (context-aware, runnable suggestions)
              ├── NotesPanel
              ├── SessionLauncher (projects → project page; containers → global tabs)
              └── CommandHistoryPanel
```

### Shared core

| Module | Purpose |
|--------|---------|
| `composables/useXtermSession.ts` | xterm lifecycle, OSC8 links, session persistence, live block streaming |
| `composables/useTerminalProcess.ts` | PTY create/input/resize/kill |
| `components/core/Terminal.svelte` | xterm canvas (interactive / one-shot / display) |
| `components/core/BlocksView.svelte` | Warp-style block list (primary surface) |
| `components/core/TerminalSession.svelte` | blocks + xterm + command input + optional widget slot |
| `components/CommandBlock.svelte` | one command block: status, actions, live output |
| `stores/commandBlockStore.ts` | Unified command output capture |
| `services/terminalAiContext.ts` | AI context building + response parsing (runnable suggestions) |

### Shell integration (OSC 133)

The backend injects hooks per shell so every command becomes a tracked block
with command text, cwd, output, duration and exit code:

- **PowerShell / pwsh** (Windows default): temp profile overriding `prompt` +
  `PSConsoleHostReadLine` (needs PSReadLine, present by default).
- **zsh**: temp `ZDOTDIR` with `preexec`/`precmd`.
- **bash**: temp `--rcfile` with `DEBUG` trap + `PROMPT_COMMAND`.
- **cmd.exe**: no hooks — blocks fall back to input-bar submissions only.

Internal marker protocol: `133;A;<cwd>` command started, `133;C;<command>`
command text, `133;B;<exit>` command finished.

### AI

The AI assistant (side panel, `/ai` prefix, or Ctrl+Space in the input bar)
receives session context: OS, shell, cwd, and the last commands with output.
Fenced code blocks in AI answers render with **Run** buttons that execute in
the session. Failed blocks get an **Explain** action.

### Embedded terminals

- **Project pages:** `ProjectTerminal.svelte` — project-scoped workspace (`/projects/{id}?tab=terminal`); cwd starts at project path; future project-specific features live here
- **Global terminal:** `/terminal` — unscoped tabs; container exec via Session Launcher or `?container=`
- **Script runs:** `EmbeddedTerminal.svelte` — one-shot mode via `XtermPane mode="oneshot"`
- **Coder:** `CoderSessionTerminal.svelte` — lightweight `Terminal` primitive with per-thread tabs

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
| `?container={id}` | Open Docker exec terminal tab (global `/terminal`) |
| `?project={id}` | Redirect to `/projects/{id}?tab=terminal` |

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
