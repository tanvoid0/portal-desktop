import { onMount } from 'svelte';
import { createQuery } from '@tanstack/svelte-query';
import { page } from '$app/state';
import { afterNavigate, replaceState } from '$app/navigation';
import { toast } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
import {
  actorLabel,
  coderSession,
  errorText,
  type SubAgentNotificationEvent,
} from './coderSession.svelte.js';
import { coderUi } from './coderUi.svelte.js';
import { coderTerminalStore } from './coderTerminalStore.svelte.js';
import { coderWorkspaceStore } from './coderWorkspaceStore.svelte.js';
import { coderService } from '../services/coderService.js';
import { mirrorToolResult } from '../services/hostResponder.js';
import { threadApi } from '$lib/domains/threads/api/threadApi';
import { threadStore } from '$lib/domains/threads/state/threadStore.svelte';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { withMessageTimestamps } from '../utils/messageTimestamps.js';
import { formatConversationForCopy } from '../utils/formatConversation.js';
import { toastSubAgentEvent } from '../utils/subAgentToasts.js';
import { deleteHistoryMessage } from '../utils/threadMenu.js';
import type { ChatMessage, CoderThread, GitDiffStats } from '../types.js';

/**
 * Script-side state of CoderPanel. Construct once during component init: the
 * constructor registers $effects, onMount and afterNavigate, which need that
 * context (and tear down with the component).
 */
export class CoderPanelState {
  showSettings = $state(false);
  scrollViewport = $state<HTMLElement | null>(null);
  input = $state('');
  gitStats = $state<GitDiffStats | null>(null);
  showCommitDialog = $state(false);
  sessionsPanelOpen = $state(false);
  readonly blockedPlaceholder = 'Agent Platform is unavailable — fix it above';

  showChanges = $derived(coderWorkspaceStore.activePanel === 'changes');
  showGitChanges = $derived(coderWorkspaceStore.activePanel === 'git-changes');
  showWorkspace = $derived(coderWorkspaceStore.activePanel !== 'chat');
  platformBlocked = $derived(aiPlatform.blocked);

  /** The active thread's live view from the hub's log (null until its snapshot lands). */
  view = $derived(coderSession.view);
  rt = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.activeRuntime;
  });
  thread = $derived(coderSession.thread);
  /** True from the click until the hub's first frame lands, so the empty hero does not flash. */
  sending = $state(false);
  private stamped: { id: string | null; list: ChatMessage[] } = { id: null, list: [] };
  messages = $derived.by(() => {
    const raw = this.view?.messages ?? [];
    // First sight of a thread is anchored on its row; after that only new messages are stamped, now.
    const fresh = this.stamped.id !== this.activeThreadId || this.stamped.list.length === 0;
    const list = withMessageTimestamps(
      raw,
      fresh ? raw : this.stamped.list,
      fresh ? this.thread?.updated_at : undefined
    );
    this.stamped = { id: this.activeThreadId, list };
    return list;
  });
  pending = $derived(this.view?.approvals[0] ?? null);
  running = $derived(this.view?.status === 'running');
  stopping = $derived(this.view?.stopping ?? false);
  streamingText = $derived(this.view?.streamingText ?? '');
  waitingSeconds = $derived(this.view?.waitingSeconds ?? 0);
  error = $derived(this.view?.error ?? null);
  showRetry = $derived(!!this.thread && !!this.view?.canRetry);
  /** Who decided the latest approval, when it was not this computer. */
  decidedNote = $derived.by(() => {
    const r = this.view?.recentlyResolved.at(-1);
    return r && r.by !== 'local'
      ? `${r.allow ? 'Approved' : 'Denied'} by ${actorLabel(r.by)}`
      : null;
  });
  pendingChangeCount = $derived(coderSession.changes.filter((c) => c.status === 'pending').length);
  composerDisabled = $derived(!!this.pending || this.platformBlocked);
  isEmpty = $derived(
    this.messages.filter((m) => m.role === 'user' || m.role === 'assistant').length === 0 &&
      !this.streamingText &&
      !this.running &&
      !this.sending
  );
  workspaceName = $derived(
    coderSession.workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ?? coderSession.workspaceRoot
  );
  contextUsage = $derived(this.rt.contextUsage);
  /** The hub's queue behind the open turn; read-only (no remove command yet). */
  queuedMessages = $derived((this.view?.queue ?? []).map((q) => q.text));
  subAgents = $derived(this.rt.subAgents);
  parentCoordinatorId = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.parentCoordinatorId;
  });
  showSubAgentInline = $derived(
    this.thread?.thread_kind === 'coordinator' && this.subAgents.length > 0
  );
  multitaskMode = $derived.by(() => {
    coderSession.runtimeRevision;
    return (
      coderSession.agentMode === 'multitask' ||
      coderSession.multitaskMode ||
      this.thread?.thread_kind === 'coordinator'
    );
  });
  effectiveMode = $derived.by(() => {
    coderSession.runtimeRevision;
    return this.rt.effectiveMode;
  });
  sessionThreads = $derived.by(() => {
    coderSession.threadsRevision;
    return coderSession.threads;
  });
  sessionsLoading = $derived(coderSession.threadsLoading);
  activeThreadId = $derived.by(() => {
    coderSession.threadsRevision;
    return coderSession.activeThreadId;
  });
  // Every thread's status, any provider; the active one is overridden by its live view.
  private threadList = createQuery(() => ({
    queryKey: queryKeys.threads.list,
    queryFn: () => threadApi.list(),
    refetchInterval: 5000, // ponytail: poll; the store only follows the subscribed thread
  }));
  runningThreadIds = $derived.by(() => {
    const ids = new Set(
      (this.threadList.data ?? []).filter((s) => s.status === 'running').map((s) => s.id)
    );
    const status = this.view?.status;
    if (this.activeThreadId && status) {
      if (status === 'running') ids.add(this.activeThreadId);
      else ids.delete(this.activeThreadId);
    }
    return ids;
  });
  changesRevision = $derived(coderSession.changes.map((c) => `${c.id}:${c.status}`).join('|'));
  terminalOpen = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.terminalOpen;
  });
  activeTerminalThread = $derived(
    this.sessionThreads.find((t) => t.id === this.activeThreadId) ?? this.thread
  );
  canEditMessages = $derived(
    !this.running && !this.stopping && !this.pending && !this.composerDisabled
  );
  canCopyConversation = $derived(
    this.messages.some((m) => m.role !== 'system') ||
      !!this.streamingText ||
      !!this.pending ||
      !!this.error
  );
  workspaceRoot = $derived(this.thread?.workspace_root ?? coderSession.workspaceRoot);

  // afterNavigate's first call is the navigation that mounted us — onMount owns that one.
  private mountNavSeen = false;

  constructor() {
    $effect(() => {
      if (this.workspaceRoot) void this.refreshGitStats();
    });

    $effect(() => {
      const id = this.activeThreadId;
      const t = this.activeTerminalThread;
      if (!this.terminalOpen || !id || !t) return;
      const tab = coderTerminalStore.ensureDefault(id, t.workspace_root);
      const wsActive = coderWorkspaceStore.activeTerminalId();
      if (coderWorkspaceStore.activePanel !== 'terminal' || wsActive !== tab.id) {
        coderWorkspaceStore.openTerminal(id, tab.id, tab.label);
      }
    });

    $effect(() => {
      const vp = this.scrollViewport;
      if (vp && (this.messages.length > 0 || this.streamingText || this.pending)) {
        requestAnimationFrame(() => {
          if (this.scrollViewport) this.scrollViewport.scrollTop = this.scrollViewport.scrollHeight;
        });
      }
    });

    $effect(() => {
      const id = coderSession.activeThreadId;
      if (id) this.input = coderSession.runtimeFor(id).draftInput;
    });

    $effect(() => {
      const id = coderSession.activeThreadId;
      if (id) return threadStore.subscribe(id);
    });

    // A turn closing refreshes the thread row and the context gauge.
    let openFor: string | null = null;
    $effect(() => {
      const id = coderSession.activeThreadId;
      const open = this.view?.status === 'running' || this.view?.status === 'waiting';
      if (id && !open && openFor === id) void coderSession.syncThread(id);
      openFor = open ? id : null;
    });

    // Threads running elsewhere (phone, agents page) appear in the sidebar.
    $effect(() => {
      for (const id of this.runningThreadIds) void coderSession.ensureThreadListed(id);
    });

    // Echo tool results this window did not run into the session terminal. The first
    // view of a thread is history and is not echoed.
    let echoed = { id: '', n: 0 };
    $effect(() => {
      const id = coderSession.activeThreadId;
      const msgs = this.view?.messages;
      if (!id || !msgs) return;
      if (echoed.id === id) for (const m of msgs.slice(echoed.n)) mirrorToolResult(id, msgs, m);
      echoed = { id, n: msgs.length };
    });

    // External goto('/ai/coder?id=X') while this panel is already mounted reuses
    // the component, so onMount never re-reads `?id=` (#690, same as #662).
    // syncUrl's replaceState never fires this, so it can't loop.
    afterNavigate(() => {
      if (!this.mountNavSeen) {
        this.mountNavSeen = true;
        return;
      }
      const urlId = page.url.searchParams.get('id');
      if (urlId && urlId !== coderSession.activeThreadId) {
        this.selectThread(urlId).catch((e) => console.error('coder: select from URL failed', e));
      }
    });

    // Svelte ignores anything an async onMount returns, so the cleanup has to be
    // returned synchronously — otherwise the notification handler outlives the
    // component and fires against a destroyed panel.
    onMount(() => {
      void aiPlatform.refresh();
      coderSession.setSubAgentNotificationHandler(this.handleSubAgentNotification);
      coderUi.initFromStorage();
      if (coderUi.activeProjectPath && !coderSession.workspaceRoot) {
        coderSession.workspaceRoot = coderUi.activeProjectPath;
      }
      void this.initSession();
      return () => {
        coderSession.setSubAgentNotificationHandler(null);
      };
    });
  }

  private handleSubAgentNotification = (event: SubAgentNotificationEvent) => {
    toastSubAgentEvent(event, {
      selectThread: this.selectThread,
      openSubAgent: this.openSubAgent,
    });
  };

  private async initSession() {
    try {
      await coderSession.ensureInit();
    } catch (e) {
      console.error('coder: ensureInit failed', e);
    }

    const pendingPrompt = coderSession.pendingPrompt;
    coderSession.pendingPrompt = null;
    const urlId = page.url.searchParams.get('id');
    if (pendingPrompt && !urlId) {
      // Launched from another page ("Open in Coder") with a starting prompt.
      // openCoderSession already set up the fresh session + workspace; just
      // pre-fill the composer and don't attach to a running thread.
      this.input = pendingPrompt;
    } else if (urlId) {
      const exists = coderSession.threads.find((t) => t.id === urlId);
      if (exists) await this.selectThread(exists.id);
      else {
        try {
          await coderSession.selectThread(urlId);
          this.syncUrl(urlId);
        } catch {
          /* ignore */
        }
      }
    } else if (!coderSession.activeThreadId) {
      const attached = await this.attachToRunningThread();
      if (attached) this.syncUrl(attached);
    }
  }

  /** On page load, follow the most recent running session so the UI matches background work. */
  private async attachToRunningThread(): Promise<string | null> {
    const running = (await threadApi.list())
      .filter((s) => s.status === 'running')
      .sort((a, b) => b.updated_at.localeCompare(a.updated_at))[0];
    if (!running) return null;
    await coderSession.selectThread(running.id);
    return running.id;
  }

  deleteSession = async (t: CoderThread) => {
    const ok = await confirmAction(
      `Delete "${t.title || 'this session'}" and its transcript?`,
      'Delete session',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!ok) return;
    try {
      await coderSession.removeThread(t);
      toast.success('Session deleted');
    } catch (e) {
      toast.error(`Failed to delete session: ${e}`);
    }
  };

  /** Delete a workspace's chat history after one confirm. */
  deleteSessions = async (threads: CoderThread[], label: string, kept: number) => {
    const ok = await confirmAction(
      deleteHistoryMessage(threads.length, label, kept),
      'Delete chat history',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!ok) return;
    try {
      await coderSession.removeThreads(threads);
      toast.success('Chat history deleted');
    } catch (e) {
      toast.error(`Failed to delete chat history: ${e}`);
    }
  };

  refreshGitStats = async () => {
    const root = this.workspaceRoot;
    if (!root.trim()) {
      this.gitStats = null;
      return;
    }
    try {
      this.gitStats = await coderService.getGitDiffStats(root);
    } catch {
      this.gitStats = null;
    }
  };

  syncUrl(id: string | null) {
    const url = new URL(page.url.href);
    if (id) url.searchParams.set('id', id);
    else url.searchParams.delete('id');
    replaceState(url, {});
  }

  selectThread = async (id: string) => {
    await coderSession.selectThread(id);
    const t = coderSession.thread;
    if (t) {
      coderUi.syncFromThread(t.workspace_root, t.project_id);
    } else if (coderSession.workspaceRoot) {
      coderUi.syncFromThread(coderSession.workspaceRoot);
    }
    this.input = coderSession.activeRuntime.draftInput;
    this.syncUrl(id);
  };

  openSubAgent = async (childThreadId: string, coordinatorId: string) => {
    await coderSession.openSubAgent(childThreadId, coordinatorId);
    const t = coderSession.thread;
    if (t) {
      coderUi.syncFromThread(t.workspace_root, t.project_id);
    }
    this.input = coderSession.activeRuntime.draftInput;
    this.syncUrl(childThreadId);
  };

  backToCoordinator = async () => {
    if (!this.parentCoordinatorId) return;
    await this.selectThread(this.parentCoordinatorId);
  };

  newSession = () => {
    const project = coderUi.activeProjectPath;
    coderSession.newSession();
    if (project) coderSession.workspaceRoot = project;
    this.input = '';
    this.syncUrl(null);
  };

  handleProjectSelect = (path: string, projectId?: string) => {
    coderUi.setActiveProject(path, projectId ?? null);
    if (!coderSession.activeThreadId) {
      coderSession.workspaceRoot = path;
    }
  };

  /** Shared guard + clear for send / spawnBackground; returns the text to submit. */
  private takeInput(): string | null {
    const text = this.input.trim();
    if (!text || this.composerDisabled) return null;
    if (!coderSession.workspaceRoot.trim()) {
      toast.error('Set a workspace folder first.');
      return null;
    }
    this.input = '';
    return text;
  }

  send = async () => {
    const text = this.takeInput();
    if (text === null) return;
    this.sending = true;
    try {
      await coderSession.send(text);
      if (coderSession.activeThreadId) this.syncUrl(coderSession.activeThreadId);
    } catch (e) {
      // Nothing was sent (`busy`, a failed write): give the draft back.
      if (!this.input) this.input = text;
      toast.error(errorText(e));
    } finally {
      this.sending = false;
    }
  };

  spawnBackground = async () => {
    const text = this.takeInput();
    if (text === null) return;
    try {
      const ok = await coderSession.spawnBackgroundTask(text);
      if (ok && coderSession.activeThreadId) {
        this.syncUrl(coderSession.activeThreadId);
      }
    } catch (e) {
      toast.error('Failed to spawn background sub-agent', errorText(e));
    }
  };

  /** Only the open thread's queue is live. */
  queuedCountFor = (id: string) => (id === this.activeThreadId ? this.queuedMessages.length : 0);

  openChanges = () => {
    coderWorkspaceStore.openChanges();
    coderSession.refreshChanges(this.thread?.id);
  };

  copyText = () =>
    formatConversationForCopy({
      thread: this.thread,
      messages: this.messages,
      streamingText: this.streamingText,
      pending: this.pending,
      error: this.error,
    });
}
