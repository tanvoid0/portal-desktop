<!-- Run launcher + live conversation. Reuses coder's feed/approval components
     via `eventsToMessages` rather than a parallel chat UI (see
     `$lib/domains/agents/utils/eventsToMessages.ts`). -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { createQuery } from '@tanstack/svelte-query';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { PageError, PageLoading } from '$lib/components/shell';
  import CoderFeed from '$lib/domains/coder/components/CoderFeed.svelte';
  import ApprovalBanner from '$lib/domains/coder/components/ApprovalBanner.svelte';
  import RunLauncherForm from '$lib/domains/agents/components/RunLauncherForm.svelte';
  import { createAgentRunState } from '$lib/domains/agents/state/agentRun.svelte.js';
  import { agentBackendsQuery } from '$lib/domains/agents/queries/agentsQueries.js';

  const run = createAgentRunState();
  const backendsQuery = createQuery(agentBackendsQuery);
  let draft = $state('');

  // The first pending approval renders inside `CoderFeed` (its existing single-slot
  // prop, shared with the coder domain); any concurrent extras render as their own
  // banners right after it — see `eventsToMessages.ts` for why `pending` is a list.
  const firstPending = $derived(run.view.pending[0] ?? null);
  const extraPending = $derived(run.view.pending.slice(1));

  onMount(() => () => run.destroy());

  function handleStart(backendId: string, cwd: string, prompt: string, model: string | null) {
    run.start(backendId, cwd, prompt, model);
  }

  function send() {
    const text = draft.trim();
    if (!text || !run.runId) return;
    draft = '';
    run.sendUser(text);
  }

  function onComposerKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }
</script>

<div class="flex h-full w-full flex-col overflow-hidden">
  {#if !run.runId}
    {#if backendsQuery.isPending}
      <PageLoading message="Loading agent backends…" />
    {:else if backendsQuery.isError}
      <div class="p-4">
        <PageError
          message={backendsQuery.error instanceof Error
            ? backendsQuery.error.message
            : 'Failed to load agent backends'}
          onRetry={() => backendsQuery.refetch()}
        />
      </div>
    {:else}
      <ScrollArea class="min-h-0 flex-1">
        <RunLauncherForm
          backends={backendsQuery.data ?? []}
          busy={run.starting}
          onStart={handleStart}
        />
      </ScrollArea>
    {/if}
  {:else}
    <div
      class="divider-edge-b flex shrink-0 items-center justify-between gap-2 px-3 py-2 text-xs text-muted-foreground"
    >
      <span>{run.view.ended ? 'Run ended' : 'Run in progress'}</span>
      {#if run.view.ended}
        <Button size="sm" variant="outline" onclick={() => run.reset()}>New run</Button>
      {:else}
        <Button size="sm" variant="outline" onclick={() => run.cancel()}>End run</Button>
      {/if}
    </div>

    <ScrollArea class="min-h-0 flex-1">
      <div class="mx-auto max-w-3xl space-y-3 p-4">
        <CoderFeed
          messages={run.view.messages}
          streamingText={run.view.streamingText}
          running={run.view.running}
          pending={firstPending}
          showRemember={false}
          onDecision={(approve) => firstPending && run.decide(firstPending.call_id, approve)}
        />
        {#each extraPending as approval (approval.call_id)}
          <ApprovalBanner
            pending={approval}
            showRemember={false}
            onDecision={(approve) => run.decide(approval.call_id, approve)}
          />
        {/each}
      </div>
    </ScrollArea>

    <div class="divider-edge-t flex shrink-0 items-end gap-2 p-3">
      <Textarea
        bind:value={draft}
        placeholder={run.view.ended ? 'Run has ended.' : 'Send a follow-up…'}
        rows={2}
        disabled={run.view.ended}
        class="flex-1 resize-none"
        onkeydown={onComposerKeydown}
      />
      {#if run.view.running}
        <Button variant="outline" onclick={() => run.interrupt()}>Stop</Button>
      {:else}
        <Button disabled={!draft.trim() || run.view.ended} onclick={send}>Send</Button>
      {/if}
    </div>
  {/if}

  {#if run.error}
    <div
      class="shrink-0 border-t border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive"
    >
      {run.error}
    </div>
  {/if}
</div>
