---
name: terminal-rewrite
overview: Introduce a new, aiTerm-inspired terminal component (kept separate) and wire it in place of the current xterm terminal. Extend the Rust backend to persist scrollback + notes via DB and inject OSC 133 command markers for reliable command tracking; add OSC 8 file hyperlink rendering in the new terminal output pipeline.
todos:
  - id: new-terminal-component
    content: Create `TerminalAiV2.svelte` as a separate terminal UI component (xterm mount + IO wiring + sidebar + notes panel). Keep `Terminal.svelte` untouched and unused.
    status: completed
  - id: new-command-blocks
    content: Add a new command-block UI component (e.g. `CommandBlocksAiV2.svelte`) that consumes `shell-integration-event` payloads with correct field mapping (`start_time/working_directory/exit_code/duration`). Remove the existing debug/test block behavior (do not change old command blocks file).
    status: completed
  - id: wire-new-terminal
    content: Update `TerminalTabContainer.svelte` and `ProjectTerminal.svelte` to render `TerminalAiV2.svelte` instead of `Terminal.svelte`.
    status: completed
  - id: osc8-injection
    content: Implement OSC 8 file-link rendering in the new terminal output path (inject OSC 8 sequences into terminal writes). Add a small chunk-tail buffer to handle split paths across chunks. Do not implement editor-opening click handlers yet.
    status: completed
  - id: db-backed-terminal-sessions
    content: Add SeaORM migrations/entities for persistent terminal sessions and notes. Update Rust `save_terminal_session`/`load_terminal_session`/`delete_terminal_session` to use DB-backed storage instead of the current in-memory `OnceLock` map.
    status: completed
  - id: terminal-notes-backend
    content: Add Rust Tauri commands `save_terminal_note` and `load_terminal_note` backed by a new DB table (or via terminal_sessions row if you prefer). Register them in `src-tauri/src/lib.rs`.
    status: completed
  - id: terminal-notes-frontend
    content: Add `terminalNotesStore.ts` (or local state + invoke wrappers) and wire it into `TerminalAiV2.svelte` with debounced saving.
    status: completed
  - id: fix-sessionstore-args
    content: "Fix TS/Rust argument mismatch in `sessionStore.ts`: ensure `load_terminal_session` and `delete_terminal_session` send `tab_id` rather than `tabId` to match Rust function parameters."
    status: completed
  - id: osc133-injection
    content: Implement OSC 133 injection for zsh/bash during PTY startup in `src-tauri/src/domains/terminal/manager.rs` by starting zsh with a temporary ZDOTDIR containing hooks, and starting bash with a temporary --rcfile containing DEBUG+PROMPT_COMMAND hooks (emitting OSC 133 A/B terminated with `ESC \\`).
    status: completed
  - id: update-types-if-needed
    content: Adjust or add TS types/helpers so `shell-integration-event` payload parsing works reliably with the actual Rust serialized field names.
    status: completed
  - id: smoke-tests
    content: "Run Tauri dev/build to validate: command blocks populate correctly, scrollback persists across restarts, notes persist per tab, and file hyperlinks render as OSC 8."
    status: completed
isProject: false
---

## Overview

You want to replace the current terminal UI with a new aiTerm-style implementation while keeping the old code in-repo for reference/archiving.

The repo already has:

- A PTY-backed terminal backend (`src-tauri/src/domains/terminal/*`) streaming `terminal-output` events.
- A current xterm-based frontend terminal UI (`src/lib/domains/terminal/components/Terminal.svelte`).
- Session persistence primitives in the frontend store (`sessionStore`) and backend commands, but today they are in-memory only.
- Shell integration parsing for OSC 133 in Rust (`shell_integration.rs`) and command-block UI components that listen for `shell-integration-event`.

This plan creates a new terminal component that:

- Uses the same backend process APIs as today (drop-in on top of PTY + `TerminalService`).
- Persists scrollback and terminal notes across restarts by changing backend session persistence to DB-backed storage.
- Injects OSC 133 markers for zsh/bash at terminal startup so backend emits accurate command start/end events.
- Renders OSC 8 clickable file links by injecting OSC 8 sequences into output before writing into xterm.

## Implementation outline

1. **Archive old terminal code**
  - Keep `src/lib/domains/terminal/components/Terminal.svelte` untouched for reference.
  - Keep old AI wrapper `AITerminalContainer.svelte` untouched.
2. **Add new terminal UI component(s)**
  - Create a new terminal component file (e.g. `TerminalAiV2.svelte`) that contains the xterm mount, IO wiring, sidebar layout, and notes panel.
  - Create new versions of any terminal subcomponents needed for the new command tracking UX (especially command blocks), so we don’t have to mutate the old terminal’s behavior.
  - Update `TerminalTabContainer.svelte` and `ProjectTerminal.svelte` to render the new component instead of `Terminal.svelte`.
3. **Backend: DB-backed persistent scrollback + notes**
  - Add SeaORM entities + migrations for:
    - `terminal_sessions` (tab_id, working_directory, environment_json, scrollback_text, cursor_position, terminal_size, last_activity)
    - `terminal_notes` (tab_id, markdown, updated_at)
  - Change backend commands:
    - `save_terminal_session` / `load_terminal_session` / `delete_terminal_session` to use DB instead of the current in-memory `OnceLock` map.
    - Add `save_terminal_note` / `load_terminal_note` commands.
  - Fix the current TS/Rust arg mismatch for `load_terminal_session`/`delete_terminal_session` (TS currently sends `{ tabId }` while Rust expects `tab_id`).
4. **Backend: inject OSC 133 markers**
  - Update `src-tauri/src/domains/terminal/manager.rs` in `create_process` so that when the requested shell is zsh or bash, the spawned shell is started with temporary hooks that emit OSC 133 A/B markers using `ESC ]133;A ESC \\` and `ESC ]133;B ESC \\`.
  - Rust `shell_integration.rs` will then produce `shell-integration-event` payloads which the new command-block UI will consume.
5. **Frontend: OSC 8 rendering**
  - In the new terminal component’s `handleOutput`, inject OSC 8 sequences into `output.content` before calling `terminal.write(...)`.
  - Maintain a small tail buffer so file paths split across chunks are still linkified.
  - Note: for now we will only render OSC 8 links (no editor-open handler yet), because you chose to avoid OS/in-app editor opening until an in-app file editor exists.
6. **Verification**
  - Build + run Tauri dev.
  - Validate:
    - Opening a terminal, running commands populates command blocks with exit code/duration.
    - Closing/restarting the app reloads terminal scrollback from DB.
    - Notes persist per tab.
    - Terminal shows clickable file links (even if clicking doesn’t open anything yet).

## Mermaid: data flow

```mermaid
flowchart LR
  A[PTY Output (Rust TerminalManager)] -->|terminal-output| B[New Terminal UI]
  B -->|transform + terminal.write| C[xterm.js buffer]
  B -->|periodic save| D[save_terminal_session command]
  D -->|DB upsert| E[(terminal_sessions SQLite)]
  B -->|load on mount| F[load_terminal_session command]
  F -->|read| E

  A -->|shell-integration-event (OSC 133 parse)| G[New Command Blocks UI]

  B -->|optional: OSC 8 injection| H[OSC 8 sequences in output]
  H --> C

  B -->|notes editor changes| I[save_terminal_note command]
  I --> (terminal_notes SQLite)
```



## Key files to change/add

- Add: `src/lib/domains/terminal/components/TerminalAiV2.svelte`
- Update: `src/lib/domains/terminal/components/TerminalTabContainer.svelte`
- Update: `src/lib/domains/terminal/components/ProjectTerminal.svelte`
- Add: `src/lib/domains/terminal/components/CommandBlocksAiV2.svelte` (if needed for fixed event mapping)
- Update: `src/lib/domains/terminal/stores/sessionStore.ts` (fix `tab_id` arg)
- Add: `src/lib/domains/terminal/stores/terminalNotesStore.ts`
- Add: `src/lib/domains/terminal/utils/osc8.ts`
- Add: SeaORM entities + migrations under `src-tauri/src/entities/*` and `src-tauri/src/migrations/*`
- Update: `src-tauri/src/domains/terminal/commands.rs` (DB persistence + note commands)
- Update: `src-tauri/src/domains/terminal/manager.rs` (OSC 133 injection for zsh/bash)
- Update: `src-tauri/src/lib.rs` (register new Tauri commands)

