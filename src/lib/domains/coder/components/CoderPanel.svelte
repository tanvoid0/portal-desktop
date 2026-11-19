<script lang="ts">
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { coderSession } from '../state/coderSession.svelte.js';
  import { coderWorkspaceStore } from '../state/coderWorkspaceStore.svelte.js';
  import { CoderPanelState } from '../state/coderPanel.svelte.js';
  import { PORTAL_CAPABILITIES as caps } from '../config/capabilities.js';
  import PlatformStatusBanner from '$lib/domains/ai/components/PlatformStatusBanner.svelte';
  import CoderShell from './CoderShell.svelte';
  import CoderSessionList from './CoderSessionList.svelte';
  import CoderSessionHeader from './CoderSessionHeader.svelte';
  import CoderTopbarActions from './CoderTopbarActions.svelte';
  import CoderEmptyHero from './CoderEmptyHero.svelte';
  import CoderMultitaskBar from './CoderMultitaskBar.svelte';
  import CoderComposer from './CoderComposer.svelte';
  import CoderFeed from './CoderFeed.svelte';
  import CoderWorkspaceSidebar from './CoderWorkspaceSidebar.svelte';
  import CoderWorkspacePanel from './CoderWorkspacePanel.svelte';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';
  import PageContainer from '$lib/components/shell/page-container.svelte';

  const p = new CoderPanelState();

  // Hand the action row to the AI layout's tab bar row — it has the width.
  $effect(() => {
    aiTopbar.actions = topbarActions;
    return () => {
      aiTopbar.actions = null;
    };
  });
</script>

{#snippet topbarActions()}
  <CoderTopbarActions
    thread={p.thread}
    activeThreadId={p.activeThreadId}
    activeTerminalThread={p.activeTerminalThread}
    running={p.running}
    showRetry={p.showRetry}
    platformBlocked={p.platformBlocked}
    pendingChangeCount={p.pendingChangeCount}
    gitStats={p.gitStats}
    canCopyConversation={p.canCopyConversation}
    getConversationText={p.copyText}
    onCommit={() => (p.showCommitDialog = true)}
  />
{/snippet}

{#snippet sessions()}
  <CoderSessionList
    threads={p.sessionThreads}
    loading={p.sessionsLoading}
    selectedThreadId={p.activeThreadId}
    runningThreadIds={p.runningThreadIds}
    showRules={p.showSettings}
    onToggleRules={() => (p.showSettings = !p.showSettings)}
    onProjectSelect={p.handleProjectSelect}
    onThreadClick={(t) => {
      p.selectThread(t.id);
      p.sessionsPanelOpen = false;
    }}
    onCreateNew={p.newSession}
    onDeleteThread={p.deleteSession}
    queuedCountFor={(id) => {
      coderSession.runtimeRevision;
      return coderSession.queuedCountFor(id);
    }}
    subAgentSummaryFor={(id) => {
      coderSession.runtimeRevision;
      return coderSession.subAgentSummaryFor(id);
    }}
  />
{/snippet}

{#snippet header()}
  <CoderSessionHeader
    thread={p.thread}
    parentCoordinatorId={p.parentCoordinatorId}
    running={p.running}
    queuedCount={p.queuedMessages.length}
    workspaceName={p.workspaceName}
    onOpenSessions={() => (p.sessionsPanelOpen = true)}
    onBackToCoordinator={() => void p.backToCoordinator()}
    onProjectSelect={p.handleProjectSelect}
  />
{/snippet}

{#snippet banner()}
  <PlatformStatusBanner blocks="Sending, retry and approvals" />

  {#if p.multitaskMode && p.thread?.thread_kind === 'coordinator'}
    <CoderMultitaskBar
      subAgents={p.subAgents}
      coordinatorId={p.thread.id}
      onOpen={(childId, coordId) => void p.openSubAgent(childId, coordId)}
      onCancel={(id) => void coderSession.cancelSubAgent(id)}
      onCleanupOne={(id) => void coderSession.cleanupSubAgents([id], true)}
      onCleanupAll={() => void coderSession.cleanupSubAgents([], true)}
    />
  {/if}
{/snippet}

{#snippet body()}
  {#if p.isEmpty}
    <div
      class="flex min-h-0 flex-1 flex-col items-center justify-center overflow-y-auto px-4 pb-8 [@media(max-height:700px)]:pb-2"
    >
      <CoderEmptyHero />
      <div class="w-full">
        <PageContainer variant="chat" class="px-0">
          <CoderComposer
            bind:value={p.input}
            onSend={p.send}
            rows={3}
            placeholder={p.platformBlocked
              ? p.blockedPlaceholder
              : p.multitaskMode
                ? 'Ask the coder… (one task per line runs parallel agents)'
                : 'Ask the coder…'}
            disabled={p.composerDisabled}
            running={p.running}
            queuedMessages={p.queuedMessages}
            onRemoveQueued={p.removeQueued}
            agentMode={coderSession.agentMode}
            effectiveMode={p.effectiveMode}
            onAgentModeChange={(m) => coderSession.changeAgentMode(m)}
            permissionMode={coderSession.permissionMode}
            onPermissionModeChange={(m) => coderSession.changePermissionMode(m)}
            workspaceRoot={coderSession.workspaceRoot}
            onToggleChanges={p.openChanges}
            onToggleGitChanges={() => coderWorkspaceStore.openGitChanges()}
            showChanges={p.showChanges}
            showGitChanges={p.showGitChanges}
            pendingChangeCount={p.pendingChangeCount}
            contextUsage={p.contextUsage}
            llmUsage={p.llmUsage}
            class="px-0 pb-0"
          />
        </PageContainer>
      </div>
      {#if p.error}
        <div
          class="mt-4 flex max-w-lg items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive"
        >
          <span class="flex-1">{p.error}</span>
        </div>
      {/if}
    </div>
  {:else}
    <ScrollArea class="min-h-0 flex-1" bind:viewportRef={p.scrollViewport}>
      <PageContainer variant="chat" class="space-y-3 py-4">
        <CoderFeed
          messages={p.messages}
          streamingText={p.streamingText}
          waitingSeconds={p.waitingSeconds}
          running={p.running}
          pending={p.pending}
          error={p.error}
          showRetry={p.showRetry}
          showRemember={caps.rememberRule}
          showSubAgentInline={p.showSubAgentInline}
          subAgents={p.subAgents}
          coordinatorId={p.thread?.id}
          workspaceRoot={p.workspaceRoot}
          threadId={p.activeThreadId ?? undefined}
          canEdit={p.canEditMessages}
          onEditMessage={(idx, text) => void coderSession.editMessage(idx, text)}
          onRetry={p.platformBlocked ? undefined : () => coderSession.retry()}
          decisionsDisabled={p.platformBlocked}
          onOpenSubAgent={(childId, coordId) => void p.openSubAgent(childId, coordId)}
          onCancelSubAgent={(id) => void coderSession.cancelSubAgent(id)}
          onCleanupSubAgent={(id) => void coderSession.cleanupSubAgents([id], true)}
          onDecision={(a, r, d) => coderSession.decide(a, r, d)}
        />
      </PageContainer>
    </ScrollArea>
  {/if}
{/snippet}

{#snippet composer()}
  <div class="shrink-0 bg-gradient-to-t from-background via-background to-transparent pt-2">
    <CoderComposer
      bind:value={p.input}
      onSend={p.send}
      onStop={() => void coderSession.stop()}
      disabled={p.composerDisabled}
      placeholder={p.platformBlocked ? p.blockedPlaceholder : undefined}
      running={p.running}
      canStop={p.running || !!p.pending}
      queuedMessages={p.queuedMessages}
      onRemoveQueued={p.removeQueued}
      agentMode={coderSession.agentMode}
      effectiveMode={p.effectiveMode}
      onAgentModeChange={(m) => coderSession.changeAgentMode(m)}
      permissionMode={coderSession.permissionMode}
      onPermissionModeChange={(m) => coderSession.changePermissionMode(m)}
      workspaceRoot={coderSession.workspaceRoot}
      onToggleChanges={p.openChanges}
      onToggleGitChanges={() => coderWorkspaceStore.openGitChanges()}
      showChanges={p.showChanges}
      showGitChanges={p.showGitChanges}
      pendingChangeCount={p.pendingChangeCount}
      changesRevision={p.changesRevision}
      contextUsage={p.contextUsage}
      llmUsage={p.llmUsage}
      class="pt-0"
    />
  </div>
{/snippet}

{#snippet workspacePanel()}
  <CoderWorkspacePanel
    threadId={p.activeThreadId}
    workspaceRoot={p.activeTerminalThread?.workspace_root ?? coderSession.workspaceRoot}
    changes={coderSession.changes}
    onRefreshChanges={() => coderSession.refreshChanges(p.thread?.id)}
    onCommit={() => (p.showCommitDialog = true)}
  />
{/snippet}

{#snippet workspaceSidebar()}
  <CoderWorkspaceSidebar
    threadId={p.activeThreadId}
    workspaceRoot={p.activeTerminalThread?.workspace_root ?? coderSession.workspaceRoot}
    workspaceName={p.workspaceName}
    gitStats={p.gitStats}
    pendingChangeCount={p.pendingChangeCount}
    agentChanges={caps.agentChanges}
  />
{/snippet}

<CoderShell
  bind:sessionsOpen={p.sessionsPanelOpen}
  bind:commitOpen={p.showCommitDialog}
  workspaceRoot={p.workspaceRoot}
  onCommitted={() => void p.refreshGitStats()}
  {sessions}
  {header}
  {banner}
  {body}
  composer={p.isEmpty ? undefined : composer}
  workspacePanel={p.showWorkspace ? workspacePanel : undefined}
  {workspaceSidebar}
/>
