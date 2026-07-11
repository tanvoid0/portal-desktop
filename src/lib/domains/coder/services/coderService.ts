import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ContextUsage, LlmUsage } from "$lib/domains/ai/types/index.js";
import type {
  ChatMessage,
  CoderDoneEvent,
  CoderSubAgent,
  CoderThread,
  CoderThreadKind,
  CoderThreadSummary,
  FileChange,
  GitDiffStats,
  GitFileChange,
  GitCommitDraft,
  MultitaskCancelRequest,
  MultitaskCleanupRequest,
  MultitaskSpawnRequest,
  PendingApproval,
  PermissionMode,
  PermissionRule,
  ThreadTitleEvent,
  RunCommandEvent,
  ListTerminalsEvent,
} from "../types.js";

/**
 * Thin wrapper over the `coder_*` Tauri commands. The agent loop, tool
 * execution, and permission gating all live in Rust; this only marshals calls.
 */
export class CoderService {
  createThread(
    workspaceRoot: string,
    model?: string,
    llmProvider?: string,
    threadKind?: CoderThreadKind | null,
    projectId?: number | null,
  ): Promise<CoderThread> {
    return invoke<CoderThread>("coder_create_thread", {
      workspaceRoot,
      model: model ?? null,
      llmProvider: llmProvider ?? null,
      threadKind: threadKind ?? null,
      projectId: projectId ?? null,
    });
  }

  listThreads(): Promise<CoderThread[]> {
    return invoke<CoderThread[]>("coder_list_threads");
  }

  listThreadSummaries(): Promise<CoderThreadSummary[]> {
    return invoke<CoderThreadSummary[]>("coder_list_thread_summaries");
  }

  getThread(threadId: string): Promise<CoderThread | null> {
    return invoke<CoderThread | null>("coder_get_thread", { threadId });
  }

  deleteThread(threadId: string): Promise<boolean> {
    return invoke<boolean>("coder_delete_thread", { threadId });
  }

  updateThreadModel(
    threadId: string,
    model: string | null,
    llmProvider?: string | null,
  ): Promise<void> {
    return invoke<void>("coder_update_thread_model", {
      threadId,
      model,
      llmProvider: llmProvider ?? null,
    });
  }

  setThreadKind(threadId: string, threadKind: CoderThreadKind): Promise<void> {
    return invoke<void>("coder_set_thread_kind", { threadId, threadKind });
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
    rememberTool?: string,
    rememberArgs?: Record<string, unknown>,
  ): Promise<void> {
    return invoke<void>("coder_approve", {
      threadId,
      callId,
      approve,
      remember,
      editedPattern: editedPattern ?? null,
      rememberTool: rememberTool ?? null,
      rememberArgs: rememberArgs ?? null,
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

  /** Per-file git working-tree diffs for a workspace path. */
  listGitChanges(workspaceRoot: string): Promise<GitFileChange[]> {
    return invoke<GitFileChange[]>("coder_list_git_changes", {
      workspaceRoot,
    });
  }

  /** Review working tree and suggest a commit message (optional AI). */
  prepareGitCommit(
    workspaceRoot: string,
    useAi = true,
  ): Promise<GitCommitDraft> {
    return invoke<GitCommitDraft>("coder_prepare_git_commit", {
      workspaceRoot,
      useAi,
    });
  }

  /** Stage all changes and commit with the given title and summary. */
  gitCommit(
    workspaceRoot: string,
    title: string,
    summary?: string | null,
  ): Promise<string> {
    return invoke<string>("coder_git_commit", {
      workspaceRoot,
      title,
      summary: summary ?? null,
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

  /** Backend requests frontend terminal execution for run_command. */
  onRunCommand(cb: (p: RunCommandEvent) => void): Promise<UnlistenFn> {
    return listen<RunCommandEvent>("coder://run_command", (e) =>
      cb(e.payload),
    );
  }

  /** Backend requests the list of session terminals. */
  onListTerminals(cb: (p: ListTerminalsEvent) => void): Promise<UnlistenFn> {
    return listen<ListTerminalsEvent>("coder://list_terminals", (e) =>
      cb(e.payload),
    );
  }

  submitCommandResult(
    threadId: string,
    callId: string,
    result: string,
  ): Promise<void> {
    return invoke<void>("coder_submit_command_result", {
      threadId,
      callId,
      result,
    });
  }

  submitTerminalList(
    threadId: string,
    callId: string,
    listJson: string,
  ): Promise<void> {
    return invoke<void>("coder_submit_terminal_list", {
      threadId,
      callId,
      listJson: listJson,
    });
  }

  multitaskSpawn(request: MultitaskSpawnRequest): Promise<CoderSubAgent[]> {
    return invoke<CoderSubAgent[]>("coder_multitask_spawn", { request });
  }

  listSubAgents(coordinatorThreadId: string): Promise<CoderSubAgent[]> {
    return invoke<CoderSubAgent[]>("coder_multitask_list", { coordinatorThreadId });
  }

  multitaskCancel(request: MultitaskCancelRequest): Promise<CoderSubAgent[]> {
    return invoke<CoderSubAgent[]>("coder_multitask_cancel", { request });
  }

  multitaskCleanup(request: MultitaskCleanupRequest): Promise<CoderSubAgent[]> {
    return invoke<CoderSubAgent[]>("coder_multitask_cleanup", { request });
  }

  onSubAgentStarted(
    cb: (p: { coordinator_id: string; subagent: CoderSubAgent }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ coordinator_id: string; subagent: CoderSubAgent }>(
      "coder://subagent-started",
      (e) => cb(e.payload),
    );
  }

  onSubAgentProgress(
    cb: (p: { coordinator_id: string; subagent: CoderSubAgent }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ coordinator_id: string; subagent: CoderSubAgent }>(
      "coder://subagent-progress",
      (e) => cb(e.payload),
    );
  }

  onSubAgentFinished(
    cb: (p: { coordinator_id: string; subagent: CoderSubAgent | null }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ coordinator_id: string; subagent: CoderSubAgent | null }>(
      "coder://subagent-finished",
      (e) => cb(e.payload),
    );
  }

  onMultitaskComplete(
    cb: (p: { coordinator_id: string; subagents: CoderSubAgent[] }) => void,
  ): Promise<UnlistenFn> {
    return listen<{ coordinator_id: string; subagents: CoderSubAgent[] }>(
      "coder://multitask-complete",
      (e) => cb(e.payload),
    );
  }
}

export const coderService = new CoderService();
