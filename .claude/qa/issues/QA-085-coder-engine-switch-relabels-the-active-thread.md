---
id: QA-085
title: Switching the coder engine rewrites the active thread's llm_provider, falsifying which engine did the work
status: verified
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

## Correction to the root cause above

`createThreadIfNeeded` was only the *secondary* path. The one that actually fired in the
repro is `handleBackendProviderChange`
([coderSession.svelte.ts:1410](../../../src/lib/domains/coder/state/coderSession.svelte.ts#L1410)),
which called `syncThreadLlmConfig(this.thread.id)` **the moment the dropdown changed** —
before any "New session" click. That is why thread `2bb8fc85` was already relabelled by
the time a new session was started. Both paths wrote; both are fixed.

## Fix — 2026-11-06

- `handleBackendProviderChange` no longer writes to the active thread at all. It records
  the choice (state + `localStorage` default for the next session) and, when a thread is
  already on a different engine, says so instead of staying silent:
  `This session stays on Ollama. Start a new session to use Claude Code (CLI).`
- `createThreadIfNeeded` now **back-fills only** — it writes model/provider solely when
  the thread has one missing, and preserves whatever is already recorded.
- `syncThreadLlmConfig` takes explicit `model` / `provider` arguments rather than reading
  the selector, so it can no longer clobber one field while filling the other.
- `handleModelChange` still updates the active thread's model (legitimate mid-thread) but
  passes the thread's existing engine through.

**Verified in the running app:**

| check | result |
| ----- | ------ |
| thread `77074dc4` provider before switching the selector | `ollama` |
| switched selector to **Claude Code (CLI)** | toast shown, selector moved |
| thread `77074dc4` provider after | **still `ollama`** |
| new session on the selected engine (non-regression) | new thread `e55cfc54` created as `claude-cli`, turn ran |

Shot: `.claude/qa/shots/coder-QA-085-after-fix.png`.

## Verification

2026-11-09 — tauri (code read, app went down mid-sweep — see QA-104, so the live
switch-then-requery repro could not be rerun). `coderSession.svelte.ts:1443-1462`
(`handleBackendProviderChange`) no longer calls `syncThreadLlmConfig` at all — it only sets
`selectedBackendProvider`, persists it to `localStorage`, and toasts
`This session stays on <engine>. Start a new session to use <engine>.` when a thread is
already on a different engine. `createThreadIfNeeded` (`:965-977`) now back-fills only,
calling `syncThreadLlmConfig` solely when `llmConfigFor(this.thread)` found a missing
field. Matches the Fix description exactly; the coder's own before/after table
(thread `77074dc4` provider unchanged after switching the selector) is consistent with this
code. **Verified.**

## Known tradeoff

The engine dropdown can now show an engine the *current* thread will not use. The toast is
what keeps that from being silent. Pinning the dropdown to the active thread, or disabling
it once a thread has messages, would be tidier but is a UI change beyond this fix.
