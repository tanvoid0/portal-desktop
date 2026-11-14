<!-- The Agents tab's one real page (#546): a project-grouped session sidebar
     beside a panel that is the launcher when nothing is focused and a live
     transcript when a session is. `?session=` is the URL's source of truth
     for which session is focused — same `replaceState` sync CoderPanel uses
     for `?id=`. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { replaceState } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import ResponsivePanel from '$lib/components/shell/responsive-panel.svelte';
  import PanelLeftOpen from '~icons/fa6-solid/table-columns';
  import AgentSessionSidebar from './AgentSessionSidebar.svelte';
  import AgentSessionPanel from './AgentSessionPanel.svelte';
  import { agentRuns } from '../state/agentRuns.svelte.js';

  let sidebarOpen = $state(false);
  let prefillCwd = $state<string | undefined>(page.url.searchParams.get('cwd') ?? undefined);
  let prefillPrompt = $state<string | undefined>(undefined);
  let prefillTaskId = $state<string | null>(null);

  const focusedId = $derived(page.url.searchParams.get('session'));

  function syncUrl(sessionId: string | null) {
    const url = new URL(page.url.href);
    if (sessionId) url.searchParams.set('session', sessionId);
    else url.searchParams.delete('session');
    url.searchParams.delete('cwd');
    replaceState(url, {});
  }

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
    <AgentSessionSidebar {focusedId} onSelect={selectSession} onNewRun={newRun} />
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
      sessionId={focusedId}
      initialCwd={prefillCwd}
      initialPrompt={prefillPrompt}
      taskId={prefillTaskId}
      onStarted={(id) => syncUrl(id)}
    />
  </div>
</div>
