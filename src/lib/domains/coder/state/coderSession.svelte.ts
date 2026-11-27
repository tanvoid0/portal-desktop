/**
 * Global coder session state — survives page navigation and thread switches. View state
 * only: what a thread says (messages, status, approvals, queue) is `threadStore`'s view of
 * the hub's log. Sub-agents, file-change bodies and the context gauge are not in that log
 * yet, so their listeners and fetches stay here.
 */

import { aiProviderService } from '$lib/domains/ai/services/aiProviderService.js';
import { coderService } from '../services/coderService.js';
import { threadStore } from '$lib/domains/threads/state/threadStore.svelte';
import type { ThreadView } from '$lib/domains/threads/utils/reducer';
import type {
  CoderSubAgent,
  CoderThread,
  CoderThreadKind,
  SpawnSubAgentTask,
  FileChange,
  PermissionRule,
  CoderAgentMode,
  PermissionMode,
  ThreadTitleEvent,
} from '../types.js';
import { summaryToThread } from '../types.js';
import type { ContextUsage, LlmUsage, ProviderType } from '$lib/domains/ai/types/index.js';
import {
  fallbackTitleFromMessage,
  isPlaceholderTitle,
  reconcileThreadTitle,
} from '$lib/domains/chat/title.js';
import { loadChatCatalogPrefs } from '$lib/domains/ai/utils/chatCatalogPrefs.js';
import { abortAgentCommands } from '../services/coderTerminalCoordinator.js';
import { startHostResponder } from '../services/hostResponder.js';
import { coderTerminalStore } from './coderTerminalStore.svelte.js';
import { inferEffectiveMode } from '../config/agentModes.js';
import { coderUi } from './coderUi.svelte.js';
import { toast } from '$lib/utils/toast';

/** Human-readable name for a coder engine id. */
function engineLabel(providerId: string): string {
  return providerId.charAt(0).toUpperCase() + providerId.slice(1);
}

export type SubAgentNotificationEvent =
  | {
      type: 'finished';
      coordinatorId: string;
      subAgent: CoderSubAgent;
    }
  | {
      type: 'complete';
      coordinatorId: string;
      subAgents: CoderSubAgent[];
    }
  | {
      type: 'spawned';
      coordinatorId: string;
      count: number;
    };

export type SubAgentNotificationHandler = (event: SubAgentNotificationEvent) => void;

/** Per-thread view state; the thread's own content is `threadStore.view(id)`. */
export interface ThreadRuntime {
  /** Context window gauge, fetched (it is not in the log). */
  contextUsage: ContextUsage | null;
  /** Last LLM step's tokens and cost; only the legacy event carries it. */
  llmUsage: LlmUsage | null;
  draftInput: string;
  subAgents: CoderSubAgent[];
  multitaskMode: boolean;
  /** Inferred sub-mode when agentMode is "auto" (UI subtitle only). */
  effectiveMode: CoderAgentMode | null;
  /** Set when viewing a sub-agent thread opened from a coordinator. */
  parentCoordinatorId?: string | null;
}

function emptyRuntime(): ThreadRuntime {
  return {
    contextUsage: null,
    llmUsage: null,
    draftInput: '',
    subAgents: [],
    multitaskMode: false,
    effectiveMode: null,
  };
}

const PROVIDER_KEY = 'portal.coder.backendProvider';

/** `local`, `device:<id>` or `remote` (see `Caller::actor`) as something a person can read. */
export function actorLabel(by: string): string {
  if (by === 'local') return 'this computer';
  if (by === 'remote') return 'a remote device';
  return by.replace(/^device:/, 'device ');
}

export function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function loadStoredProvider(): string | null {
  try {
    // The removed Claude Code (CLI) engine may still be stored from an old build.
    const p = localStorage.getItem(PROVIDER_KEY);
    return p === 'claude-cli' ? null : p;
  } catch {
    return null;
  }
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
  workspaceRoot = $state('');
  agentMode = $state<CoderAgentMode>('auto');
  permissionMode = $state<PermissionMode>('review');
  rules = $state<PermissionRule[]>([]);
  changes = $state<FileChange[]>([]);
  selectedProvider = $state<ProviderType | null>('AgentPlatform');
  selectedBackendProvider = $state<string | null>(loadStoredProvider());
  selectedModel = $state<string | null>(null);
  multitaskMode = $state(false);
  /** Coordinator thread when drilling into a sub-agent session. */
  parentCoordinatorId = $state<string | null>(null);
  private subAgentNotificationHandler: SubAgentNotificationHandler | null = null;
  /** Bumped when any per-thread runtime changes (drives UI reactivity). */
  runtimeRevision = $state(0);
  /** Session terminal panel visibility (shared across threads). */
  terminalOpen = $state(false);
  /** Composer text to pre-fill on the next fresh session (set by "Open in
   *  Coder" from another page; consumed once by CoderPanel on mount). */
  pendingPrompt = $state<string | null>(null);

  /** Per-thread view state. */
  private runtimes = $state<Record<string, ThreadRuntime>>({});
  /** Threads with user-edited titles — ignore incoming title events. */
  private userRenamedThreadIds = new Set<string>();
  /** Approvals whose decision is in flight, so a double click is one command. */
  private deciding = new Set<string>();

  private touchRuntime() {
    this.runtimeRevision += 1;
  }

  /** The active thread's live view from the hub's log; null until its snapshot lands. */
  get view(): ThreadView | null {
    return this.activeThreadId ? threadStore.view(this.activeThreadId) : null;
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

  isCoordinatorThread(thread?: CoderThread | null): boolean {
    return thread?.thread_kind === 'coordinator';
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
      console.error('coder: getMode failed', e);
    }

    try {
      this.permissionMode = await coderService.getPermissionMode();
    } catch (e) {
      console.error('coder: getPermissionMode failed', e);
    }

    try {
      this.rules = await coderService.listRules();
    } catch (e) {
      console.error('coder: listRules failed', e);
    }

    // Load sessions first so the sidebar works even if provider config fails.
    await this.refreshThreads();

    void this.initProviderDefaults();

    await coderService.onContextUsage(({ thread_id, context_usage, llm_usage }) => {
      const rt = this.runtimeFor(thread_id);
      rt.contextUsage = context_usage;
      if (llm_usage) rt.llmUsage = llm_usage;
      this.touchRuntime();
    });

    await coderService.onTitle(({ thread_id, title }) => {
      this.applyThreadTitle(thread_id, title);
    });

    await coderService.onChange(({ change }) => {
      const i = this.changes.findIndex((c) => c.id === change.id);
      if (i >= 0) this.changes[i] = change;
      else this.changes = [...this.changes, change];
    });

    void startHostResponder(() => {
      this.terminalOpen = true;
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
          type: 'complete',
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
      this.selectedProvider = (await aiProviderService.getDefaultProvider()) ?? 'AgentPlatform';
    }
    if (!this.selectedBackendProvider && saved?.backendProvider) {
      this.selectedBackendProvider = saved.backendProvider;
    }
    if (!this.selectedModel && saved?.model) {
      this.selectedModel = saved.model;
    }
    if (!this.selectedModel && this.selectedProvider) {
      try {
        const config = await aiProviderService.getProviderConfig(this.selectedProvider);
        this.selectedModel = config.model || null;
      } catch {
        /* provider config may be unavailable offline */
      }
    }
  }

  /**
   * What to persist for `t`: keep the model/engine it already has, and fill a
   * blank from the current selection. A thread keeps the engine it was created
   * with — see `syncThreadLlmConfig` and QA-085.
   */
  private llmConfigFor(t: CoderThread): { model: string | null; provider: string | null } {
    return {
      model: t.model || this.selectedModel,
      provider: t.llm_provider || this.selectedBackendProvider,
    };
  }

  /**
   * Persist the given model/provider on a thread.
   *
   * Callers must pass what they want written — never read the selector here.
   * Pushing the selector's engine onto an existing thread misattributes the
   * messages and `coder_file_changes` rows the previous engine produced
   * (QA-085). Use `llmConfigFor` to get back-fill-safe values.
   */
  private async syncThreadLlmConfig(
    threadId: string,
    model: string | null,
    provider: string | null
  ): Promise<void> {
    await coderService.updateThreadModel(threadId, model, provider);
    if (this.thread?.id === threadId) {
      this.thread = {
        ...this.thread,
        model,
        llm_provider: provider,
      };
      this.upsertThread(this.thread);
    }
  }

  setSubAgentNotificationHandler(handler: SubAgentNotificationHandler | null) {
    this.subAgentNotificationHandler = handler;
  }

  private shouldNotifyForCoordinator(
    coordinatorId: string,
    subAgent?: CoderSubAgent | null
  ): boolean {
    if (this.activeThreadId === coordinatorId) return false;
    if (subAgent && this.activeThreadId === subAgent.childThreadId) return false;
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

    const wasActive = previous?.status === 'running' || previous?.status === 'pending';
    const isDone = ['completed', 'failed', 'cancelled'].includes(subAgent.status);
    if (wasActive && isDone && this.shouldNotifyForCoordinator(coordinatorId, subAgent)) {
      this.emitSubAgentNotification({
        type: 'finished',
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
    const nextKind: CoderThreadKind = enabled ? 'coordinator' : 'session';
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
    return {
      id: threadId,
      title: 'Running session',
      workspace_root: this.workspaceRoot,
      model: null,
      thread_kind: 'session',
      messages: [],
      created_at: '',
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

  /** Re-read the thread row (title, updated_at, model) and the context gauge, e.g. when a turn ends. */
  async syncThread(threadId: string) {
    const t = await coderService.getThread(threadId);
    if (!t) return;
    if (this.activeThreadId === threadId) this.thread = t;
    this.upsertThread(t);
    void this.refreshContextUsage(threadId);
  }

  async refreshThreads() {
    this.threadsLoading = true;
    try {
      const summaries = await coderService.listThreadSummaries();
      this.threads = summaries.map(summaryToThread);
    } catch (summaryErr) {
      console.warn('coder: list summaries failed, falling back to full threads', summaryErr);
      try {
        this.threads = await coderService.listThreads();
      } catch (err) {
        console.error('coder: list threads failed', err);
      }
    } finally {
      this.threadsLoading = false;
    }
    this.bumpThreads();
    void this.refreshCoordinatorSubAgents();
  }

  private async refreshCoordinatorSubAgents() {
    const coordinators = this.threads.filter((t) => t.thread_kind === 'coordinator');
    await Promise.all(
      coordinators.map((t) =>
        this.loadSubAgents(t.id).catch((err) => {
          console.warn(`coder: load sub-agents for ${t.id} failed`, err);
        })
      )
    );
  }

  subAgentSummaryFor(threadId: string): { running: number; total: number } {
    const rt = this.peekRuntime(threadId);
    const running = rt.subAgents.filter(
      (s) => s.status === 'running' || s.status === 'pending'
    ).length;
    return { running, total: rt.subAgents.length };
  }

  async refreshChanges(threadId?: string) {
    this.changes = await coderService.listChanges(threadId);
  }

  private findCoordinatorForSubAgent(childThreadId: string): string | null {
    for (const [coordId, rt] of Object.entries(this.runtimes)) {
      if (rt.subAgents.some((s) => s.childThreadId === childThreadId)) {
        return coordId;
      }
    }
    return null;
  }

  private async resolveCoordinatorForSubAgent(childThreadId: string): Promise<string | null> {
    const cached = this.findCoordinatorForSubAgent(childThreadId);
    if (cached) return cached;

    for (const summary of this.threads) {
      if (summary.thread_kind !== 'coordinator') continue;
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
    if (t.llm_provider && t.llm_provider !== 'claude-cli') {
      this.selectedBackendProvider = t.llm_provider;
    }
    this.multitaskMode = t.thread_kind === 'coordinator';

    if (t.thread_kind === 'sub-agent') {
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
    this.thread = t;
    rt.multitaskMode = this.multitaskMode;
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
    this.workspaceRoot = '';
    this.changes = [];
    this.multitaskMode = false;
  }

  async createThreadIfNeeded(): Promise<CoderThread> {
    if (this.thread) {
      this.upsertThread(this.thread);
      // Back-fill only. A thread keeps the engine it was created with: its
      // messages and its `coder_file_changes` rows were produced by that
      // engine, and pushing the selector's value onto an existing thread
      // silently misattributes that work to an engine which never ran it
      // (QA-085). Switching engine is done by starting a new session.
      const { model, provider } = this.llmConfigFor(this.thread);
      if ((!this.thread.model || !this.thread.llm_provider) && (model || provider)) {
        await this.syncThreadLlmConfig(this.thread.id, model, provider);
      }
      return this.thread;
    }
    const t = await coderService.createThread(
      this.workspaceRoot.trim(),
      this.selectedModel || undefined,
      this.selectedBackendProvider || undefined,
      this.agentMode === 'multitask' ? 'coordinator' : 'session',
      coderUi.activeProjectIdAsNumber()
    );
    this.thread = t;
    this.activeThreadId = t.id;
    this.upsertThread(t);
    const rt = this.runtimeFor(t.id);
    rt.multitaskMode = t.thread_kind === 'coordinator';
    this.touchRuntime();
    await this.refreshThreads();
    return t;
  }

  private looksLikeMultitaskInput(text: string): boolean {
    if (this.extractIssueUrls(text).length >= 2) return true;
    // Line-splitting fan-out is explicit: only when Multitask mode is selected (QA-135).
    const multitask =
      this.agentMode === 'multitask' ||
      this.multitaskMode ||
      this.thread?.thread_kind === 'coordinator';
    if (!multitask) return false;
    return (this.parseMultitaskTasks(text)?.length ?? 0) >= 2;
  }

  private async ensureCoordinatorThread(t: CoderThread): Promise<CoderThread> {
    if (t.thread_kind === 'coordinator') return t;
    if (t.thread_kind === 'sub-agent') return t;

    await coderService.setThreadKind(t.id, 'coordinator');
    const updated: CoderThread = { ...t, thread_kind: 'coordinator' };
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
    text: string
  ): Promise<{ count: number; coordinatorId: string } | null> {
    if (t.thread_kind === 'sub-agent' || !this.looksLikeMultitaskInput(text)) {
      return null;
    }

    const coord = await this.ensureCoordinatorThread(t);
    if (coord.thread_kind !== 'coordinator') return null;

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

    if (!this.workspaceRoot.trim()) throw new Error('Set a workspace folder first.');

    const t = await this.createThreadIfNeeded();
    this.activeThreadId = t.id;
    this.thread = t;
    this.upsertThread(t);
    const cfg = this.llmConfigFor(t);
    await this.syncThreadLlmConfig(t.id, cfg.model, cfg.provider);

    const coord = await this.ensureCoordinatorThread(t);
    if (coord.thread_kind !== 'coordinator') return false;

    const title = trimmed.length > 72 ? `${trimmed.slice(0, 69)}…` : trimmed;
    await this.spawnMultitask(coord.id, {
      tasks: [{ title, prompt: trimmed }],
    });

    if (this.activeThreadId === coord.id) {
      this.runtimeFor(coord.id).draftInput = '';
    }

    this.emitSubAgentNotification({
      type: 'spawned',
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
    const stripped = text.replace(/https:\/\/github\.com\/[^/\s]+\/[^/\s]+\/issues\/\d+/g, '');
    const blocks = stripped.includes('\n---')
      ? stripped
          .split(/\n---\n?/)
          .map((line) => line.trim())
          .filter(Boolean)
      : stripped
          .split('\n')
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
    request: { tasks?: SpawnSubAgentTask[]; issueUrls?: string[] }
  ) {
    await coderService.multitaskSpawn({
      coordinatorThreadId: threadId,
      tasks: request.tasks ?? [],
      issueUrls: request.issueUrls,
    });
    await this.loadSubAgents(threadId);
    this.touchRuntime();
  }

  /**
   * Plain Send: a message while a turn is open is the hub's to queue (shown from the thread
   * state). Throws the hub's rejection so the caller can restore the draft.
   */
  async send(text: string) {
    await this.ensureInit();
    const trimmed = text.trim();
    if (!trimmed) return;

    if (!this.workspaceRoot.trim()) throw new Error('Set a workspace folder first.');
    const t = await this.createThreadIfNeeded();

    this.activeThreadId = t.id;
    this.thread = t;
    this.upsertThread(t);

    const cfg = this.llmConfigFor(t);
    await this.syncThreadLlmConfig(t.id, cfg.model, cfg.provider);

    const spawnResult = await this.trySpawnMultitask(t, trimmed);
    if (spawnResult) {
      if (this.activeThreadId === t.id) {
        this.runtimeFor(t.id).draftInput = '';
      }
      this.emitSubAgentNotification({
        type: 'spawned',
        coordinatorId: spawnResult.coordinatorId,
        count: spawnResult.count,
      });
      return;
    }

    const rt = this.runtimeFor(t.id);
    rt.effectiveMode = inferEffectiveMode(trimmed, this.agentMode);
    this.touchRuntime();

    const isFirst = !(this.view?.messages ?? []).some((m) => m.role === 'user');
    await coderService.send(t.id, trimmed);
    if (isFirst && isPlaceholderTitle(t.title)) {
      this.applyThreadTitle(t.id, fallbackTitleFromMessage(trimmed, 'New session'));
    }
    if (this.activeThreadId === t.id) rt.draftInput = '';
  }

  /** Writes other than `send` have no draft to give back: a failure is a toast. */
  private async write(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (e) {
      toast.error(errorText(e));
    }
  }

  stop() {
    const id = this.activeThreadId;
    if (!id) return Promise.resolve();
    abortAgentCommands(id);
    return this.write(() => coderService.stop(id));
  }

  /** `index` is the position in the view's queue. */
  async removeQueued(index: number) {
    const id = this.thread?.id;
    const q = this.view?.queue[index];
    if (id && q) await this.write(() => coderService.removeQueued(id, q.queue_id));
  }

  /** Resume a failed run, or answer a user turn nobody answered. */
  async retry() {
    const id = this.thread?.id;
    if (id && this.view?.canRetry) await this.write(() => coderService.retry(id));
  }

  /** `messageIndex` is the index into the view's messages, which is the hub's own. */
  async editMessage(messageIndex: number, content: string) {
    const id = this.thread?.id;
    const trimmed = content.trim();
    const status = this.view?.status;
    if (!id || !trimmed || !status || status === 'running' || status === 'waiting') return;
    await this.write(() => coderService.editMessage(id, messageIndex, trimmed));
  }

  /** Decide the first open approval. A loser of the first-wins race gets a notice, not an error. */
  async decide(approve: boolean, remember: boolean, editedPattern?: string) {
    const id = this.thread?.id;
    const call = this.view?.approvals[0];
    if (!id || !call || this.deciding.has(call.call_id)) return;
    this.deciding.add(call.call_id);
    try {
      await coderService.approve(id, call.call_id, approve, remember, editedPattern);
      if (remember) this.rules = await coderService.listRules();
    } catch (e) {
      if (!String(e).includes('already_resolved')) return void toast.error(errorText(e));
      const won = this.view?.recentlyResolved.find((r) => r.call_id === call.call_id);
      toast.info(
        won
          ? `Already ${won.allow ? 'approved' : 'denied'} by ${actorLabel(won.by)}`
          : 'Already decided on another device'
      );
    } finally {
      this.deciding.delete(call.call_id);
    }
  }

  async removeThread(t: CoderThread) {
    await this.removeThreads([t]);
  }

  async removeThreads(threads: CoderThread[]) {
    // One refusal (the backend will not delete a live run) must not stop the rest.
    const failed: { title: string; reason: string }[] = [];
    try {
      // ponytail: one invoke per thread; add a bulk command if histories reach hundreds.
      for (const t of threads) {
        try {
          await coderService.deleteThread(t.id);
        } catch (e) {
          failed.push({ title: t.title, reason: errorText(e) });
          continue;
        }
        delete this.runtimes[t.id];
        coderTerminalStore.clearThread(t.id);
        if (this.activeThreadId === t.id) this.newSession();
      }
    } finally {
      // Also after a partial failure, so the list shows what is really left.
      this.touchRuntime();
      await this.refreshThreads();
    }
    // Callers toast a thrown error.
    if (failed.length) {
      // One thread asked for: the bare reason, no "Deleted 0 of 1" wrapper.
      if (threads.length === 1) throw new Error(failed[0].reason);
      throw new Error(
        `Deleted ${threads.length - failed.length} of ${threads.length}; ${failed.map((f) => `"${f.title}": ${f.reason}`).join('; ')}`
      );
    }
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

    if (next === 'multitask') {
      await this.setMultitaskMode(true);
    } else if (this.thread?.thread_kind === 'coordinator') {
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
      // Changing model on the same engine is fine; keep the thread's engine.
      await this.syncThreadLlmConfig(
        this.thread.id,
        model,
        this.thread.llm_provider || this.selectedBackendProvider
      );
    } catch (e) {
      console.error('Failed to update session model:', e);
    }
  }

  async handleBackendProviderChange(providerId: string) {
    const previous = this.thread?.llm_provider ?? null;
    this.selectedBackendProvider = providerId;
    try {
      localStorage.setItem(PROVIDER_KEY, providerId);
    } catch {
      /* private mode */
    }
    // Never push the new engine onto an existing thread. Its messages and its
    // `coder_file_changes` rows were produced by the engine already recorded
    // on it, so relabelling attributes that work to an engine which never ran
    // (QA-085). The choice becomes the default for the next session instead —
    // say so, rather than leaving the selector quietly disagreeing with the
    // engine the next turn will actually use.
    if (previous && previous !== providerId) {
      toast.info(
        `This session stays on ${engineLabel(previous)}. Start a new session to use ${engineLabel(providerId)}.`
      );
    }
  }
}

export const coderSession = new CoderSessionState();
