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
     can't fight the local writes above. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import { page } from '$app/state';
  import { afterNavigate, replaceState } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import PanelLeftOpen from '~icons/fa6-solid/table-columns';
  import AgentSessionSidebar from './AgentSessionSidebar.svelte';
  import AgentSessionPanel from './AgentSessionPanel.svelte';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { agentSessionsQuery } from '../queries/agentsQueries.js';

  interface Props {
    /** Which backend's tab this is — scopes the sidebar list, the launcher,
     *  and (below) the focused-session guard. Always 'claude-code' until
     *  #725 adds one route per backend. */
    backendId: string;
  }

  let { backendId }: Props = $props();

  let sidebarOpen = $state(false);
  let prefillCwd = $state<string | undefined>(page.url.searchParams.get('cwd') ?? undefined);
  let prefillPrompt = $state<string | undefined>(undefined);
  let prefillTaskId = $state<string | null>(null);
  let focusedId = $state<string | null>(page.url.searchParams.get('session'));

  function syncUrl(sessionId: string | null) {
    focusedId = sessionId;
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
      focusedId = fromUrl;
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
  $effect(() => {
    const session = sessionsQuery.data?.find((s) => s.id === focusedId);
    if (session && session.backend_id !== backendId) syncUrl(null);
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
</script>

<div class="flex h-full w-full overflow-hidden">
  <ResponsivePanel bind:open={sidebarOpen} side="left" desktopClass="w-80 bg-muted/20">
    <AgentSessionSidebar {backendId} {focusedId} onSelect={selectSession} onNewRun={newRun} />
  </ResponsivePanel>

  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <div class="divider-edge-b flex shrink-0 items-center gap-2 px-2 py-1.5 md:hidden">
      <Button
        size="icon"
        variant="ghost"
        class="h-8 w-8"
        title="Sessions"
        onclick={() => (sidebarOpen = true)}
      >
        <PanelLeftOpen class="h-4 w-4" />
      </Button>
    </div>
    <AgentSessionPanel
      {backendId}
      sessionId={focusedId}
      initialCwd={prefillCwd}
      initialPrompt={prefillPrompt}
      taskId={prefillTaskId}
      onStarted={(id) => syncUrl(id)}
    />
  </div>
</div>
