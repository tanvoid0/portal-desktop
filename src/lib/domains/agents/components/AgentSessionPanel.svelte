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
  import { Textarea } from '$lib/components/ui/textarea';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { PageContainer, PageLoading, PageError } from '$lib/components/shell';
  import CoderFeed from '$lib/domains/coder/components/CoderFeed.svelte';
  import ApprovalBanner from '$lib/domains/coder/components/ApprovalBanner.svelte';
  import RunLauncherForm from './RunLauncherForm.svelte';
  import AgentSessionTimeline from './AgentSessionTimeline.svelte';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { agentBackendsQuery, agentSessionQuery } from '../queries/agentsQueries.js';
  import { groupSessionsByProject } from '../utils/groupSessions.js';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { createTaskQuery } from '$lib/domains/tasks/queries/taskQueries';
  import { toast } from '$lib/utils/toast';

  interface Props {
    sessionId: string | null;
    initialCwd?: string;
    initialPrompt?: string;
    taskId?: string | null;
    onStarted: (id: string) => void;
  }

  let { sessionId, initialCwd, initialPrompt, taskId = null, onStarted }: Props = $props();

  const backendsQuery = createQuery(agentBackendsQuery);
  const sessionDetailQuery = createQuery(() => ({
    ...agentSessionQuery(sessionId ?? ''),
    enabled: !!sessionId,
  }));

  let starting = $state(false);
  let draft = $state('');
  let timelineView = $state<'transcript' | 'timeline'>('transcript');
  let scrollViewport = $state<HTMLElement | null>(null);
  let pinnedToBottom = $state(true);

  $effect(() => {
    if (sessionId) void agentRuns.attach(sessionId);
  });

  // A fresh session starts on the transcript with an empty composer, not
  // wherever the previously-focused one left off.
  $effect(() => {
    sessionId;
    draft = '';
    timelineView = 'transcript';
  });

  const view = $derived(sessionId ? agentRuns.viewFor(sessionId) : null);
  const detail = $derived(sessionDetailQuery.data ?? null);
  const title = $derived(detail?.session.title || detail?.session.backend_id || 'Session');
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

  async function handleStart(backendId: string, cwd: string, prompt: string, model: string | null) {
    starting = true;
    try {
      const id = await agentRuns.start(backendId, cwd, prompt, model, taskId);
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
      if (view?.ended) {
        const newId = await agentRuns.resume(id, text);
        onStarted(newId);
      } else {
        await agentRuns.sendUser(id, text);
      }
    } catch (e) {
      toast.error(String(e));
    }
  }

  function onComposerKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void send();
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
    <div class="flex min-h-0 flex-1 flex-col items-center justify-center overflow-y-auto px-4 py-8">
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
        <RunLauncherForm
          backends={backendsQuery.data ?? []}
          busy={starting}
          {initialCwd}
          {initialPrompt}
          onStart={handleStart}
        />
      {/if}
    </div>
  {:else}
    <div
      class="divider-edge-b flex shrink-0 flex-wrap items-center justify-between gap-2 px-3 py-2"
    >
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <h2 class="min-w-0 truncate text-sm font-semibold">{title}</h2>
        <Badge variant={view?.running ? 'default' : 'secondary'}>
          {view?.ended ? 'Ended' : view?.running ? 'Running' : (detail?.session.status ?? '…')}
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
        {#if view && !view.ended}
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
            showRemember={false}
            onDecision={(approve) =>
              firstPending && agentRuns.decide(sessionId ?? '', firstPending.call_id, approve)}
          />
          {#each extraPending as approval (approval.call_id)}
            <ApprovalBanner
              pending={approval}
              showRemember={false}
              onDecision={(approve) => agentRuns.decide(sessionId ?? '', approval.call_id, approve)}
            />
          {/each}
        </PageContainer>
      </ScrollArea>
    {/if}

    <div class="divider-edge-t flex shrink-0 items-end gap-2 p-3">
      <Textarea
        bind:value={draft}
        placeholder={view?.ended ? 'Send a message to resume…' : 'Send a follow-up…'}
        rows={2}
        class="flex-1 resize-none"
        onkeydown={onComposerKeydown}
      />
      {#if view?.running}
        <Button variant="outline" onclick={() => sessionId && agentRuns.interrupt(sessionId)}>
          Stop
        </Button>
      {:else}
        <Button disabled={!draft.trim()} onclick={send}>
          {view?.ended ? 'Resume' : 'Send'}
        </Button>
      {/if}
    </div>
  {/if}
</div>
