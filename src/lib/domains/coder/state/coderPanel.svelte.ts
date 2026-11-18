import { onMount } from 'svelte';
import { page } from '$app/state';
import { afterNavigate, replaceState } from '$app/navigation';
import { toast } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';
import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
import { coderSession, type SubAgentNotificationEvent } from './coderSession.svelte.js';
import { coderUi } from './coderUi.svelte.js';
import { coderTerminalStore } from './coderTerminalStore.svelte.js';
import { coderWorkspaceStore } from './coderWorkspaceStore.svelte.js';
import { coderService } from '../services/coderService.js';
import { withMessageTimestamps } from '../utils/messageTimestamps.js';
import { formatConversationForCopy } from '../utils/formatConversation.js';
import { toastSubAgentEvent } from '../utils/subAgentToasts.js';
import type { CoderThread, GitDiffStats } from '../types.js';

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

  rt = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.activeRuntime;
  });
  thread = $derived(coderSession.thread);
  messages = $derived.by(() => {
    coderSession.runtimeRevision;
    const raw = this.rt.messages;
    return withMessageTimestamps(raw, raw, this.thread?.updated_at);
  });
  pending = $derived(this.rt.pending);
  running = $derived(this.rt.running);
  streamingText = $derived(this.rt.streamingText);
  waitingSeconds = $derived(this.rt.waitingSeconds);
  error = $derived(this.rt.error);
  showRetry = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.shouldShowRetry();
  });
  pendingChangeCount = $derived(coderSession.changes.filter((c) => c.status === 'pending').length);
  composerDisabled = $derived(!!this.pending || this.platformBlocked);
  isEmpty = $derived(
    this.messages.filter((m) => m.role === 'user' || m.role === 'assistant').length === 0 &&
      !this.streamingText &&
      !this.running
  );
  workspaceName = $derived(
    coderSession.workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ?? coderSession.workspaceRoot
  );
  contextUsage = $derived(this.rt.contextUsage);
  llmUsage = $derived(this.rt.llmUsage);
  queuedMessages = $derived(this.rt.messageQueue);
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
  runningThreadIds = $derived.by(() => {
    coderSession.threadsRevision;
    coderSession.runtimeRevision;
    return coderSession.runningThreadIds;
  });
  changesRevision = $derived(coderSession.changes.map((c) => `${c.id}:${c.status}`).join('|'));
  terminalOpen = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.terminalOpen;
  });
  activeTerminalThread = $derived(
    this.sessionThreads.find((t) => t.id === this.activeThreadId) ?? this.thread
  );
  canEditMessages = $derived(!this.running && !this.pending && !this.composerDisabled);
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
      const attached = await coderSession.attachToRunningThread();
      if (attached) this.syncUrl(attached);
    }
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
    await coderSession.send(text);
    if (coderSession.activeThreadId) this.syncUrl(coderSession.activeThreadId);
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
      toast.error('Failed to spawn background sub-agent', String(e));
    }
  };

  removeQueued = (index: number) => {
    const id = coderSession.activeThreadId;
    if (id) coderSession.removeFromQueue(id, index);
  };

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
