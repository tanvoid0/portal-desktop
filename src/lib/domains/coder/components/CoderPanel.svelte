<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import {
    Bot,
    GitBranch,
    GitCommitHorizontal,
    RotateCcw,
    FolderOpen,
    Copy,
    Check,
    Terminal as TerminalIcon,
    PanelRightOpen,
    PanelLeftOpen,
    ArrowLeft,
  } from '@lucide/svelte';
  import { toast } from '$lib/utils/toast';
  import { replaceState } from '$app/navigation';
  import { coderSession, type SubAgentNotificationEvent } from '../state/coderSession.svelte.js';
  import { coderUi } from '../state/coderUi.svelte.js';
  import { coderService } from '../services/coderService.js';
  import { coderTerminalStore } from '../state/coderTerminalStore.svelte.js';
  import type { ChatMessage, GitDiffStats } from '../types.js';
  import CoderSessionList from './CoderSessionList.svelte';
  import CoderProjectSelector from './CoderProjectSelector.svelte';
  import CoderMultitaskBar from './CoderMultitaskBar.svelte';
  import ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
  import CoderComposer from './CoderComposer.svelte';
  import CoderFeed from './CoderFeed.svelte';
  import CoderWorkspaceSidebar from './CoderWorkspaceSidebar.svelte';
  import CoderWorkspacePanel from './CoderWorkspacePanel.svelte';
  import GitCommitDialog from './GitCommitDialog.svelte';
  import { coderWorkspaceStore } from '../state/coderWorkspaceStore.svelte.js';
  import { withMessageTimestamps } from '../utils/messageTimestamps.js';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import PageContainer from '$lib/components/shell/page-container.svelte';

  const showChanges = $derived(coderWorkspaceStore.activePanel === 'changes');
  const showGitChanges = $derived(coderWorkspaceStore.activePanel === 'git-changes');
  let showSettings = $state(false);
  let scrollViewport = $state<HTMLElement | null>(null);
  let input = $state('');
  let copied = $state(false);
  let gitStats = $state<GitDiffStats | null>(null);
  let showCommitDialog = $state(false);

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
    coderSession.changes.filter((c) => c.status === 'pending').length,
  );
  const composerDisabled = $derived(!!pending);
  const isEmpty = $derived(
    messages.filter((m) => m.role === 'user' || m.role === 'assistant').length === 0 &&
      !streamingText &&
      !running,
  );
  const workspaceName = $derived(
    coderSession.workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ??
      coderSession.workspaceRoot,
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
    thread?.thread_kind === 'coordinator' && subAgents.length > 0,
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
  const runningCount = $derived(runningThreadIds.size);
  const changesRevision = $derived(
    coderSession.changes.map((c) => `${c.id}:${c.status}`).join('|'),
  );
  const terminalOpen = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.terminalOpen;
  });
  const activeTerminalThread = $derived(
    sessionThreads.find((t) => t.id === activeThreadId) ?? thread,
  );

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
      activeTerminalThread.workspace_root,
    );
    const wsActive = coderWorkspaceStore.activeTerminalId();
    if (
      coderWorkspaceStore.activePanel !== 'terminal' ||
      wsActive !== tab.id
    ) {
      coderWorkspaceStore.openTerminal(activeThreadId, tab.id, tab.label);
    }
  });

  function handleSubAgentNotification(event: SubAgentNotificationEvent) {
    switch (event.type) {
      case 'spawned': {
        const label = `${event.count} sub-agent${event.count === 1 ? '' : 's'}`;
        if (coderSession.activeThreadId === event.coordinatorId) {
          toast.success(`Spawned ${label}`);
          break;
        }
        toast.success(`Spawned ${label}`, {
          action: {
            label: 'View',
            onClick: () => void selectThread(event.coordinatorId),
          },
        });
        break;
      }
      case 'finished': {
        const sa = event.subAgent;
        const statusLabel =
          sa.status === 'completed'
            ? 'completed'
            : sa.status === 'failed'
              ? 'failed'
              : 'finished';
        const toastFn = sa.status === 'failed' ? toast.error : toast.success;
        toastFn(`Sub-agent ${statusLabel}`, {
          description: sa.title,
          action: {
            label: 'View',
            onClick: () => void openSubAgent(sa.child_thread_id, event.coordinatorId),
          },
        });
        break;
      }
      case 'complete': {
        const done = event.subAgents.filter((s) => s.status === 'completed').length;
        const failed = event.subAgents.filter((s) => s.status === 'failed').length;
        const parts = [`${done} completed`];
        if (failed > 0) parts.push(`${failed} failed`);
        toast.info('Multitask complete', {
          description: parts.join(', '),
          action: {
            label: 'View',
            onClick: () => void selectThread(event.coordinatorId),
          },
        });
        break;
      }
    }
  }

  onMount(async () => {
    coderSession.setSubAgentNotificationHandler(handleSubAgentNotification);
    coderUi.initFromStorage();
    if (coderUi.activeProjectPath && !coderSession.workspaceRoot) {
      coderSession.workspaceRoot = coderUi.activeProjectPath;
    }

    try {
      await coderSession.ensureInit();
    } catch (e) {
      console.error("coder: ensureInit failed", e);
    }

    const urlId = $page.url.searchParams.get('id');
    if (urlId) {
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

    return () => {
      coderSession.setSubAgentNotificationHandler(null);
    };
  });

  function syncUrl(id: string | null) {
    const url = new URL($page.url);
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

  function toolNameFor(callId: string): string {
    for (const m of messages) {
      for (const tc of m.tool_calls ?? []) {
        if (tc.id === callId) return tc.function.name;
      }
    }
    return callId;
  }

  function formatConversationForCopy(): string {
    const lines: string[] = ['=== Coder Session ==='];
    if (thread?.title) lines.push(`Title: ${thread.title}`);
    if (coderSession.workspaceRoot) lines.push(`Workspace: ${coderSession.workspaceRoot}`);
    if (coderSession.selectedModel) lines.push(`Model: ${coderSession.selectedModel}`);
    lines.push(`Agent mode: ${coderSession.agentMode}`);
    lines.push(`Permission: ${coderSession.permissionMode}`);
    if (thread?.id) lines.push(`Thread ID: ${thread.id}`);
    lines.push('');

    for (const m of messages) {
      if (m.role === 'system') continue;

      const roleLabel =
        m.role === 'user'
          ? 'User'
          : m.role === 'assistant'
            ? 'Assistant'
            : m.role === 'tool'
              ? `Tool (${toolNameFor(m.tool_call_id ?? '')})`
              : m.role;

      lines.push(`--- ${roleLabel} ---`);
      if (m.content) lines.push(m.content);

      for (const tc of m.tool_calls ?? []) {
        lines.push('');
        lines.push(`[Tool Call: ${tc.function.name}]`);
        lines.push(tc.function.arguments);
      }
      lines.push('');
    }

    if (streamingText) {
      lines.push('--- Assistant (streaming) ---');
      lines.push(streamingText);
      lines.push('');
    }

    if (pending) {
      lines.push('--- Pending Approval ---');
      lines.push(`Tool: ${pending.tool}`);
      lines.push(`Summary: ${pending.summary}`);
      lines.push(JSON.stringify(pending.arguments, null, 2));
      lines.push('');
    }

    if (error) {
      lines.push('--- Error ---');
      lines.push(error);
    }

    return lines.join('\n').trim();
  }

  const canCopyConversation = $derived(
    messages.some((m) => m.role !== 'system') || !!streamingText || !!pending || !!error,
  );

  async function copyConversation() {
    const text = formatConversationForCopy();
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      toast.success('Conversation copied to clipboard');
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      toast.error('Failed to copy conversation');
    }
  }
</script>

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel
    bind:open={sessionsPanelOpen}
    side="left"
    desktopClass="w-80 bg-muted/20"
  >
    {#snippet header()}
      <div class="divider-edge-b divider-edge-full bg-background px-3 py-2.5">
        <h2 class="flex items-center gap-2 text-sm font-semibold tracking-tight">
          <Bot class="h-4 w-4 text-primary" />
          Sessions
          {#if runningCount > 0}
            <span
              class="rounded-full bg-primary/15 px-2 py-0.5 text-[10px] font-medium text-primary"
              title="{runningCount} session(s) running"
            >
              {runningCount} running
            </span>
          {/if}
        </h2>
      </div>
    {/snippet}
    <CoderSessionList
      threads={sessionThreads}
      loading={sessionsLoading}
      selectedThreadId={activeThreadId}
      runningThreadIds={runningThreadIds}
      showRules={showSettings}
      onToggleRules={() => (showSettings = !showSettings)}
      onProjectSelect={handleProjectSelect}
      onThreadClick={(t) => {
        selectThread(t.id);
        sessionsPanelOpen = false;
      }}
      onCreateNew={newSession}
      onDeleteThread={(t) => coderSession.removeThread(t)}
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
    <div
      class="divider-edge-b divider-edge-full flex shrink-0 flex-wrap items-center justify-between gap-2 bg-background px-4 py-2.5"
    >
      <div class="flex min-w-0 items-center gap-2">
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8 shrink-0 md:hidden"
          title="Sessions"
          onclick={() => (sessionsPanelOpen = true)}
        >
          <PanelLeftOpen class="h-4 w-4" />
        </Button>
        <h2 class="truncate text-sm font-semibold">
          {#if thread?.thread_kind === 'sub-agent' && parentCoordinatorId}
            <span class="text-muted-foreground">Sub-agent</span>
            <span class="mx-1 text-muted-foreground">/</span>
          {/if}
          {thread?.title || 'New session'}
        </h2>
        {#if thread?.thread_kind === 'sub-agent' && parentCoordinatorId}
          <Button
            size="sm"
            variant="ghost"
            class="h-7 gap-1 text-xs"
            title="Back to coordinator"
            onclick={() => void backToCoordinator()}
          >
            <ArrowLeft class="h-3.5 w-3.5" />
            Coordinator
          </Button>
        {/if}
        {#if running}
          <span class="flex items-center gap-1 text-xs text-primary">
            <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-primary"></span>
            Running
          </span>
        {/if}
        {#if queuedMessages.length > 0}
          <span class="text-xs text-muted-foreground">
            {queuedMessages.length} queued
          </span>
        {/if}
        {#if coderSession.workspaceRoot}
          <span class="hidden items-center gap-1 text-xs text-muted-foreground sm:flex">
            <FolderOpen class="h-3 w-3" />
            <span class="max-w-[200px] truncate">{workspaceName}</span>
          </span>
        {/if}
      </div>

      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-1.5 sm:max-w-md">
        <CoderProjectSelector
          bind:value={coderSession.workspaceRoot}
          disabled={!!thread}
          onSelect={handleProjectSelect}
        />
      </div>

      <div class="flex flex-wrap items-center gap-1.5">
        <ProviderModelSelector
          bind:selectedProvider={coderSession.selectedProvider}
          bind:selectedBackendProvider={coderSession.selectedBackendProvider}
          bind:selectedModel={coderSession.selectedModel}
          onModelChange={(m) => coderSession.handleModelChange(m)}
          onBackendProviderChange={(p) =>
            coderSession.handleBackendProviderChange(p)}
          modelSelectClass="w-[180px]"
        />
        {#if showRetry && thread && !running}
          <Button
            size="sm"
            variant="outline"
            class="h-8 gap-1"
            title="Retry the last message"
            onclick={() => coderSession.retry()}
          >
            <RotateCcw class="h-3.5 w-3.5" />
            Retry
          </Button>
        {/if}
        <Button
          size="sm"
          variant={coderWorkspaceStore.sidebarOpen ? 'secondary' : 'ghost'}
          class="h-8 gap-1"
          title="Workspace sidebar"
          onclick={() => {
            coderWorkspaceStore.sidebarOpen = !coderWorkspaceStore.sidebarOpen;
          }}
        >
          <PanelRightOpen class="h-3.5 w-3.5" />
        </Button>
        <Button
          size="sm"
          variant={terminalOpen ? 'secondary' : 'ghost'}
          class="h-8 gap-1"
          title="Session terminal"
          disabled={!activeThreadId}
          onclick={() => {
            if (!activeThreadId || !activeTerminalThread) return;
            const tab = coderTerminalStore.ensureDefault(
              activeThreadId,
              activeTerminalThread.workspace_root,
            );
            coderWorkspaceStore.openTerminal(activeThreadId, tab.id, tab.label);
            coderSession.terminalOpen = true;
          }}
        >
          <TerminalIcon class="h-3.5 w-3.5" />
          Terminal
        </Button>
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8"
          title="Copy conversation for debugging"
          disabled={!canCopyConversation}
          onclick={copyConversation}
        >
          {#if copied}
            <Check class="h-3.5 w-3.5 text-green-500" />
          {:else}
            <Copy class="h-3.5 w-3.5" />
          {/if}
        </Button>
        <Button
          size="sm"
          variant={showChanges ? 'secondary' : 'ghost'}
          class="h-8 gap-1"
          title="Review agent file changes"
          onclick={() => {
            coderWorkspaceStore.openChanges();
            coderSession.refreshChanges(thread?.id);
          }}
        >
          <Bot class="h-3.5 w-3.5" />
          {#if pendingChangeCount > 0}
            <span class="rounded-full bg-amber-500 px-1.5 text-[10px] text-white">
              {pendingChangeCount}
            </span>
          {/if}
        </Button>
        <Button
          size="sm"
          variant={showGitChanges ? 'secondary' : 'ghost'}
          class="h-8 gap-1"
          title="View git working tree changes"
          onclick={() => coderWorkspaceStore.openGitChanges()}
        >
          <GitBranch class="h-3.5 w-3.5" />
          {#if gitStats?.hasChanges}
            <span class="font-mono text-[10px] text-green-600 dark:text-green-400">
              {#if gitStats.additions > 0}+{gitStats.additions}{/if}
              {#if gitStats.deletions > 0}-{gitStats.deletions}{/if}
            </span>
          {/if}
        </Button>
        <Button
          size="sm"
          variant="outline"
          class="h-8 gap-1"
          title="Review and commit git changes"
          disabled={!gitStats?.hasChanges}
          onclick={() => (showCommitDialog = true)}
        >
          <GitCommitHorizontal class="h-3.5 w-3.5" />
          Commit
        </Button>
      </div>
    </div>

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
      <main class="flex min-w-0 flex-1 flex-col overflow-hidden">

    {#if isEmpty}
      <div class="flex flex-1 flex-col items-center justify-center px-4 pb-8">
        <div class="mb-6 text-center">
          <div
            class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10"
          >
            <Bot class="h-6 w-6 text-primary" />
          </div>
          <h3 class="text-lg font-semibold">What should we build?</h3>
          <p class="mt-1 max-w-md text-sm text-muted-foreground">
            Describe a task. The agent explores, edits, and runs commands — mode controls how it operates.
          </p>
        </div>
        <div class="w-full">
          <PageContainer variant="chat" class="px-0">
          <CoderComposer
            bind:value={input}
            onSend={send}
            rows={3}
            placeholder="Ask the coder… (2+ lines auto-spawns parallel agents)"
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
            onRetry={() => coderSession.retry()}
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
          {changesRevision}
          {contextUsage}
          {llmUsage}
          class="pt-0"
        />
      </div>
    {/if}
      </main>

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
            workspaceName={workspaceName}
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
