/**
 * Global coder session state — survives page navigation and thread switches.
 * Event listeners register once and route updates per thread id.
 */

import { aiProviderService } from "$lib/domains/ai/services/aiProviderService.js";
import { coderService } from "../services/coderService.js";
import { getToolCallDisplay, formatFailedResult, isToolResultFailure } from "../utils/toolCallDisplay.js";
import { withMessageTimestamps } from "../utils/messageTimestamps.js";
import type {
  ChatMessage,
  CoderSubAgent,
  CoderThread,
  CoderThreadKind,
  SpawnSubAgentTask,
  FileChange,
  PendingApproval,
  PermissionRule,
  CoderAgentMode,
  PermissionMode,
  ThreadTitleEvent,
} from "../types.js";
import { summaryToThread } from "../types.js";
import type {
  ContextUsage,
  LlmUsage,
  ProviderType,
} from "$lib/domains/ai/types/index.js";
import {
  fallbackTitleFromMessage,
  isPlaceholderTitle,
  reconcileThreadTitle,
} from "$lib/domains/chat/title.js";
import { loadChatCatalogPrefs } from "$lib/domains/ai/utils/chatCatalogPrefs.js";
import {
  abortAgentCommands,
  executeAgentCommand,
  mirrorCommandOutput,
} from "../services/coderTerminalCoordinator.js";
import { coderTerminalStore } from "./coderTerminalStore.svelte.js";
import { coderWorkspaceStore } from "./coderWorkspaceStore.svelte.js";
import { inferEffectiveMode } from "../config/agentModes.js";
import { coderUi } from "./coderUi.svelte.js";

export type SubAgentNotificationEvent =
  | {
      type: "finished";
      coordinatorId: string;
      subAgent: CoderSubAgent;
    }
  | {
      type: "complete";
      coordinatorId: string;
      subAgents: CoderSubAgent[];
    }
  | {
      type: "spawned";
      coordinatorId: string;
      count: number;
    };

export type SubAgentNotificationHandler = (event: SubAgentNotificationEvent) => void;

export interface ThreadRuntime {
  messages: ChatMessage[];
  streamingText: string;
  /** Seconds waited on the current LLM step with no output yet (0 = not waiting). */
  waitingSeconds: number;
  running: boolean;
  pending: PendingApproval | null;
  error: string | null;
  canRetry: boolean;
  contextUsage: ContextUsage | null;
  llmUsage: LlmUsage | null;
  lastRetry:
    | null
    | { type: "send" }
    | {
        type: "approve";
        callId: string;
        approve: boolean;
        remember: boolean;
        editedPattern?: string;
      };
  draftInput: string;
  /** Follow-ups queued while the agent is running (Cursor-style). */
  messageQueue: string[];
  subAgents: CoderSubAgent[];
  multitaskMode: boolean;
  /** Inferred sub-mode when agentMode is "auto" (UI subtitle only). */
  effectiveMode: CoderAgentMode | null;
  /** Set when viewing a sub-agent thread opened from a coordinator. */
  parentCoordinatorId?: string | null;
}

function emptyRuntime(): ThreadRuntime {
  return {
    messages: [],
    streamingText: "",
    waitingSeconds: 0,
    running: false,
    pending: null,
    error: null,
    canRetry: false,
    contextUsage: null,
    llmUsage: null,
    lastRetry: null,
    draftInput: "",
    messageQueue: [],
    subAgents: [],
    multitaskMode: false,
    effectiveMode: null,
  };
}

function summarizeTool(tool: string, args: Record<string, unknown>): string {
  const { label, detail } = getToolCallDisplay(tool, args);
  return detail ? `${label}: ${detail}` : label;
}

function suggestedRule(tool: string, args: Record<string, unknown>): string {
  switch (tool) {
    case "run_command": {
      const cmd = String(args.command ?? "");
      return cmd.split(/\s+/)[0] ?? "";
    }
    case "write_file":
    case "edit_file": {
      const p = String(args.path ?? "");
      const parts = p.replace(/\\/g, "/").split("/");
      if (parts.length > 1) {
        parts.pop();
        return `${parts.join("/")}/*`;
      }
      return "*";
    }
    default:
      return "";
  }
}

function parseToolArgs(raw: string): Record<string, unknown> {
  try {
    return JSON.parse(raw || "{}") as Record<string, unknown>;
  } catch {
    return {};
  }
}

function commandForToolCall(
  messages: ChatMessage[],
  toolCallId: string,
): string | null {
  for (const m of messages) {
    if (m.role !== "assistant" || !m.tool_calls?.length) continue;
    const call = m.tool_calls.find((c) => c.id === toolCallId);
    if (!call || call.function.name !== "run_command") continue;
    const args = parseToolArgs(call.function.arguments);
    return String(args.command ?? "");
  }
  return null;
}

function appendRunCommandToTerminal(
  threadId: string,
  messages: ChatMessage[],
  toolCallId: string,
  content: string,
) {
  const command = commandForToolCall(messages, toolCallId);
  if (!command) return;
  const failed = isToolResultFailure("run_command", content);
  const output = failed ? formatFailedResult("run_command", content) : content;
  mirrorCommandOutput(threadId, null, command, output, failed);
}

/** Rebuild pending approval from open tool calls in persisted messages. */
export function inferPending(messages: ChatMessage[]): PendingApproval | null {
  const answered = new Set(
    messages
      .filter((m) => m.role === "tool" && m.tool_call_id)
      .map((m) => m.tool_call_id as string),
  );

  for (let i = messages.length - 1; i >= 0; i--) {
    const m = messages[i];
    if (m.role !== "assistant" || !m.tool_calls?.length) continue;
    const open = m.tool_calls.find((c) => !answered.has(c.id));
    if (!open) return null;
    const args = parseToolArgs(open.function.arguments);
    return {
      call_id: open.id,
      tool: open.function.name,
      arguments: args,
      suggested_rule: suggestedRule(open.function.name, args),
      summary: summarizeTool(open.function.name, args),
    };
  }
  return null;
}

/** True when the latest user message has no assistant reply yet. */
export function lastUserTurnNeedsReply(messages: ChatMessage[]): boolean {
  let lastUserIdx = -1;
  for (let i = messages.length - 1; i >= 0; i--) {
    if (messages[i].role === "user") {
      lastUserIdx = i;
      break;
    }
  }
  if (lastUserIdx < 0) return false;

  for (let i = lastUserIdx + 1; i < messages.length; i++) {
    const m = messages[i];
    if (m.role !== "assistant") continue;
    const content = (m.content ?? "").trim();
    if (content || (m.tool_calls?.length ?? 0) > 0) return false;
  }
  return true;
}

class CoderSessionState {
  initialized = false;
  /** Shared init promise so concurrent callers wait for full hydration. */
  private initPromise: Promise<void> | null = null;
  threads = $state<CoderThread[]>([]);
  /** True while the session list is being fetched from the backend. */
  threadsLoading = $state(true);
  /** Bumped whenever the session list changes (drives sidebar reactivity). */
  threadsRevision = $state(0);
  activeThreadId = $state<string | null>(null);
  thread = $state<CoderThread | null>(null);
  workspaceRoot = $state("");
  agentMode = $state<CoderAgentMode>("auto");
  permissionMode = $state<PermissionMode>("review");
  rules = $state<PermissionRule[]>([]);
  changes = $state<FileChange[]>([]);
  runningThreadIds = $state<Set<string>>(new Set());
  selectedProvider = $state<ProviderType | null>("AgentPlatform");
  selectedBackendProvider = $state<string | null>(null);
  selectedModel = $state<string | null>(null);
  multitaskMode = $state(false);
  /** Coordinator thread when drilling into a sub-agent session. */
  parentCoordinatorId = $state<string | null>(null);
  private subAgentNotificationHandler: SubAgentNotificationHandler | null = null;
  /** Bumped when any per-thread runtime changes (drives UI reactivity). */
  runtimeRevision = $state(0);
  /** Session terminal panel visibility (shared across threads). */
  terminalOpen = $state(false);

  /** Per-thread ephemeral UI + streaming state. */
  private runtimes = $state<Record<string, ThreadRuntime>>({});
  /** Threads with user-edited titles — ignore incoming title events. */
  private userRenamedThreadIds = new Set<string>();
  /** Tool calls already executed via frontend terminal delegation. */
  private delegatedCommandIds = new Set<string>();

  private touchRuntime() {
    this.runtimeRevision += 1;
  }

  get activeRuntime(): ThreadRuntime {
    const id = this.activeThreadId;
    if (!id) return emptyRuntime();
    return this.runtimes[id] ?? emptyRuntime();
  }

  /** Read-only lookup — never creates entries (safe inside $derived / templates). */
  peekRuntime(threadId: string): ThreadRuntime {
    return this.runtimes[threadId] ?? emptyRuntime();
  }

  runtimeFor(threadId: string): ThreadRuntime {
    if (!this.runtimes[threadId]) {
      this.runtimes[threadId] = emptyRuntime();
    }
    return this.runtimes[threadId];
  }

  isThreadRunning(threadId: string): boolean {
    return this.runningThreadIds.has(threadId);
  }

  isCoordinatorThread(thread?: CoderThread | null): boolean {
    return thread?.thread_kind === "coordinator";
  }

  /** Show retry when the run failed or the last user turn got no assistant reply. */
  shouldShowRetry(): boolean {
    if (!this.thread) return false;
    const rt = this.activeRuntime;
    if (rt.running || rt.pending) return false;
    return rt.canRetry || lastUserTurnNeedsReply(rt.messages);
  }

  private updateRetryFromMessages(rt: ThreadRuntime, messages: ChatMessage[]) {
    if (rt.running || rt.pending) return;
    if (lastUserTurnNeedsReply(messages)) {
      rt.canRetry = true;
      if (!rt.lastRetry) rt.lastRetry = { type: "send" };
    }
  }

  async ensureInit(): Promise<void> {
    if (!this.initPromise) {
      this.initPromise = this.runInit().catch((err) => {
        this.initPromise = null;
        throw err;
      });
    }
    await this.initPromise;
  }

  private async runInit(): Promise<void> {
    try {
      this.agentMode = await coderService.getMode();
    } catch (e) {
      console.error("coder: getMode failed", e);
    }

    try {
      this.permissionMode = await coderService.getPermissionMode();
    } catch (e) {
      console.error("coder: getPermissionMode failed", e);
    }

    try {
      this.rules = await coderService.listRules();
    } catch (e) {
      console.error("coder: listRules failed", e);
    }

    // Load sessions first so the sidebar works even if provider config fails.
    await this.refreshThreads();

    try {
      await this.syncRunningThreads();
    } catch (e) {
      console.error("coder: syncRunningThreads failed", e);
    }

    void this.initProviderDefaults();

    await coderService.onToken(({ thread_id, delta }) => {
      const rt = this.runtimeFor(thread_id);
      rt.streamingText += delta;
      rt.waitingSeconds = 0;
      this.touchRuntime();
    });

    await coderService.onHeartbeat(({ thread_id, waited_seconds }) => {
      const rt = this.runtimeFor(thread_id);
      rt.waitingSeconds = waited_seconds;
      this.touchRuntime();
    });

    await coderService.onContextUsage(({ thread_id, context_usage, llm_usage }) => {
      const rt = this.runtimeFor(thread_id);
      rt.contextUsage = context_usage;
      if (llm_usage) rt.llmUsage = llm_usage;
      this.touchRuntime();
    });

    await coderService.onTitle(({ thread_id, title }) => {
      this.applyThreadTitle(thread_id, title);
    });

    await coderService.onMessage(({ thread_id, message }) => {
      const rt = this.runtimeFor(thread_id);
      const stampedMessage: ChatMessage = {
        ...message,
        timestamp: message.timestamp ?? new Date().toISOString(),
      };
      if (stampedMessage.role === "assistant") rt.streamingText = "";
      rt.waitingSeconds = 0;
      const last = rt.messages[rt.messages.length - 1];
      const dup =
        last &&
        last.role === stampedMessage.role &&
        last.content === stampedMessage.content &&
        JSON.stringify(last.tool_calls) === JSON.stringify(stampedMessage.tool_calls);
      if (!dup) {
        rt.messages = [...rt.messages, stampedMessage];
      }
      if (
        stampedMessage.role === "tool" &&
        stampedMessage.tool_call_id &&
        stampedMessage.content != null
      ) {
        if (this.delegatedCommandIds.has(stampedMessage.tool_call_id)) {
          this.delegatedCommandIds.delete(stampedMessage.tool_call_id);
        } else {
          appendRunCommandToTerminal(
            thread_id,
            rt.messages,
            stampedMessage.tool_call_id,
            stampedMessage.content,
          );
        }
        if (this.terminalOpen && thread_id) {
          const root =
            this.thread?.workspace_root ?? this.workspaceRoot;
          if (root) {
            const tab = coderTerminalStore.ensureDefault(thread_id, root);
            coderWorkspaceStore.openTerminal(thread_id, tab.id, tab.label);
          }
        }
      }
      this.touchRuntime();
      if (thread_id === this.activeThreadId) {
        this.thread = { ...this.thread!, messages: rt.messages };
      }
    });

    await coderService.onPending(({ thread_id, pending }) => {
      const rt = this.runtimeFor(thread_id);
      rt.pending = pending;
      rt.running = false;
      rt.streamingText = "";
      rt.waitingSeconds = 0;
      this.touchRuntime();
      this.setRunning(thread_id, false);
      void this.syncThreadFromBackend(thread_id);
    });

    await coderService.onDone(({ thread_id, exhausted, cancelled, title }) => {
      const rt = this.runtimeFor(thread_id);
      rt.running = false;
      rt.streamingText = "";
      rt.waitingSeconds = 0;
      rt.pending = null;
      rt.canRetry = false;
      rt.lastRetry = null;
      if (exhausted) {
        rt.error = "Agent hit the max iteration limit.";
        rt.canRetry = true;
        rt.lastRetry = { type: "send" };
      } else if (cancelled) {
        rt.error = null;
      }
      this.touchRuntime();
      this.setRunning(thread_id, false);
      if (title) {
        this.applyThreadTitle(thread_id, title);
      }
      void this.syncThreadFromBackend(thread_id).then(() => {
        const synced = this.runtimeFor(thread_id);
        if (!cancelled && !exhausted) {
          this.updateRetryFromMessages(synced, synced.messages);
        }
        this.touchRuntime();
        if (!cancelled && !exhausted) {
          void this.processQueue(thread_id);
        }
      });
    });

    await coderService.onRunning(({ thread_id, running }) => {
      this.runtimeFor(thread_id).running = running;
      this.touchRuntime();
      this.setRunning(thread_id, running);
      if (running) {
        void this.ensureThreadListed(thread_id);
      }
    });

    await coderService.onError(({ thread_id, error }) => {
      const rt = this.runtimeFor(thread_id);
      rt.error = error;
      rt.running = false;
      rt.waitingSeconds = 0;
      rt.canRetry = true;
      rt.lastRetry = { type: "send" };
      this.touchRuntime();
      this.setRunning(thread_id, false);
      void this.syncThreadFromBackend(thread_id);
    });

    await coderService.onChange(({ change }) => {
      const i = this.changes.findIndex((c) => c.id === change.id);
      if (i >= 0) this.changes[i] = change;
      else this.changes = [...this.changes, change];
    });

    await coderService.onRunCommand(async (req) => {
      this.terminalOpen = true;
      const tab = coderTerminalStore.ensureDefault(
        req.thread_id,
        req.workspace_root,
      );
      coderWorkspaceStore.openTerminal(req.thread_id, tab.id, tab.label);
      this.delegatedCommandIds.add(req.call_id);
      try {
        const result = await executeAgentCommand({
          threadId: req.thread_id,
          callId: req.call_id,
          command: req.command,
          workspaceRoot: req.workspace_root,
          terminalId: req.terminal_id,
        });
        await coderService.submitCommandResult(
          req.thread_id,
          req.call_id,
          result,
        );
      } catch (e) {
        await coderService.submitCommandResult(
          req.thread_id,
          req.call_id,
          `Error: ${e}`,
        );
      }
    });

    await coderService.onListTerminals(async (req) => {
      const list = coderTerminalStore.listForAgent(req.thread_id);
      await coderService.submitTerminalList(
        req.thread_id,
        req.call_id,
        JSON.stringify(list),
      );
    });

    await coderService.onSubAgentStarted(({ coordinator_id, subagent }) => {
      this.upsertSubAgent(coordinator_id, subagent);
    });

    await coderService.onSubAgentProgress(({ coordinator_id, subagent }) => {
      this.upsertSubAgent(coordinator_id, subagent);
    });

    await coderService.onSubAgentFinished(({ coordinator_id, subagent }) => {
      if (subagent) this.upsertSubAgent(coordinator_id, subagent);
    });

    await coderService.onMultitaskComplete(({ coordinator_id, subagents }) => {
      const rt = this.runtimeFor(coordinator_id);
      rt.subAgents = subagents;
      this.touchRuntime();
      if (this.shouldNotifyForCoordinator(coordinator_id)) {
        this.emitSubAgentNotification({
          type: "complete",
          coordinatorId: coordinator_id,
          subAgents: subagents,
        });
      }
    });

    this.initialized = true;
  }

  /** Keep provider/model on the session singleton so parent binds stay stable. */
  private async initProviderDefaults(): Promise<void> {
    const saved = loadChatCatalogPrefs();

    if (!this.selectedProvider) {
      this.selectedProvider =
        (await aiProviderService.getDefaultProvider()) ?? "AgentPlatform";
    }
    if (!this.selectedBackendProvider && saved?.backendProvider) {
      this.selectedBackendProvider = saved.backendProvider;
    }
    if (!this.selectedModel && saved?.model) {
      this.selectedModel = saved.model;
    }
    if (!this.selectedModel && this.selectedProvider) {
      try {
        const config = await aiProviderService.getProviderConfig(
          this.selectedProvider,
        );
        this.selectedModel = config.model || null;
      } catch {
        /* provider config may be unavailable offline */
      }
    }
  }

  /** Persist current UI model/provider selection on the active thread. */
  private async syncThreadLlmConfig(threadId: string): Promise<void> {
    await coderService.updateThreadModel(
      threadId,
      this.selectedModel,
      this.selectedBackendProvider,
    );
    if (this.thread?.id === threadId) {
      this.thread = {
        ...this.thread,
        model: this.selectedModel,
        llm_provider: this.selectedBackendProvider,
      };
      this.upsertThread(this.thread);
    }
  }

  setSubAgentNotificationHandler(handler: SubAgentNotificationHandler | null) {
    this.subAgentNotificationHandler = handler;
  }

  private shouldNotifyForCoordinator(
    coordinatorId: string,
    subAgent?: CoderSubAgent | null,
  ): boolean {
    if (this.activeThreadId === coordinatorId) return false;
    if (subAgent && this.activeThreadId === subAgent.child_thread_id) return false;
    return true;
  }

  private emitSubAgentNotification(event: SubAgentNotificationEvent) {
    this.subAgentNotificationHandler?.(event);
  }

  private upsertSubAgent(coordinatorId: string, subAgent: CoderSubAgent) {
    const rt = this.runtimeFor(coordinatorId);
    const idx = rt.subAgents.findIndex((item) => item.id === subAgent.id);
    const previous = idx >= 0 ? rt.subAgents[idx] : null;
    if (idx >= 0) {
      rt.subAgents[idx] = subAgent;
      rt.subAgents = [...rt.subAgents];
    } else {
      rt.subAgents = [subAgent, ...rt.subAgents];
    }
    rt.multitaskMode = true;
    this.touchRuntime();

    const wasActive =
      previous?.status === "running" || previous?.status === "pending";
    const isDone = ["completed", "failed", "cancelled"].includes(subAgent.status);
    if (
      wasActive &&
      isDone &&
      this.shouldNotifyForCoordinator(coordinatorId, subAgent)
    ) {
      this.emitSubAgentNotification({
        type: "finished",
        coordinatorId,
        subAgent,
      });
    }
  }

  async loadSubAgents(threadId: string) {
    const rt = this.runtimeFor(threadId);
    rt.subAgents = await coderService.listSubAgents(threadId);
    rt.multitaskMode = true;
    this.touchRuntime();
  }

  async setMultitaskMode(enabled: boolean) {
    this.multitaskMode = enabled;
    if (!this.thread) return;
    const nextKind: CoderThreadKind = enabled ? "coordinator" : "session";
    await coderService.setThreadKind(this.thread.id, nextKind);
    this.thread = { ...this.thread, thread_kind: nextKind };
    this.upsertThread(this.thread);
    const rt = this.runtimeFor(this.thread.id);
    rt.multitaskMode = enabled;
    if (enabled) {
      await this.loadSubAgents(this.thread.id);
    } else {
      rt.subAgents = [];
    }
    this.touchRuntime();
  }

  private bumpThreads() {
    this.threadsRevision += 1;
  }

  private stubThread(threadId: string): CoderThread {
    if (this.thread?.id === threadId) {
      return { ...this.thread };
    }
    const rt = this.runtimes[threadId];
    return {
      id: threadId,
      title: "Running session",
      workspace_root: this.workspaceRoot,
      messages: rt?.messages ?? [],
      created_at: "",
      updated_at: new Date().toISOString(),
    };
  }

  private upsertThread(t: CoderThread) {
    const idx = this.threads.findIndex((x) => x.id === t.id);
    if (idx >= 0) {
      this.threads[idx] = t;
      this.threads = [...this.threads];
    } else {
      this.threads = [t, ...this.threads];
    }
    this.bumpThreads();
  }

  /** Ensure a thread appears in the sidebar list. */
  async ensureThreadListed(threadId: string): Promise<void> {
    const cached = this.threads.find((t) => t.id === threadId);
    if (cached) return;

    try {
      const t = await coderService.getThread(threadId);
      if (t) {
        this.upsertThread(t);
        return;
      }
    } catch {
      /* full thread payload may be too large — fall through */
    }

    try {
      const summaries = await coderService.listThreadSummaries();
      const summary = summaries.find((s) => s.id === threadId);
      if (summary) {
        this.upsertThread(summaryToThread(summary));
        return;
      }
    } catch {
      /* ignore */
    }

    this.upsertThread(this.stubThread(threadId));
  }

  /** Refresh running-thread flags and make sure each appears in the sidebar. */
  async syncRunningThreads(): Promise<void> {
    const running = await coderService.listRunning();
    this.runningThreadIds = new Set(running);
    for (const id of running) {
      this.runtimeFor(id).running = true;
      await this.ensureThreadListed(id);
    }
    this.touchRuntime();
  }

  /**
   * On page load: attach to the most recent running session so the UI matches
   * background agent work.
   */
  async attachToRunningThread(): Promise<string | null> {
    await this.ensureInit();
    const running = await coderService.listRunning();
    if (running.length === 0) return null;

    await this.syncRunningThreads();

    const runningSet = new Set(running);
    const candidates = this.threads
      .filter((t) => runningSet.has(t.id))
      .sort((a, b) => b.updated_at.localeCompare(a.updated_at));
    const pick = candidates[0]?.id ?? running[0];
    await this.selectThread(pick);
    return pick;
  }

  private setRunning(threadId: string, running: boolean) {
    const next = new Set(this.runningThreadIds);
    if (running) {
      next.add(threadId);
      if (!this.threads.some((t) => t.id === threadId)) {
        this.upsertThread(this.stubThread(threadId));
      }
      void this.ensureThreadListed(threadId);
    } else {
      next.delete(threadId);
    }
    this.runningThreadIds = next;
    this.bumpThreads();
    this.touchRuntime();
  }

  private async refreshContextUsage(threadId: string) {
    try {
      const usage = await coderService.getContextUsage(threadId);
      if (usage) {
        this.runtimeFor(threadId).contextUsage = usage;
        this.touchRuntime();
      }
    } catch {
      /* platform thread may not exist yet */
    }
  }

  private applyThreadTitle(threadId: string, title: string) {
    if (this.userRenamedThreadIds.has(threadId)) return;

    const idx = this.threads.findIndex((x) => x.id === threadId);
    const current = idx >= 0 ? this.threads[idx].title : this.thread?.title;
    const next = reconcileThreadTitle(current, title);

    if (idx >= 0) {
      this.threads[idx] = { ...this.threads[idx], title: next };
      this.threads = [...this.threads];
      this.bumpThreads();
    } else if (this.activeThreadId === threadId && this.thread) {
      this.upsertThread({ ...this.thread, title: next });
    }
    if (this.activeThreadId === threadId && this.thread) {
      this.thread = { ...this.thread, title: next };
    }
  }

  private async syncThreadFromBackend(threadId: string) {
    const t = await coderService.getThread(threadId);
    if (!t) {
      console.warn(`coder: syncThreadFromBackend — thread ${threadId} not found`);
      return;
    }
    const rt = this.runtimeFor(threadId);
    const stamped = withMessageTimestamps(t.messages, rt.messages, t.updated_at);
    rt.messages = stamped;
    if (!rt.pending) rt.pending = inferPending(stamped);
    this.touchRuntime();
    if (this.activeThreadId === threadId) {
      this.thread = { ...t, messages: stamped };
    }
    const idx = this.threads.findIndex((x) => x.id === threadId);
    if (idx >= 0) {
      this.threads[idx] = { ...t, messages: stamped };
      this.threads = [...this.threads];
      this.bumpThreads();
    } else {
      this.upsertThread({ ...t, messages: stamped });
    }
    this.updateRetryFromMessages(rt, stamped);
    void this.refreshContextUsage(threadId);
  }

  async refreshThreads() {
    this.threadsLoading = true;
    try {
      const summaries = await coderService.listThreadSummaries();
      this.threads = summaries.map(summaryToThread);
    } catch (summaryErr) {
      console.warn("coder: list summaries failed, falling back to full threads", summaryErr);
      try {
        this.threads = await coderService.listThreads();
      } catch (err) {
        console.error("coder: list threads failed", err);
      }
    } finally {
      this.threadsLoading = false;
    }
    this.bumpThreads();
    void this.refreshCoordinatorSubAgents();
  }

  private async refreshCoordinatorSubAgents() {
    const coordinators = this.threads.filter((t) => t.thread_kind === "coordinator");
    await Promise.all(
      coordinators.map((t) =>
        this.loadSubAgents(t.id).catch((err) => {
          console.warn(`coder: load sub-agents for ${t.id} failed`, err);
        }),
      ),
    );
  }

  subAgentSummaryFor(threadId: string): { running: number; total: number } {
    const rt = this.peekRuntime(threadId);
    const running = rt.subAgents.filter(
      (s) => s.status === "running" || s.status === "pending",
    ).length;
    return { running, total: rt.subAgents.length };
  }

  async refreshChanges(threadId?: string) {
    this.changes = await coderService.listChanges(threadId);
  }

  private findCoordinatorForSubAgent(childThreadId: string): string | null {
    for (const [coordId, rt] of Object.entries(this.runtimes)) {
      if (rt.subAgents.some((s) => s.child_thread_id === childThreadId)) {
        return coordId;
      }
    }
    return null;
  }

  private async resolveCoordinatorForSubAgent(childThreadId: string): Promise<string | null> {
    const cached = this.findCoordinatorForSubAgent(childThreadId);
    if (cached) return cached;

    for (const summary of this.threads) {
      if (summary.thread_kind !== "coordinator") continue;
      await this.loadSubAgents(summary.id);
      const coordId = this.findCoordinatorForSubAgent(childThreadId);
      if (coordId) return coordId;
    }
    return null;
  }

  async openSubAgent(childThreadId: string, coordinatorId?: string) {
    const coordId =
      coordinatorId ??
      this.findCoordinatorForSubAgent(childThreadId) ??
      (await this.resolveCoordinatorForSubAgent(childThreadId));
    if (coordId) {
      this.parentCoordinatorId = coordId;
      this.runtimeFor(childThreadId).parentCoordinatorId = coordId;
    }
    await this.selectThread(childThreadId);
  }

  async selectThread(id: string) {
    await this.ensureInit();
    const t = await coderService.getThread(id);
    if (!t) return;
    this.activeThreadId = id;
    this.upsertThread(t);
    this.workspaceRoot = t.workspace_root;
    this.selectedModel = t.model ?? this.selectedModel;
    this.selectedBackendProvider = t.llm_provider ?? this.selectedBackendProvider;
    this.multitaskMode = t.thread_kind === "coordinator";

    if (t.thread_kind === "sub-agent") {
      const rt = this.runtimeFor(id);
      this.parentCoordinatorId =
        rt.parentCoordinatorId ?? (await this.resolveCoordinatorForSubAgent(id));
      if (this.parentCoordinatorId) {
        rt.parentCoordinatorId = this.parentCoordinatorId;
      }
    } else {
      this.parentCoordinatorId = null;
    }

    const rt = this.runtimeFor(id);
    const stamped = withMessageTimestamps(t.messages, rt.messages, t.updated_at);
    rt.messages = stamped;
    this.thread = { ...t, messages: stamped };
    if (!rt.pending) rt.pending = inferPending(stamped);
    rt.multitaskMode = this.multitaskMode;
    const running = await coderService.listRunning();
    rt.running = running.includes(id);
    this.runningThreadIds = new Set(running);
    this.updateRetryFromMessages(rt, rt.messages);
    this.touchRuntime();

    await this.refreshChanges(id);
    if (this.multitaskMode) {
      await this.loadSubAgents(id);
    } else {
      rt.subAgents = [];
    }
    void this.refreshContextUsage(id);
  }

  newSession() {
    this.activeThreadId = null;
    this.thread = null;
    this.workspaceRoot = "";
    this.changes = [];
    this.multitaskMode = false;
  }

  async createThreadIfNeeded(): Promise<CoderThread> {
    if (this.thread) {
      this.upsertThread(this.thread);
      if (
        (this.selectedModel && this.selectedModel !== this.thread.model) ||
        (this.selectedBackendProvider &&
          this.selectedBackendProvider !== this.thread.llm_provider)
      ) {
        await this.syncThreadLlmConfig(this.thread.id);
      }
      return this.thread;
    }
    const t = await coderService.createThread(
      this.workspaceRoot.trim(),
      this.selectedModel || undefined,
      this.selectedBackendProvider || undefined,
      this.agentMode === "multitask" ? "coordinator" : "session",
      coderUi.activeProjectIdAsNumber(),
    );
    this.thread = t;
    this.activeThreadId = t.id;
    this.upsertThread(t);
    const rt = this.runtimeFor(t.id);
    rt.messages = withMessageTimestamps(t.messages, rt.messages, t.updated_at);
    rt.multitaskMode = t.thread_kind === "coordinator";
    this.touchRuntime();
    await this.refreshThreads();
    return t;
  }

  private shouldQueue(threadId: string): boolean {
    const rt = this.runtimeFor(threadId);
    return rt.running || this.isThreadRunning(threadId);
  }

  removeFromQueue(threadId: string, index: number) {
    const rt = this.runtimeFor(threadId);
    rt.messageQueue = rt.messageQueue.filter((_, i) => i !== index);
    this.touchRuntime();
  }

  private async processQueue(threadId: string) {
    const rt = this.runtimeFor(threadId);
    if (
      rt.running ||
      this.isThreadRunning(threadId) ||
      rt.pending ||
      rt.messageQueue.length === 0
    ) {
      return;
    }
    const next = rt.messageQueue[0];
    rt.messageQueue = rt.messageQueue.slice(1);
    this.touchRuntime();
    await this.sendNow(next, threadId);
  }

  private looksLikeMultitaskInput(text: string): boolean {
    if (this.extractIssueUrls(text).length >= 2) return true;
    return (this.parseMultitaskTasks(text)?.length ?? 0) >= 2;
  }

  private async ensureCoordinatorThread(t: CoderThread): Promise<CoderThread> {
    if (t.thread_kind === "coordinator") return t;
    if (t.thread_kind === "sub-agent") return t;

    await coderService.setThreadKind(t.id, "coordinator");
    const updated: CoderThread = { ...t, thread_kind: "coordinator" };
    if (this.thread?.id === t.id) {
      this.thread = updated;
    }
    this.upsertThread(updated);
    const rt = this.runtimeFor(t.id);
    rt.multitaskMode = true;
    if (this.activeThreadId === t.id) {
      this.multitaskMode = true;
    }
    this.touchRuntime();
    return updated;
  }

  private async trySpawnMultitask(
    t: CoderThread,
    text: string,
  ): Promise<{ count: number; coordinatorId: string } | null> {
    if (t.thread_kind === "sub-agent" || !this.looksLikeMultitaskInput(text)) {
      return null;
    }

    const coord = await this.ensureCoordinatorThread(t);
    if (coord.thread_kind !== "coordinator") return null;

    const issueUrls = this.extractIssueUrls(text);
    if (issueUrls.length >= 2) {
      await this.spawnMultitask(coord.id, { issueUrls });
      return { count: issueUrls.length, coordinatorId: coord.id };
    }

    const tasks = this.parseMultitaskTasks(text);
    if (tasks && tasks.length >= 2) {
      await this.spawnMultitask(coord.id, { tasks });
      return { count: tasks.length, coordinatorId: coord.id };
    }

    return null;
  }

  async spawnBackgroundTask(text: string): Promise<boolean> {
    await this.ensureInit();
    const trimmed = text.trim();
    if (!trimmed) return false;

    if (!this.workspaceRoot.trim()) {
      if (this.activeThreadId) {
        const rt = this.runtimeFor(this.activeThreadId);
        rt.error = "Set a workspace folder first.";
        this.touchRuntime();
      }
      return false;
    }

    const t = await this.createThreadIfNeeded();
    this.activeThreadId = t.id;
    this.thread = t;
    this.upsertThread(t);
    await this.syncThreadLlmConfig(t.id);

    const coord = await this.ensureCoordinatorThread(t);
    if (coord.thread_kind !== "coordinator") return false;

    const title = trimmed.length > 72 ? `${trimmed.slice(0, 69)}…` : trimmed;
    await this.spawnMultitask(coord.id, {
      tasks: [{ title, prompt: trimmed }],
    });

    if (this.activeThreadId === coord.id) {
      this.runtimeFor(coord.id).draftInput = "";
    }

    this.emitSubAgentNotification({
      type: "spawned",
      coordinatorId: coord.id,
      count: 1,
    });
    return true;
  }

  private extractIssueUrls(text: string): string[] {
    const matches = text.match(/https:\/\/github\.com\/[^/\s]+\/[^/\s]+\/issues\/\d+/g);
    return matches ? [...new Set(matches)] : [];
  }

  private parseMultitaskTasks(text: string): SpawnSubAgentTask[] | null {
    const stripped = text.replace(
      /https:\/\/github\.com\/[^/\s]+\/[^/\s]+\/issues\/\d+/g,
      "",
    );
    const blocks = stripped.includes("\n---")
      ? stripped.split(/\n---\n?/).map((line) => line.trim()).filter(Boolean)
      : stripped
          .split("\n")
          .map((line) => line.trim())
          .filter(Boolean);
    if (blocks.length < 2) return null;

    return blocks.map((block) => {
      const title = block.length > 72 ? `${block.slice(0, 69)}…` : block;
      return { title, prompt: block };
    });
  }

  private async spawnMultitask(
    threadId: string,
    request: { tasks?: SpawnSubAgentTask[]; issueUrls?: string[] },
  ) {
    const rt = this.runtimeFor(threadId);
    rt.error = null;
    await coderService.multitaskSpawn({
      coordinatorThreadId: threadId,
      tasks: request.tasks ?? [],
      issueUrls: request.issueUrls,
    });
    await this.loadSubAgents(threadId);
    this.touchRuntime();
  }

  async send(text: string) {
    await this.ensureInit();
    const trimmed = text.trim();
    if (!trimmed) return;

    if (this.thread && this.shouldQueue(this.thread.id)) {
      if (this.looksLikeMultitaskInput(trimmed)) {
        await this.sendNow(trimmed);
        return;
      }
      const rt = this.runtimeFor(this.thread.id);
      rt.messageQueue = [...rt.messageQueue, trimmed];
      rt.draftInput = "";
      this.touchRuntime();
      return;
    }

    await this.sendNow(trimmed);
  }

  private async sendNow(text: string, explicitThreadId?: string) {
    await this.ensureInit();
    const trimmed = text.trim();
    if (!trimmed) return;

    let t: CoderThread | null = null;
    if (explicitThreadId) {
      t = (await coderService.getThread(explicitThreadId)) ?? null;
    }
    if (!t) {
      if (!this.workspaceRoot.trim()) {
        if (this.activeThreadId) {
          const rt = this.runtimeFor(this.activeThreadId);
          rt.error = "Set a workspace folder first.";
          this.touchRuntime();
        }
        return;
      }
      t = await this.createThreadIfNeeded();
    }

    this.activeThreadId = t.id;
    this.thread = t;
    this.upsertThread(t);

    await this.syncThreadLlmConfig(t.id);

    const spawnResult = await this.trySpawnMultitask(t, trimmed);
    if (spawnResult) {
      if (this.activeThreadId === t.id) {
        this.runtimeFor(t.id).draftInput = "";
      }
      this.emitSubAgentNotification({
        type: "spawned",
        coordinatorId: spawnResult.coordinatorId,
        count: spawnResult.count,
      });
      return;
    }

    const rt = this.runtimeFor(t.id);
    rt.effectiveMode = inferEffectiveMode(trimmed, this.agentMode);
    rt.error = null;
    rt.canRetry = false;
    rt.lastRetry = { type: "send" };
    rt.running = true;
    rt.streamingText = "";
    rt.waitingSeconds = 0;
    this.touchRuntime();
    this.setRunning(t.id, true);

    try {
      const isFirst =
        t.messages.filter((m) => m.role === "user").length === 0 &&
        isPlaceholderTitle(t.title);
      if (isFirst) {
        const fb = fallbackTitleFromMessage(trimmed, "New session");
        this.applyThreadTitle(t.id, fb);
      }
      if (this.activeThreadId === t.id) {
        rt.draftInput = "";
      }
      await coderService.send(t.id, trimmed);
    } catch (e) {
      rt.error = String(e);
      rt.canRetry = true;
      rt.running = false;
      this.touchRuntime();
      this.setRunning(t.id, false);
    }
  }

  queuedCountFor(threadId: string): number {
    return this.peekRuntime(threadId).messageQueue.length;
  }

  async stop() {
    if (!this.activeThreadId) return;
    const threadId = this.activeThreadId;
    abortAgentCommands(threadId);
    const rt = this.runtimeFor(threadId);
    rt.running = false;
    rt.streamingText = "";
    rt.waitingSeconds = 0;
    rt.error = null;
    rt.pending = null;
    this.touchRuntime();
    this.setRunning(threadId, false);
    await coderService.stop(threadId);
  }

  async retry() {
    if (!this.thread || !this.shouldShowRetry()) return;
    const rt = this.activeRuntime;
    const threadId = this.thread.id;
    rt.error = null;
    rt.running = true;
    rt.streamingText = "";
    rt.waitingSeconds = 0;
    rt.canRetry = false;
    this.touchRuntime();
    this.setRunning(threadId, true);
    try {
      if (rt.lastRetry?.type === "approve") {
        const { callId, approve, remember, editedPattern } = rt.lastRetry;
        await coderService.approve(
          threadId,
          callId,
          approve,
          remember,
          editedPattern,
        );
        if (remember) this.rules = await coderService.listRules();
      } else {
        await coderService.retry(threadId);
        await this.syncThreadFromBackend(threadId);
      }
      rt.canRetry = false;
      rt.lastRetry = null;
    } catch (e) {
      rt.error = String(e);
      rt.canRetry = true;
      rt.lastRetry = { type: "send" };
      rt.running = false;
      this.touchRuntime();
      this.setRunning(threadId, false);
    }
  }

  async editMessage(messageIndex: number, content: string) {
    if (!this.thread || !this.activeThreadId) return;
    const trimmed = content.trim();
    if (!trimmed) return;

    const threadId = this.thread.id;
    const rt = this.runtimeFor(threadId);
    if (rt.running || rt.pending) return;

    rt.error = null;
    rt.canRetry = false;
    rt.lastRetry = { type: "send" };
    rt.running = true;
    rt.streamingText = "";
    rt.waitingSeconds = 0;
    rt.messageQueue = [];
    this.touchRuntime();
    this.setRunning(threadId, true);

    try {
      await coderService.editMessage(threadId, messageIndex, trimmed);
      await this.syncThreadFromBackend(threadId);
    } catch (e) {
      rt.error = String(e);
      rt.canRetry = true;
      rt.lastRetry = { type: "send" };
      rt.running = false;
      this.touchRuntime();
      this.setRunning(threadId, false);
    }
  }

  async decide(
    approve: boolean,
    remember: boolean,
    editedPattern?: string,
  ) {
    if (!this.thread || !this.activeRuntime.pending) return;
    const rt = this.activeRuntime;
    const savedPending = rt.pending;
    if (!savedPending) return;
    const threadId = this.thread.id;
    const callId = savedPending.call_id;

    rt.running = true;
    rt.streamingText = "";
    rt.waitingSeconds = 0;
    rt.error = null;
    rt.canRetry = false;
    rt.lastRetry = {
      type: "approve",
      callId,
      approve,
      remember,
      editedPattern,
    };
    rt.pending = null;
    this.touchRuntime();

    try {
      await coderService.approve(
        threadId,
        callId,
        approve,
        remember,
        editedPattern,
        savedPending?.tool,
        savedPending?.arguments as Record<string, unknown> | undefined,
      );
      rt.canRetry = false;
      rt.lastRetry = null;
      if (remember) this.rules = await coderService.listRules();
    } catch (e) {
      rt.error = String(e);
      rt.pending = savedPending;
      rt.canRetry = true;
      rt.running = false;
      this.touchRuntime();
    }
  }

  async removeThread(t: CoderThread) {
    await coderService.deleteThread(t.id);
    delete this.runtimes[t.id];
    coderTerminalStore.clearThread(t.id);
    this.touchRuntime();
    if (this.activeThreadId === t.id) this.newSession();
    await this.refreshThreads();
  }

  async cancelSubAgent(subAgentId: string) {
    if (!this.thread) return;
    const subAgents = await coderService.multitaskCancel({
      coordinatorThreadId: this.thread.id,
      subAgentId,
    });
    const rt = this.runtimeFor(this.thread.id);
    rt.subAgents = subAgents;
    this.touchRuntime();
  }

  async cleanupSubAgents(subAgentIds: string[] = [], force = false) {
    if (!this.thread) return;
    const removed = await coderService.multitaskCleanup({
      coordinatorThreadId: this.thread.id,
      subAgentIds,
      force,
    });
    const removedIds = new Set(removed.map((item) => item.id));
    const rt = this.runtimeFor(this.thread.id);
    rt.subAgents = rt.subAgents.filter((item) => !removedIds.has(item.id));
    this.touchRuntime();
  }

  async changeAgentMode(next: CoderAgentMode) {
    this.agentMode = next;
    await coderService.setMode(next);

    if (next === "multitask") {
      await this.setMultitaskMode(true);
    } else if (this.thread?.thread_kind === "coordinator") {
      await this.setMultitaskMode(false);
    }
  }

  async changePermissionMode(next: PermissionMode) {
    this.permissionMode = next;
    await coderService.setPermissionMode(next);
  }

  async removeRule(r: PermissionRule) {
    await coderService.removeRule(r.tool, r.pattern);
    this.rules = await coderService.listRules();
  }

  async handleModelChange(model: string) {
    this.selectedModel = model;
    if (!this.thread) return;
    try {
      await this.syncThreadLlmConfig(this.thread.id);
    } catch (e) {
      console.error("Failed to update session model:", e);
    }
  }

  async handleBackendProviderChange(providerId: string) {
    this.selectedBackendProvider = providerId;
    if (!this.thread) return;
    try {
      await this.syncThreadLlmConfig(this.thread.id);
    } catch (e) {
      console.error("Failed to update session provider:", e);
    }
  }
}

export const coderSession = new CoderSessionState();
