import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ChatMessage,
  CoderRunResult,
  CoderThread,
  FileChange,
  PendingApproval,
  PermissionMode,
  PermissionRule,
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

  /** Send a user message and run until the loop pauses or finishes. */
  send(threadId: string, message: string): Promise<CoderRunResult> {
    return invoke<CoderRunResult>("coder_send", { threadId, message });
  }

  /** Resolve a pending approval and continue the run. */
  approve(
    threadId: string,
    callId: string,
    approve: boolean,
    remember = false,
    editedPattern?: string,
  ): Promise<CoderRunResult> {
    return invoke<CoderRunResult>("coder_approve", {
      threadId,
      callId,
      approve,
      remember,
      editedPattern: editedPattern ?? null,
    });
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

  /** The run finished. */
  onDone(
    cb: (p: {
      thread_id: string;
      final_text: string | null;
      exhausted: boolean;
    }) => void,
  ): Promise<UnlistenFn> {
    return listen<{
      thread_id: string;
      final_text: string | null;
      exhausted: boolean;
    }>("coder://done", (e) => cb(e.payload));
  }
}

export const coderService = new CoderService();
