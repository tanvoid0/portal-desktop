<!-- An agent backend's tab in the Code workspace (/ai/coder/<backend>, #546/#725): a project-grouped session sidebar
     beside a panel that is the launcher when nothing is focused and a live
     transcript when a session is. `focusedId` is local state (what actually
     drives the panel); the URL's `?session=` is kept in sync via
     `replaceState` purely for deep-linking/refresh — like CoderPanel's `?id=`,
     which likewise drives off its own `coderSession.activeThreadId` and only
     mirrors into the URL, never the other way around (a plain
     `$derived(page.url...)` here doesn't reliably re-fire on our own
     `replaceState`, see #635). External navigation (a notification's
     `goto('/ai/coder/claude-code?session=…')` while this page is already open, #662)
     goes the other way, picked up via `afterNavigate` — `replaceState` is
     exempt from the navigation lifecycle, so it never trips that handler and
     can't fight the local writes above. A focused session also gets the Coder
     workspace (git changes, terminal, files, browser) beside its transcript,
     rooted at the session's cwd. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import { page } from '$app/state';
  import { afterNavigate, replaceState } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { toast } from '$lib/utils/toast';
  import AgentSessionSidebar from './AgentSessionSidebar.svelte';
  import PanelRightOpen from '~icons/fa6-solid/table-columns';
  import AgentSessionPanel from './AgentSessionPanel.svelte';
  import CoderWorkspacePanel from '$lib/domains/coder/components/CoderWorkspacePanel.svelte';
  import CoderWorkspaceSidebar from '$lib/domains/coder/components/CoderWorkspaceSidebar.svelte';
  import CoderShell from '$lib/domains/coder/components/CoderShell.svelte';
  import { agentCapabilities } from '$lib/domains/coder/config/capabilities.js';
  import { provideCoderWorkspace } from '$lib/domains/coder/state/coderWorkspaceStore.svelte.js';
  import { aiTopbar } from '$lib/domains/ai/state/aiTopbarStore.svelte.js';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { agentSessionsQuery } from '../queries/agentsQueries.js';

  interface Props {
    /** Which backend's tab this is — scopes the sidebar list, the launcher,
     *  and (below) the focused-session guard. Always 'claude-code' until
     *  #725 adds one route per backend. */
    backendId: string;
  }

  let { backendId }: Props = $props();

  const caps = $derived(agentCapabilities(backendId));
  let sidebarOpen = $state(false);
  let feedView = $state<'transcript' | 'timeline'>('transcript');
  let prefillCwd = $state<string | undefined>(page.url.searchParams.get('cwd') ?? undefined);
  let prefillPrompt = $state<string | undefined>(undefined);
  let prefillTaskId = $state<string | null>(null);
  let focusedId = $state<string | null>(page.url.searchParams.get('session'));

  // This tab's own workspace, so Portal-tab coder runs can't touch it. Reset
  // synchronously with the focus change, before the new session renders.
  const coderWorkspaceStore = provideCoderWorkspace();
  let commitOpen = $state(false);

  function focus(sessionId: string | null) {
    if (sessionId !== focusedId) {
      coderWorkspaceStore.openTabs = [];
      coderWorkspaceStore.openChat();
      commitOpen = false;
    }
    focusedId = sessionId;
  }

  function syncUrl(sessionId: string | null) {
    focus(sessionId);
    agentRuns.lastFocused[backendId] = sessionId;
    const url = new URL(page.url.href);
    if (sessionId) url.searchParams.set('session', sessionId);
    else url.searchParams.delete('session');
    url.searchParams.delete('cwd');
    replaceState(url, {});
  }

  // Catches external navigation to a different `?session=` while this page
  // is already mounted (#662) — SvelteKit reuses the component for a
  // same-route `goto`, so the mount-time $state above goes stale otherwise.
  // Skip the initial 'enter' event: the $state initializer already covers
  // it, and onMount's pending-launch handling below may still override it.
  // Also fires (after onMount) when a client-side nav mounts this page, so a
  // bare tab URL (engine-tab switch, #722 — tab links carry no `?session=`)
  // restores this tab's last session here rather than in onMount.
  afterNavigate(({ type }) => {
    if (type === 'enter') {
      agentRuns.lastFocused[backendId] = focusedId;
      return;
    }
    const fromUrl = page.url.searchParams.get('session');
    if (fromUrl || page.url.searchParams.has('cwd') || agentRuns.pendingLaunch) {
      focus(fromUrl);
      agentRuns.lastFocused[backendId] = fromUrl;
    } else {
      syncUrl(agentRuns.lastFocused[backendId] ?? null);
    }
  });

  // A deep link or notification carries only a session id (#662's syncUrl,
  // task-handoff links). If it turns out to belong to a different backend
  // than this tab, don't render it under this tab's title/filtered list.
  // There's no other tab to send it to yet — #725 adds one route per
  // backend; until then the safe fallback is just dropping focus.
  const sessionsQuery = createQuery(agentSessionsQuery);
  const focused = $derived(sessionsQuery.data?.find((s) => s.id === focusedId));
  const cwd = $derived(focused?.cwd ?? '');
  $effect(() => {
    if (focused && focused.backend_id !== backendId) syncUrl(null);
  });

  $effect(() => {
    if (!cwd && !focusedId) return;
    aiTopbar.actions = topbarActions;
    return () => {
      aiTopbar.actions = null;
    };
  });

  onMount(() => {
    void agentRuns.ensureInit();
    const pending = agentRuns.pendingLaunch;
    if (pending) {
      agentRuns.pendingLaunch = null;
      prefillCwd = pending.cwd ?? prefillCwd;
      prefillPrompt = pending.prompt;
      prefillTaskId = pending.taskId ?? null;
      syncUrl(null);
    }
  });

  function selectSession(id: string) {
    sidebarOpen = false;
    syncUrl(id);
  }

  function newRun() {
    sidebarOpen = false;
    prefillCwd = undefined;
    prefillPrompt = undefined;
    prefillTaskId = null;
    syncUrl(null);
  }
  async function endRun() {
    if (!focusedId) return;
    try {
      await agentRuns.cancel(focusedId);
    } catch (e) {
      toast.error(String(e));
    }
  }
</script>

{#snippet topbarActions()}
  {#if focusedId && caps.timeline}
    <Tabs value={feedView} onValueChange={(v) => (feedView = v as typeof feedView)}>
      <TabsList class="h-8">
        <TabsTrigger value="transcript" class="px-2.5 text-xs">Transcript</TabsTrigger>
        <TabsTrigger value="timeline" class="px-2.5 text-xs">Timeline</TabsTrigger>
      </TabsList>
    </Tabs>
  {/if}
  {#if focusedId && caps.endRun && !agentRuns.isEnded(focusedId)}
    <Button size="sm" variant="outline" class="h-8" onclick={endRun}>End run</Button>
  {/if}
  {#if cwd}
    <Button
      size="sm"
      variant={coderWorkspaceStore.sidebarOpen ? 'secondary' : 'ghost'}
      class="h-8 gap-1"
      title="Workspace sidebar"
      aria-label="Workspace sidebar"
      onclick={() => {
        coderWorkspaceStore.sidebarOpen = !coderWorkspaceStore.sidebarOpen;
      }}
    >
      <PanelRightOpen class="h-3.5 w-3.5" />
    </Button>
  {/if}
{/snippet}

{#snippet sessions()}
  <AgentSessionSidebar {backendId} {focusedId} onSelect={selectSession} onNewRun={newRun} />
{/snippet}

{#snippet body()}
  <AgentSessionPanel
    {backendId}
    sessionId={focusedId}
    initialCwd={prefillCwd}
    initialPrompt={prefillPrompt}
    taskId={prefillTaskId}
    bind:feedView
    onStarted={(id) => syncUrl(id)}
  />
{/snippet}

{#snippet workspacePanel()}
  <CoderWorkspacePanel
    threadId={focusedId}
    workspaceRoot={cwd}
    changes={[]}
    onRefreshChanges={() => {}}
    onCommit={() => (commitOpen = true)}
  />
{/snippet}

{#snippet workspaceSidebar()}
  <CoderWorkspaceSidebar
    threadId={focusedId}
    workspaceRoot={cwd}
    workspaceName={cwd.split(/[\\/]/).filter(Boolean).pop() ?? ''}
    agentChanges={caps.agentChanges}
  />
{/snippet}

<CoderShell
  adaptiveDock
  bind:sessionsOpen={sidebarOpen}
  bind:commitOpen
  workspaceRoot={cwd}
  {sessions}
  {body}
  workspacePanel={cwd && coderWorkspaceStore.activePanel !== 'chat' ? workspacePanel : undefined}
  workspaceSidebar={cwd ? workspaceSidebar : undefined}
/>
