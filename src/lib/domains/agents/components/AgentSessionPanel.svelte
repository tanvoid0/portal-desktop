<!-- Main panel of the Agents workspace (#546): the launcher when no session
     is focused, a live transcript/timeline + composer when one is. Reuses
     Coder's feed/approval leaves — see `eventsToMessages.ts` for the shape
     that bridges a native `AgentEvent` stream onto them. -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { goto } from '$app/navigation';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { PageContainer, PageLoading, PageError } from '$lib/components/shell';
  import CoderFeed from '$lib/domains/coder/components/CoderFeed.svelte';
  import ApprovalBanner from '$lib/domains/coder/components/ApprovalBanner.svelte';
  import CoderComposer from '$lib/domains/coder/components/CoderComposer.svelte';
  import CoderEmptyHero from '$lib/domains/coder/components/CoderEmptyHero.svelte';
  import CoderProjectSelector from '$lib/domains/coder/components/CoderProjectSelector.svelte';
  import Select from '$lib/components/ui/select.svelte';
  import { Input } from '$lib/components/ui/input';
  import AgentSessionTimeline from './AgentSessionTimeline.svelte';
  import { agentCapabilities } from '$lib/domains/coder/config/capabilities.js';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { agentBackendsQuery, agentSessionQuery } from '../queries/agentsQueries.js';
  import { groupSessionsByProject } from '../utils/groupSessions.js';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { createTaskQuery } from '$lib/domains/tasks/queries/taskQueries';
  import { toast } from '$lib/utils/toast';

  const CWD_STORAGE_KEY = 'portal-agents-last-cwd';

  interface Props {
    backendId: string;
    sessionId: string | null;
    initialCwd?: string;
    initialPrompt?: string;
    taskId?: string | null;
    onStarted: (id: string) => void;
  }

  let {
    backendId,
    sessionId,
    initialCwd,
    initialPrompt,
    taskId = null,
    onStarted,
  }: Props = $props();

  const backendsQuery = createQuery(agentBackendsQuery);
  const backend = $derived((backendsQuery.data ?? []).find((b) => b.id === backendId));
  const sessionDetailQuery = createQuery(() => ({
    ...agentSessionQuery(sessionId ?? ''),
    enabled: !!sessionId,
  }));

  let starting = $state(false);
  let draft = $state('');
  // Launch settings for a new run; the model is fixed once the run starts (QA-085).
  let cwd = $state(initialCwd ?? '');
  let modelName = $state('');
  let timelineView = $state<'transcript' | 'timeline'>('transcript');
  let scrollViewport = $state<HTMLElement | null>(null);
  let pinnedToBottom = $state(true);

  $effect(() => {
    if (sessionId) void agentRuns.attach(sessionId);
  });

  // A fresh session starts on the transcript with an empty composer, not
  // wherever the previously-focused one left off.
  $effect(() => {
    draft = sessionId ? '' : (initialPrompt ?? '');
    if (!sessionId && initialCwd) cwd = initialCwd;
    timelineView = 'transcript';
  });

  const caps = $derived(agentCapabilities(backendId));
  const view = $derived(sessionId ? agentRuns.viewFor(sessionId) : null);
  const detail = $derived(sessionDetailQuery.data ?? null);
  const title = $derived(detail?.session.title || detail?.session.backend_id || 'Session');
  // Imported sessions never emit `exited`, so the view alone says "not ended".
  const ended = $derived(!!sessionId && (!!view?.ended || !agentRuns.runningIds.has(sessionId)));
  const firstPending = $derived(view?.pending[0] ?? null);
  const extraPending = $derived(view?.pending.slice(1) ?? []);

  // Project/task chips — same cwd→project matching the sidebar groups by,
  // scoped to just this one session.
  const projectsQuery = createProjectsQuery();
  const projectMatch = $derived.by(() => {
    if (!detail) return null;
    const [group] = groupSessionsByProject([detail.session], projectsQuery.data ?? []);
    return group?.projectId ? { id: group.projectId, label: group.label } : null;
  });
  const taskQuery = createTaskQuery(() => detail?.session.task_id ?? null);

  // Only autoscroll the feed if the user hadn't scrolled away — a high-frequency
  // stream forcing the viewport down every delta is the rough edge CoderPanel
  // has today; don't copy it here.
  $effect(() => {
    const el = scrollViewport;
    if (!el) return;
    const onScroll = () => {
      pinnedToBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
    };
    el.addEventListener('scroll', onScroll);
    onScroll();
    return () => el.removeEventListener('scroll', onScroll);
  });

  $effect(() => {
    view?.messages.length;
    view?.streamingText;
    if (scrollViewport && pinnedToBottom) {
      const el = scrollViewport;
      requestAnimationFrame(() => (el.scrollTop = el.scrollHeight));
    }
  });

  async function handleStart() {
    const prompt = draft.trim();
    if (!prompt) return;
    if (!cwd.trim()) {
      toast.error('Select a project first');
      return;
    }
    starting = true;
    try {
      const id = await agentRuns.start(
        backendId,
        cwd.trim(),
        prompt,
        modelName.trim() || null,
        taskId
      );
      draft = '';
      onStarted(id);
    } catch (e) {
      toast.error(String(e));
    } finally {
      starting = false;
    }
  }

  async function send() {
    const id = sessionId;
    const text = draft.trim();
    if (!id || !text) return;
    draft = '';
    try {
      if (ended) {
        const newId = await agentRuns.resume(id, text);
        onStarted(newId);
      } else {
        await agentRuns.sendUser(id, text);
      }
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleCancel() {
    if (!sessionId) return;
    try {
      await agentRuns.cancel(sessionId);
    } catch (e) {
      toast.error(String(e));
    }
  }
</script>

<div class="flex min-h-0 flex-1 flex-col overflow-hidden">
  {#if !sessionId}
    <div
      class="flex min-h-0 flex-1 flex-col items-center justify-center overflow-y-auto px-4 pb-8 [@media(max-height:700px)]:pb-2"
    >
      {#if backendsQuery.isPending}
        <PageLoading message="Loading agent backends…" />
      {:else if backendsQuery.isError}
        <PageError
          message={backendsQuery.error instanceof Error
            ? backendsQuery.error.message
            : 'Failed to load agent backends'}
          onRetry={() => backendsQuery.refetch()}
        />
      {:else}
        <CoderEmptyHero />
        <div class="w-full">
          <PageContainer variant="chat" class="px-0">
            <div class="mb-2 px-4">
              <CoderProjectSelector bind:value={cwd} storageKey={CWD_STORAGE_KEY} />
            </div>
            {#if backend && !backend.available}
              <p class="mb-2 px-4 text-xs text-destructive">
                {backend.display_name} wasn't found on PATH — install it to run this.
              </p>
            {/if}
            <CoderComposer
              bind:value={draft}
              onSend={handleStart}
              rows={3}
              placeholder="What should the agent do?"
              disabled={starting}
              workspaceRoot={cwd}
              class="px-0 pb-0"
            >
              {#snippet model()}
                <!-- Launch-only: chosen here, fixed for the run. A backend with no enumerable
                     model list gets free text so the user isn't stuck on its default. -->
                {#if backend && backend.models.length > 0}
                  <Select
                    aria-label="Model"
                    options={backend.models.map((m) => ({ value: m.id, label: m.label }))}
                    bind:value={modelName}
                    placeholder="Default model"
                    class="h-8 w-44 text-xs"
                  />
                {:else}
                  <Input
                    aria-label="Model"
                    bind:value={modelName}
                    placeholder="Default model"
                    class="h-8 w-40 text-xs"
                  />
                {/if}
              {/snippet}
            </CoderComposer>
          </PageContainer>
        </div>
      {/if}
    </div>
  {:else}
    <div
      class="divider-edge-b flex shrink-0 flex-wrap items-center justify-between gap-2 px-3 py-2"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <h2 class="min-w-0 truncate text-sm font-semibold">{title}</h2>
        <Badge variant={view?.running ? 'default' : 'secondary'}>
          {ended ? 'Ended' : view?.running ? 'Running' : (detail?.session.status ?? '…')}
        </Badge>
        {#if detail}
          <span class="hidden shrink-0 text-xs text-muted-foreground sm:inline">
            {detail.session.backend_id}{detail.session.model ? ` · ${detail.session.model}` : ''}
          </span>
        {/if}
        {#if view?.totalCostUsd != null}
          <span class="shrink-0 text-xs text-muted-foreground tabular-nums">
            ${view.totalCostUsd.toFixed(4)}
          </span>
        {/if}
        {#if projectMatch}
          <Badge
            variant="outline"
            class="shrink-0 cursor-pointer text-xs hover:bg-accent"
            onclick={() => goto(`/projects/${projectMatch.id}`)}
          >
            {projectMatch.label}
          </Badge>
        {:else if detail}
          <span
            class="hidden max-w-[12rem] shrink-0 truncate text-xs text-muted-foreground sm:inline"
            title={detail.session.cwd}
          >
            {detail.session.cwd}
          </span>
        {/if}
        {#if taskQuery.data}
          <Badge
            variant="outline"
            class="max-w-[12rem] shrink-0 cursor-pointer truncate text-xs hover:bg-accent"
            onclick={() => goto(`/tasks/${taskQuery.data!.id}`)}
          >
            {taskQuery.data.title}
          </Badge>
        {/if}
      </div>

      <div class="flex shrink-0 items-center gap-2">
        <Tabs value={timelineView} onValueChange={(v) => (timelineView = v as typeof timelineView)}>
          <TabsList class="h-8">
            <TabsTrigger value="transcript" class="px-2.5 text-xs">Transcript</TabsTrigger>
            <TabsTrigger value="timeline" class="px-2.5 text-xs">Timeline</TabsTrigger>
          </TabsList>
        </Tabs>
        {#if view && !ended}
          <Button size="sm" variant="outline" onclick={handleCancel}>End run</Button>
        {/if}
      </div>
    </div>

    {#if timelineView === 'timeline'}
      <ScrollArea class="min-h-0 flex-1">
        <PageContainer variant="chat" class="py-4">
          <AgentSessionTimeline events={detail?.events ?? []} />
        </PageContainer>
      </ScrollArea>
    {:else}
      <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollViewport}>
        <PageContainer variant="chat" class="space-y-3 py-4">
          <CoderFeed
            messages={view?.messages ?? []}
            streamingText={view?.streamingText ?? ''}
            running={view?.running ?? false}
            pending={firstPending}
            showRemember={caps.rememberRule}
            onDecision={(approve) =>
              firstPending && agentRuns.decide(sessionId ?? '', firstPending.call_id, approve)}
          />
          {#each extraPending as approval (approval.call_id)}
            <ApprovalBanner
              pending={approval}
              showRemember={caps.rememberRule}
              onDecision={(approve) => agentRuns.decide(sessionId ?? '', approval.call_id, approve)}
            />
          {/each}
        </PageContainer>
      </ScrollArea>
    {/if}

    <div class="shrink-0 bg-gradient-to-t from-background via-background to-transparent pt-2">
      <CoderComposer
        bind:value={draft}
        onSend={send}
        onStop={() => sessionId && void agentRuns.interrupt(sessionId)}
        placeholder={ended ? 'Send a message to resume…' : 'Send a follow-up…'}
        running={view?.running ?? false}
        canStop={(view?.running ?? false) || !!firstPending}
        workspaceRoot={detail?.session.cwd ?? ''}
        class="pt-0"
      />
    </div>
  {/if}
</div>
