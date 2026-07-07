import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ContextUsage, LlmUsage } from "$lib/domains/ai/types/index.js";
import type {
  ChatMessage,
  CoderDoneEvent,
  CoderThread,
  FileChange,
  GitDiffStats,
  PendingApproval,
  PermissionMode,
  PermissionRule,
  ThreadTitleEvent,
} from "../types.js";

/**
 * Thin wrapper over the `coder_*` Tauri commands. The agent loop, tool
 * execution, and permission gating all live in Rust; this only marshals calls.
 */
export class CoderService {
  createThread(workspaceRoot: string, model?: string): Promise<CoderThread> {
    return invoke<CoderThread>("coder_create_thread", {
      workspaceRoot,
      model: model ?? null,
    });
  }

  listThreads(): Promise<CoderThread[]> {
    return invoke<CoderThread[]>("coder_list_threads");
  }

  getThread(threadId: string): Promise<CoderThread | null> {
    return invoke<CoderThread | null>("coder_get_thread", { threadId });
  }

  deleteThread(threadId: string): Promise<boolean> {
    return invoke<boolean>("coder_delete_thread", { threadId });
  }

  updateThreadModel(threadId: string, model: string | null): Promise<void> {
    return invoke<void>("coder_update_thread_model", { threadId, model });
  }

  /** Send a user message and start a background run. */
  send(threadId: string, message: string): Promise<void> {
    return invoke<void>("coder_send", { threadId, message });
  }

  /** Resume a failed run from the current thread state. */
  retry(threadId: string): Promise<void> {
    return invoke<void>("coder_retry", { threadId });
  }

  /** Resolve a pending approval and continue the run. */
  approve(
    threadId: string,
    callId: string,
    approve: boolean,
    remember = false,
    editedPattern?: string,
  ): Promise<void> {
    return invoke<void>("coder_approve", {
      threadId,
      callId,
      approve,
      remember,
      editedPattern: editedPattern ?? null,
    });
  }

  /** Cancel an in-flight run for a thread. */
  stop(threadId: string): Promise<boolean> {
    return invoke<boolean>("coder_stop", { threadId });
  }

  /** Thread ids with active agent loops. */
  listRunning(): Promise<string[]> {
    return invoke<string[]>("coder_list_running");
  }

  getMode(): Promise<PermissionMode> {
    return invoke<PermissionMode>("coder_get_mode");
  }

  setMode(mode: PermissionMode): Promise<void> {
    return invoke<void>("coder_set_mode", { mode });
  }

  listRules(): Promise<PermissionRule[]> {
    return invoke<PermissionRule[]>("coder_list_rules");
  }

  addRule(rule: PermissionRule): Promise<void> {
    return invoke<void>("coder_add_rule", { rule });
  }

  removeRule(tool: string, pattern: string): Promise<void> {
    return invoke<void>("coder_remove_rule", { tool, pattern });
  }

  // ---- change review (Cursor-style) ---------------------------------

  listChanges(threadId?: string): Promise<FileChange[]> {
    return invoke<FileChange[]>("coder_list_changes", {
      threadId: threadId ?? null,
    });
  }

  acceptChange(changeId: string): Promise<void> {
    return invoke<void>("coder_accept_change", { changeId });
  }

  rejectChange(changeId: string): Promise<void> {
    return invoke<void>("coder_reject_change", { changeId });
  }

  /** Accept or reject a single hunk; rewrites the file on disk. */
  setHunk(changeId: string, hunkIndex: number, accepted: boolean): Promise<void> {
    return invoke<void>("coder_set_hunk", { changeId, hunkIndex, accepted });
  }

  /** Overwrite the file with manually edited content. */
  modifyChange(changeId: string, content: string): Promise<void> {
    return invoke<void>("coder_modify_change", { changeId, content });
  }

  /** A change was created or updated. */
  onChange(cb: (p: { change: FileChange }) => void): Promise<UnlistenFn> {
    return listen<{ change: FileChange }>("coder://change", (e) => cb(e.payload));
  }

  // ---- live streaming events (emitted during a run) ------------------

  /** An incremental assistant text token during generation. */
  onToken(
    cb: (p: { thread_id: string; delta: string }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ thread_id: string; delta: string }>(
      "coder://token",
      (e) => cb(e.payload),
    );
  }

  /** A message (assistant or tool) was appended to a thread. */
  onMessage(
    cb: (p: { thread_id: string; message: ChatMessage }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ thread_id: string; message: ChatMessage }>(
      "coder://message",
      (e) => cb(e.payload),
    );
  }

  /** The run paused awaiting an approval decision. */
  onPending(
    cb: (p: { thread_id: string; pending: PendingApproval }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ thread_id: string; pending: PendingApproval }>(
      "coder://pending",
      (e) => cb(e.payload),
    );
  }

  /** Thread title updated (fallback or smart). */
  onTitle(
    cb: (p: ThreadTitleEvent) => void,
  ): Promise<UnlistenFn> {
    return listen<ThreadTitleEvent>("coder://title", (e) => cb(e.payload));
  }

  /** Context window usage updated for a thread. */
  onContextUsage(
    cb: (p: {
      thread_id: string;
      context_usage: ContextUsage;
      llm_usage?: LlmUsage | null;
    }) => void,
  ): Promise<UnlistenFn> {
    return listen<{
      thread_id: string;
      context_usage: ContextUsage;
      llm_usage?: LlmUsage | null;
    }>("coder://context-usage", (e) => cb(e.payload));
  }

  /** Fetch current context usage from agent-platform for a thread. */
  getContextUsage(threadId: string): Promise<ContextUsage | null> {
    return invoke<ContextUsage | null>("coder_get_context_usage", { threadId });
  }

  /** Git branch / working-tree diff stats for a workspace path. */
  getGitDiffStats(workspaceRoot: string): Promise<GitDiffStats> {
    return invoke<GitDiffStats>("coder_get_git_diff_stats", {
      workspaceRoot,
    });
  }

  /** The run finished. */
  onDone(
    cb: (p: CoderDoneEvent) => void,
  ): Promise<UnlistenFn> {
    return listen<CoderDoneEvent>("coder://done", (e) => cb(e.payload));
  }

  /** Run started or stopped for a thread. */
  onRunning(
    cb: (p: { thread_id: string; running: boolean }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ thread_id: string; running: boolean }>(
      "coder://running",
      (e) => cb(e.payload),
    );
  }

  /** Run failed with an error. */
  onError(
    cb: (p: { thread_id: string; error: string }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ thread_id: string; error: string }>(
      "coder://error",
      (e) => cb(e.payload),
    );
  }
}

export const coderService = new CoderService();
