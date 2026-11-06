---
id: QA-085
title: Switching the coder engine rewrites the active thread's llm_provider, falsifying which engine did the work
status: open
severity: major
kind: bug
area: coder / session state
mode: tauri
found: 2026-11-06
---

## Steps

1. AI -> Code. Start a session on engine **Claude Code (CLI)** and run a turn that edits a
   file, so the thread has real work and at least one `coder_file_changes` row.
2. Record the truth:
   ```bash
   cp ~/AppData/Roaming/com.tanvoid0.portal-desktop/portal_desktop.db /tmp/pd.db
   sqlite3 /tmp/pd.db "select id, llm_provider from coder_threads order by rowid desc limit 1;"
   ```
   -> `2bb8fc85-69ee-4b65-b700-c9425aa02574|claude-cli`
3. Change the engine selector to **Ollama**, then click **New session**.
4. Re-query the same thread id.

## Actual

The *old* thread's provider has been rewritten:

```
2bb8fc85-69ee-4b65-b700-c9425aa02574|ollama|2
```

Observed in this session: thread `2bb8fc85` was created on `claude-cli`, ran two Claude
Code CLI turns, and recorded two `coder_file_changes` rows. After the selector was moved
to Ollama it reads `llm_provider=ollama` — while still owning the two changes the Claude
CLI engine produced. No Ollama turn was ever run on it.

## Root cause

[coderSession.svelte.ts:938-947](../../../src/lib/domains/coder/state/coderSession.svelte.ts#L938)

```ts
async createThreadIfNeeded(): Promise<CoderThread> {
  if (this.thread) {
    this.upsertThread(this.thread);
    if (
      (this.selectedModel && this.selectedModel !== this.thread.model) ||
      (this.selectedBackendProvider && this.selectedBackendProvider !== this.thread.llm_provider)
    ) {
      await this.syncThreadLlmConfig(this.thread.id);   // <-- rewrites the EXISTING thread
    }
    return this.thread;
  }
  ...
```

When a thread is already active, a selector change is pushed onto **that** thread instead
of being held for the next one.

## Expected

The selector's value applies to the **next** thread created. An existing thread's
`llm_provider` is only changed when the user explicitly asks to move that conversation to
another engine — and if that is a supported action, it should say so, since the thread's
existing messages and recorded changes were produced by the previous engine.

## Why it matters

`llm_provider` is the only record of which engine produced a thread's messages and its
`coder_file_changes` rows. Both `.claude/handoff.md` and `.claude/coder-test-plan.md`
instruct the tester to trust it ("engine is stored per thread in
`coder_threads.llm_provider`, so reusing an old thread tests the wrong thing"), and the
sessions list filters on it
([sessionList.ts:196](../../../src/lib/domains/coder/utils/sessionList.ts#L196)).

Silently re-labelling a thread means:

- the per-engine history is wrong, and wrong in a way nothing in the UI reveals;
- a provider-comparison run like Part B of the test plan can attribute one engine's
  output to the other. This very session nearly did — the Part A result had to be read
  off a DB snapshot taken *before* the switch to stay trustworthy.

## Note

This also makes the test plan's own advice insufficient. "Switch the selector, *then*
start a new session" is not safe while a thread is active: the switch lands on the old
thread first. Open a new session first, then pick the engine.
