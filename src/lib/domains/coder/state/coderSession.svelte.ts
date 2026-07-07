/**
 * Global coder session state — survives page navigation and thread switches.
 * Event listeners register once and route updates per thread id.
 */

import { coderService } from "../services/coderService.js";
import { getToolCallDisplay } from "../utils/toolCallDisplay.js";
import type {
  ChatMessage,
  CoderThread,
  FileChange,
  PendingApproval,
  PermissionMode,
  PermissionRule,
  ThreadTitleEvent,
} from "../types.js";
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

export interface ThreadRuntime {
  messages: ChatMessage[];
  streamingText: string;
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
}

function emptyRuntime(): ThreadRuntime {
  return {
    messages: [],
    streamingText: "",
    running: false,
    pending: null,
    error: null,
    canRetry: false,
    contextUsage: null,
    llmUsage: null,
    lastRetry: null,
    draftInput: "",
    messageQueue: [],
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

class CoderSessionState {
  initialized = false;
  /** Shared init promise so concurrent callers wait for full hydration. */
  private initPromise: Promise<void> | null = null;
  threads = $state<CoderThread[]>([]);
  activeThreadId = $state<string | null>(null);
  thread = $state<CoderThread | null>(null);
  workspaceRoot = $state("");
  mode = $state<PermissionMode>("review");
  rules = $state<PermissionRule[]>([]);
  changes = $state<FileChange[]>([]);
  runningThreadIds = $state<Set<string>>(new Set());
  selectedProvider = $state<ProviderType | null>(null);
  selectedModel = $state<string | null>(null);
  /** Bumped when any per-thread runtime changes (drives UI reactivity). */
  runtimeRevision = $state(0);

  /** Per-thread ephemeral UI + streaming state. */
  private runtimes = $state<Record<string, ThreadRuntime>>({});
  /** Threads with user-edited titles — ignore incoming title events. */
  private userRenamedThreadIds = new Set<string>();

  private touchRuntime() {
    this.runtimeRevision += 1;
  }

  get activeRuntime(): ThreadRuntime {
    const id = this.activeThreadId;
    if (!id) return emptyRuntime();
    if (!this.runtimes[id]) {
      this.runtimes[id] = emptyRuntime();
    }
    return this.runtimes[id];
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

  async ensureInit(): Promise<void> {
    if (!this.initPromise) {
      this.initPromise = this.runInit();
    }
    await this.initPromise;
  }

  private async runInit(): Promise<void> {
    this.mode = await coderService.getMode();
    this.rules = await coderService.listRules();
    await this.refreshThreads();

    const running = await coderService.listRunning();
    this.runningThreadIds = new Set(running);
    for (const id of running) {
      this.runtimeFor(id).running = true;
    }

    await coderService.onToken(({ thread_id, delta }) => {
      const rt = this.runtimeFor(thread_id);
      rt.streamingText += delta;
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
      if (message.role === "assistant") rt.streamingText = "";
      const last = rt.messages[rt.messages.length - 1];
      const dup =
        last &&
        last.role === message.role &&
        last.content === message.content &&
        JSON.stringify(last.tool_calls) === JSON.stringify(message.tool_calls);
      if (!dup) {
        rt.messages = [...rt.messages, message];
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
      this.touchRuntime();
      this.setRunning(thread_id, false);
    });

    await coderService.onDone(({ thread_id, exhausted, cancelled, title }) => {
      const rt = this.runtimeFor(thread_id);
      rt.running = false;
      rt.streamingText = "";
      rt.pending = null;
      rt.canRetry = false;
      rt.lastRetry = null;
      if (exhausted) {
        rt.error = "Agent hit the max iteration limit.";
      } else if (cancelled) {
        rt.error = null;
      }
      this.touchRuntime();
      this.setRunning(thread_id, false);
      if (title) {
        this.applyThreadTitle(thread_id, title);
      }
      void this.syncThreadFromBackend(thread_id);
      if (!cancelled && !exhausted) {
        void this.processQueue(thread_id);
      }
    });

    await coderService.onRunning(({ thread_id, running }) => {
      this.runtimeFor(thread_id).running = running;
      this.touchRuntime();
      this.setRunning(thread_id, running);
    });

    await coderService.onError(({ thread_id, error }) => {
      const rt = this.runtimeFor(thread_id);
      rt.error = error;
      rt.running = false;
      rt.canRetry = true;
      this.touchRuntime();
      this.setRunning(thread_id, false);
    });

    await coderService.onChange(({ change }) => {
      const i = this.changes.findIndex((c) => c.id === change.id);
      if (i >= 0) this.changes[i] = change;
      else this.changes = [...this.changes, change];
    });

    this.initialized = true;
  }

  private upsertThread(t: CoderThread) {
    const idx = this.threads.findIndex((x) => x.id === t.id);
    if (idx >= 0) {
      this.threads[idx] = t;
      this.threads = [...this.threads];
    } else {
      this.threads = [t, ...this.threads];
    }
  }

  private setRunning(threadId: string, running: boolean) {
    const next = new Set(this.runningThreadIds);
    if (running) next.add(threadId);
    else next.delete(threadId);
    this.runningThreadIds = next;
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
    } else if (this.activeThreadId === threadId && this.thread) {
      this.upsertThread({ ...this.thread, title: next });
    }
    if (this.activeThreadId === threadId && this.thread) {
      this.thread = { ...this.thread, title: next };
    }
  }

  private async syncThreadFromBackend(threadId: string) {
    const t = await coderService.getThread(threadId);
    if (!t) return;
    const rt = this.runtimeFor(threadId);
    rt.messages = t.messages;
    if (!rt.pending) rt.pending = inferPending(t.messages);
    this.touchRuntime();
    if (this.activeThreadId === threadId) {
      this.thread = t;
    }
    const idx = this.threads.findIndex((x) => x.id === threadId);
    if (idx >= 0) {
      this.threads[idx] = t;
      this.threads = [...this.threads];
    } else {
      this.upsertThread(t);
    }
    void this.refreshContextUsage(threadId);
  }

  async refreshThreads() {
    this.threads = await coderService.listThreads();
  }

  async refreshChanges(threadId?: string) {
    this.changes = await coderService.listChanges(threadId);
  }

  async selectThread(id: string) {
    await this.ensureInit();
    const t = await coderService.getThread(id);
    if (!t) return;
    this.activeThreadId = id;
    this.thread = t;
    this.upsertThread(t);
    this.workspaceRoot = t.workspace_root;
    if (t.model) this.selectedModel = t.model;

    const rt = this.runtimeFor(id);
    rt.messages = t.messages;
    if (!rt.pending) rt.pending = inferPending(t.messages);
    const running = await coderService.listRunning();
    rt.running = running.includes(id);
    this.runningThreadIds = new Set(running);
    rt.error = null;
    this.touchRuntime();

    await this.refreshChanges(id);
    void this.refreshContextUsage(id);
  }

  newSession() {
    this.activeThreadId = null;
    this.thread = null;
    this.workspaceRoot = "";
    this.changes = [];
  }

  async createThreadIfNeeded(): Promise<CoderThread> {
    if (this.thread) {
      if (this.selectedModel && this.selectedModel !== this.thread.model) {
        await coderService.updateThreadModel(this.thread.id, this.selectedModel);
        this.thread = { ...this.thread, model: this.selectedModel };
      }
      return this.thread;
    }
    const t = await coderService.createThread(
      this.workspaceRoot.trim(),
      this.selectedModel || undefined,
    );
    this.thread = t;
    this.activeThreadId = t.id;
    this.upsertThread(t);
    const rt = this.runtimeFor(t.id);
    rt.messages = t.messages;
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

  async send(text: string) {
    await this.ensureInit();
    const trimmed = text.trim();
    if (!trimmed) return;

    if (this.thread && this.shouldQueue(this.thread.id)) {
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
        this.activeRuntime.error = "Set a workspace folder first.";
        return;
      }
      t = await this.createThreadIfNeeded();
    }

    const rt = this.runtimeFor(t.id);
    rt.error = null;
    rt.canRetry = false;
    rt.lastRetry = { type: "send" };
    rt.running = true;
    rt.streamingText = "";
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
    return this.runtimeFor(threadId).messageQueue.length;
  }

  async stop() {
    if (!this.activeThreadId) return;
    const threadId = this.activeThreadId;
    const rt = this.runtimeFor(threadId);
    rt.running = false;
    rt.streamingText = "";
    rt.error = null;
    this.touchRuntime();
    this.setRunning(threadId, false);
    await coderService.stop(threadId);
  }

  async retry() {
    if (!this.thread || !this.activeRuntime.canRetry) return;
    const rt = this.activeRuntime;
    const threadId = this.thread.id;
    rt.error = null;
    rt.running = true;
    rt.streamingText = "";
    this.touchRuntime();
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
      rt.running = false;
      this.touchRuntime();
    }
  }

  async decide(
    approve: boolean,
    remember: boolean,
    editedPattern?: string,
  ) {
    if (!this.thread || !this.activeRuntime.pending) return;
    const rt = this.activeRuntime;
    const threadId = this.thread.id;
    const callId = rt.pending.call_id;
    const savedPending = rt.pending;

    rt.running = true;
    rt.streamingText = "";
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
    this.touchRuntime();
    if (this.activeThreadId === t.id) this.newSession();
    await this.refreshThreads();
  }

  async changeMode(next: PermissionMode) {
    this.mode = next;
    await coderService.setMode(next);
  }

  async removeRule(r: PermissionRule) {
    await coderService.removeRule(r.tool, r.pattern);
    this.rules = await coderService.listRules();
  }

  async handleModelChange(model: string) {
    if (!this.thread) return;
    await coderService.updateThreadModel(this.thread.id, model);
    this.thread = { ...this.thread, model };
  }
}

export const coderSession = new CoderSessionState();
