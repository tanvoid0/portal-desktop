<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { toast } from '$lib/utils/toast';
  import { afterNavigate, replaceState } from '$app/navigation';
  import { coderSession, type SubAgentNotificationEvent } from '../state/coderSession.svelte.js';
  import { coderUi } from '../state/coderUi.svelte.js';
  import { coderService } from '../services/coderService.js';
  import { coderTerminalStore } from '../state/coderTerminalStore.svelte.js';
  import { aiPlatform } from '$lib/domains/ai/state/aiPlatform.svelte.js';
  import PlatformStatusBanner from '$lib/domains/ai/components/PlatformStatusBanner.svelte';
  import { confirmAction } from '$lib/utils/confirm';
  import type { ChatMessage, CoderThread, GitDiffStats } from '../types.js';
  import CoderSessionList from './CoderSessionList.svelte';
  import CoderSessionHeader from './CoderSessionHeader.svelte';
  import CoderTopbarActions from './CoderTopbarActions.svelte';
  import CoderEmptyHero from './CoderEmptyHero.svelte';
  import CoderMultitaskBar from './CoderMultitaskBar.svelte';
  import CoderComposer from './CoderComposer.svelte';
  import CoderFeed from './CoderFeed.svelte';
  import CoderWorkspaceSidebar from './CoderWorkspaceSidebar.svelte';
  import CoderWorkspacePanel from './CoderWorkspacePanel.svelte';
  import GitCommitDialog from './GitCommitDialog.svelte';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';
  import { coderWorkspaceStore } from '../state/coderWorkspaceStore.svelte.js';
  import { withMessageTimestamps } from '../utils/messageTimestamps.js';
  import { formatConversationForCopy } from '../utils/formatConversation.js';
  import { toastSubAgentEvent } from '../utils/subAgentToasts.js';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import PageContainer from '$lib/components/shell/page-container.svelte';

  const showChanges = $derived(coderWorkspaceStore.activePanel === 'changes');
  const showGitChanges = $derived(coderWorkspaceStore.activePanel === 'git-changes');
  let showSettings = $state(false);
  let scrollViewport = $state<HTMLElement | null>(null);
  let input = $state('');
  let gitStats = $state<GitDiffStats | null>(null);
  let showCommitDialog = $state(false);
  const platformBlocked = $derived(aiPlatform.blocked);

  // Hand the action row to the AI layout's tab bar row — it has the width.
  $effect(() => {
    aiTopbar.actions = topbarActions;
    return () => {
      aiTopbar.actions = null;
    };
  });

  const showWorkspace = $derived(coderWorkspaceStore.activePanel !== 'chat');
  let sessionsPanelOpen = $state(false);

  const rt = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.activeRuntime;
  });
  const messages = $derived.by(() => {
    coderSession.runtimeRevision;
    const raw = rt.messages;
    return withMessageTimestamps(raw, raw, thread?.updated_at);
  });
  const pending = $derived(rt.pending);
  const running = $derived(rt.running);
  const streamingText = $derived(rt.streamingText);
  const waitingSeconds = $derived(rt.waitingSeconds);
  const error = $derived(rt.error);
  const showRetry = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.shouldShowRetry();
  });
  const thread = $derived(coderSession.thread);
  const pendingChangeCount = $derived(
    coderSession.changes.filter((c) => c.status === 'pending').length
  );
  const blockedPlaceholder = 'Agent Platform is unavailable — fix it above';
  const composerDisabled = $derived(!!pending || platformBlocked);
  const isEmpty = $derived(
    messages.filter((m) => m.role === 'user' || m.role === 'assistant').length === 0 &&
      !streamingText &&
      !running
  );
  const workspaceName = $derived(
    coderSession.workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ?? coderSession.workspaceRoot
  );
  const contextUsage = $derived(rt.contextUsage);
  const llmUsage = $derived(rt.llmUsage);
  const queuedMessages = $derived(rt.messageQueue);
  const subAgents = $derived(rt.subAgents);
  const parentCoordinatorId = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.parentCoordinatorId;
  });
  const showSubAgentInline = $derived(
    thread?.thread_kind === 'coordinator' && subAgents.length > 0
  );
  const multitaskMode = $derived.by(() => {
    coderSession.runtimeRevision;
    return (
      coderSession.agentMode === 'multitask' ||
      coderSession.multitaskMode ||
      thread?.thread_kind === 'coordinator'
    );
  });
  const effectiveMode = $derived.by(() => {
    coderSession.runtimeRevision;
    return rt.effectiveMode;
  });
  const sessionThreads = $derived.by(() => {
    coderSession.threadsRevision;
    return coderSession.threads;
  });
  const sessionsLoading = $derived(coderSession.threadsLoading);
  const activeThreadId = $derived.by(() => {
    coderSession.threadsRevision;
    return coderSession.activeThreadId;
  });
  const runningThreadIds = $derived.by(() => {
    coderSession.threadsRevision;
    coderSession.runtimeRevision;
    return coderSession.runningThreadIds;
  });
  const changesRevision = $derived(
    coderSession.changes.map((c) => `${c.id}:${c.status}`).join('|')
  );
  const terminalOpen = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.terminalOpen;
  });
  const activeTerminalThread = $derived(
    sessionThreads.find((t) => t.id === activeThreadId) ?? thread
  );

  async function deleteSession(t: CoderThread) {
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
  }

  async function refreshGitStats() {
    const root = thread?.workspace_root ?? coderSession.workspaceRoot;
    if (!root.trim()) {
      gitStats = null;
      return;
    }
    try {
      gitStats = await coderService.getGitDiffStats(root);
    } catch {
      gitStats = null;
    }
  }

  $effect(() => {
    const root = thread?.workspace_root ?? coderSession.workspaceRoot;
    if (root) void refreshGitStats();
  });

  $effect(() => {
    if (!terminalOpen || !activeThreadId || !activeTerminalThread) return;
    const tab = coderTerminalStore.ensureDefault(
      activeThreadId,
      activeTerminalThread.workspace_root
    );
    const wsActive = coderWorkspaceStore.activeTerminalId();
    if (coderWorkspaceStore.activePanel !== 'terminal' || wsActive !== tab.id) {
      coderWorkspaceStore.openTerminal(activeThreadId, tab.id, tab.label);
    }
  });

  function handleSubAgentNotification(event: SubAgentNotificationEvent) {
    toastSubAgentEvent(event, { selectThread, openSubAgent });
  }

  // Svelte ignores anything an async onMount returns, so the cleanup has to be
  // returned synchronously — otherwise the notification handler outlives the
  // component and fires against a destroyed panel.
  onMount(() => {
    void aiPlatform.refresh();
    coderSession.setSubAgentNotificationHandler(handleSubAgentNotification);
    coderUi.initFromStorage();
    if (coderUi.activeProjectPath && !coderSession.workspaceRoot) {
      coderSession.workspaceRoot = coderUi.activeProjectPath;
    }

    void (async () => {
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
        input = pendingPrompt;
      } else if (urlId) {
        const exists = coderSession.threads.find((t) => t.id === urlId);
        if (exists) await selectThread(exists.id);
        else {
          try {
            await coderSession.selectThread(urlId);
            syncUrl(urlId);
          } catch {
            /* ignore */
          }
        }
      } else if (!coderSession.activeThreadId) {
        const attached = await coderSession.attachToRunningThread();
        if (attached) syncUrl(attached);
      }
    })();

    return () => {
      coderSession.setSubAgentNotificationHandler(null);
    };
  });

  // External goto('/ai/coder?id=X') while this panel is already mounted reuses
  // the component, so onMount never re-reads `?id=` (#690, same as #662).
  // The first call is the navigation that mounted us — onMount owns that one.
  // syncUrl's replaceState never fires this, so it can't loop.
  let mountNavSeen = false;
  afterNavigate(() => {
    if (!mountNavSeen) {
      mountNavSeen = true;
      return;
    }
    const urlId = page.url.searchParams.get('id');
    if (urlId && urlId !== coderSession.activeThreadId) {
      selectThread(urlId).catch((e) => console.error('coder: select from URL failed', e));
    }
  });

  function syncUrl(id: string | null) {
    const url = new URL(page.url.href);
    if (id) url.searchParams.set('id', id);
    else url.searchParams.delete('id');
    replaceState(url, {});
  }

  async function selectThread(id: string) {
    await coderSession.selectThread(id);
    const t = coderSession.thread;
    if (t) {
      coderUi.syncFromThread(t.workspace_root, t.project_id);
    } else if (coderSession.workspaceRoot) {
      coderUi.syncFromThread(coderSession.workspaceRoot);
    }
    input = coderSession.activeRuntime.draftInput;
    syncUrl(id);
  }

  async function openSubAgent(childThreadId: string, coordinatorId: string) {
    await coderSession.openSubAgent(childThreadId, coordinatorId);
    const t = coderSession.thread;
    if (t) {
      coderUi.syncFromThread(t.workspace_root, t.project_id);
    }
    input = coderSession.activeRuntime.draftInput;
    syncUrl(childThreadId);
  }

  async function backToCoordinator() {
    if (!parentCoordinatorId) return;
    await selectThread(parentCoordinatorId);
  }

  function newSession() {
    const project = coderUi.activeProjectPath;
    coderSession.newSession();
    if (project) coderSession.workspaceRoot = project;
    input = '';
    syncUrl(null);
  }

  function handleProjectSelect(path: string, projectId?: string) {
    coderUi.setActiveProject(path, projectId ?? null);
    if (!coderSession.activeThreadId) {
      coderSession.workspaceRoot = path;
    }
  }

  async function send() {
    const text = input.trim();
    if (!text || composerDisabled) return;
    if (!coderSession.workspaceRoot.trim()) {
      toast.error('Set a workspace folder first.');
      return;
    }
    input = '';
    await coderSession.send(text);
    if (coderSession.activeThreadId) syncUrl(coderSession.activeThreadId);
  }

  async function spawnBackground() {
    const text = input.trim();
    if (!text || composerDisabled) return;
    if (!coderSession.workspaceRoot.trim()) {
      toast.error('Set a workspace folder first.');
      return;
    }
    input = '';
    try {
      const ok = await coderSession.spawnBackgroundTask(text);
      if (ok && coderSession.activeThreadId) {
        syncUrl(coderSession.activeThreadId);
      }
    } catch (e) {
      toast.error('Failed to spawn background sub-agent', String(e));
    }
  }

  function removeQueued(index: number) {
    const id = coderSession.activeThreadId;
    if (id) coderSession.removeFromQueue(id, index);
  }

  const canEditMessages = $derived(!running && !pending && !composerDisabled);

  $effect(() => {
    if (scrollViewport && (messages.length > 0 || streamingText || pending)) {
      requestAnimationFrame(() => {
        if (scrollViewport) scrollViewport.scrollTop = scrollViewport.scrollHeight;
      });
    }
  });

  $effect(() => {
    const id = coderSession.activeThreadId;
    if (id) input = coderSession.runtimeFor(id).draftInput;
  });

  const copyText = () =>
    formatConversationForCopy({ thread, messages, streamingText, pending, error });

  const canCopyConversation = $derived(
    messages.some((m) => m.role !== 'system') || !!streamingText || !!pending || !!error
  );
</script>

{#snippet topbarActions()}
  <CoderTopbarActions
    {thread}
    {activeThreadId}
    {activeTerminalThread}
    {running}
    {showRetry}
    {platformBlocked}
    {pendingChangeCount}
    {gitStats}
    {canCopyConversation}
    getConversationText={copyText}
    onCommit={() => (showCommitDialog = true)}
  />
{/snippet}

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel bind:open={sessionsPanelOpen} side="left" desktopClass="w-80 bg-muted/20">
    <CoderSessionList
      threads={sessionThreads}
      loading={sessionsLoading}
      selectedThreadId={activeThreadId}
      {runningThreadIds}
      showRules={showSettings}
      onToggleRules={() => (showSettings = !showSettings)}
      onProjectSelect={handleProjectSelect}
      onThreadClick={(t) => {
        selectThread(t.id);
        sessionsPanelOpen = false;
      }}
      onCreateNew={newSession}
      onDeleteThread={deleteSession}
      queuedCountFor={(id) => {
        coderSession.runtimeRevision;
        return coderSession.queuedCountFor(id);
      }}
      subAgentSummaryFor={(id) => {
        coderSession.runtimeRevision;
        return coderSession.subAgentSummaryFor(id);
      }}
    />
  </ResponsivePanel>

  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <CoderSessionHeader
      {thread}
      {parentCoordinatorId}
      {running}
      queuedCount={queuedMessages.length}
      {workspaceName}
      onOpenSessions={() => (sessionsPanelOpen = true)}
      onBackToCoordinator={() => void backToCoordinator()}
      onProjectSelect={handleProjectSelect}
    />

    <PlatformStatusBanner blocks="Sending, retry and approvals" />

    {#if multitaskMode && thread?.thread_kind === 'coordinator'}
      <CoderMultitaskBar
        {subAgents}
        coordinatorId={thread.id}
        onOpen={(childId, coordId) => void openSubAgent(childId, coordId)}
        onCancel={(id) => void coderSession.cancelSubAgent(id)}
        onCleanupOne={(id) => void coderSession.cleanupSubAgents([id], true)}
        onCleanupAll={() => void coderSession.cleanupSubAgents([], true)}
      />
    {/if}

    <div class="flex min-h-0 flex-1 overflow-hidden">
      <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
        {#if isEmpty}
          <div
            class="flex min-h-0 flex-1 flex-col items-center justify-center overflow-y-auto px-4 pb-8 [@media(max-height:700px)]:pb-2"
          >
            <CoderEmptyHero />
            <div class="w-full">
              <PageContainer variant="chat" class="px-0">
                <CoderComposer
                  bind:value={input}
                  onSend={send}
                  rows={3}
                  placeholder={platformBlocked
                    ? blockedPlaceholder
                    : multitaskMode
                      ? 'Ask the coder… (one task per line runs parallel agents)'
                      : 'Ask the coder…'}
                  disabled={composerDisabled}
                  {running}
                  {queuedMessages}
                  onRemoveQueued={removeQueued}
                  agentMode={coderSession.agentMode}
                  {effectiveMode}
                  onAgentModeChange={(m) => coderSession.changeAgentMode(m)}
                  permissionMode={coderSession.permissionMode}
                  onPermissionModeChange={(m) => coderSession.changePermissionMode(m)}
                  workspaceRoot={coderSession.workspaceRoot}
                  onToggleChanges={() => {
                    coderWorkspaceStore.openChanges();
                    coderSession.refreshChanges(thread?.id);
                  }}
                  onToggleGitChanges={() => coderWorkspaceStore.openGitChanges()}
                  {showChanges}
                  {showGitChanges}
                  {pendingChangeCount}
                  {contextUsage}
                  {llmUsage}
                  class="px-0 pb-0"
                />
              </PageContainer>
            </div>
            {#if error}
              <div
                class="mt-4 flex max-w-lg items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive"
              >
                <span class="flex-1">{error}</span>
              </div>
            {/if}
          </div>
        {:else}
          <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollViewport}>
            <PageContainer variant="chat" class="space-y-3 py-4">
              <CoderFeed
                {messages}
                {streamingText}
                {waitingSeconds}
                {running}
                {pending}
                {error}
                {showRetry}
                {showSubAgentInline}
                {subAgents}
                coordinatorId={thread?.id}
                workspaceRoot={thread?.workspace_root ?? coderSession.workspaceRoot}
                threadId={activeThreadId ?? undefined}
                canEdit={canEditMessages}
                onEditMessage={(idx, text) => void coderSession.editMessage(idx, text)}
                onRetry={platformBlocked ? undefined : () => coderSession.retry()}
                decisionsDisabled={platformBlocked}
                onOpenSubAgent={(childId, coordId) => void openSubAgent(childId, coordId)}
                onCancelSubAgent={(id) => void coderSession.cancelSubAgent(id)}
                onCleanupSubAgent={(id) => void coderSession.cleanupSubAgents([id], true)}
                onDecision={(a, r, p) => coderSession.decide(a, r, p)}
              />
            </PageContainer>
          </ScrollArea>
          <div class="shrink-0 bg-gradient-to-t from-background via-background to-transparent pt-2">
            <CoderComposer
              bind:value={input}
              onSend={send}
              onStop={() => void coderSession.stop()}
              disabled={composerDisabled}
              placeholder={platformBlocked ? blockedPlaceholder : undefined}
              {running}
              canStop={running || !!pending}
              {queuedMessages}
              onRemoveQueued={removeQueued}
              agentMode={coderSession.agentMode}
              {effectiveMode}
              onAgentModeChange={(m) => coderSession.changeAgentMode(m)}
              permissionMode={coderSession.permissionMode}
              onPermissionModeChange={(m) => coderSession.changePermissionMode(m)}
              workspaceRoot={coderSession.workspaceRoot}
              onToggleChanges={() => {
                coderWorkspaceStore.openChanges();
                coderSession.refreshChanges(thread?.id);
              }}
              onToggleGitChanges={() => coderWorkspaceStore.openGitChanges()}
              {showChanges}
              {showGitChanges}
              {pendingChangeCount}
              {changesRevision}
              {contextUsage}
              {llmUsage}
              class="pt-0"
            />
          </div>
        {/if}
      </div>

      {#if showWorkspace}
        <CoderWorkspacePanel
          threadId={activeThreadId}
          workspaceRoot={activeTerminalThread?.workspace_root ?? coderSession.workspaceRoot}
          changes={coderSession.changes}
          onRefreshChanges={() => coderSession.refreshChanges(thread?.id)}
          onCommit={() => (showCommitDialog = true)}
        />
      {/if}

      {#if coderWorkspaceStore.sidebarOpen}
        <ResponsivePanel
          bind:open={coderWorkspaceStore.sidebarOpen}
          side="right"
          desktopClass="divider-edge-l w-56 bg-muted/20"
          borderClass=""
        >
          <CoderWorkspaceSidebar
            threadId={activeThreadId}
            workspaceRoot={activeTerminalThread?.workspace_root ?? coderSession.workspaceRoot}
            {workspaceName}
            {gitStats}
            {pendingChangeCount}
          />
        </ResponsivePanel>
      {/if}
    </div>
  </div>
</div>

<GitCommitDialog
  bind:open={showCommitDialog}
  workspaceRoot={thread?.workspace_root ?? coderSession.workspaceRoot}
  onCommitted={() => void refreshGitStats()}
/>
