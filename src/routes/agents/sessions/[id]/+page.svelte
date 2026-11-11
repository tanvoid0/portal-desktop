<!-- Session detail — Agents (metadata) / Conversation (user+assistant turns) /
     Timeline (raw event log) tabs over one persisted session. Read-only. -->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { createQuery } from '@tanstack/svelte-query';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { PageEmpty, PageError, PageHeader, PageLoading } from '$lib/components/shell';
  import CoderUserBlock from '$lib/domains/coder/components/CoderUserBlock.svelte';
  import CoderAssistantBlock from '$lib/domains/coder/components/CoderAssistantBlock.svelte';
  import type { ChatMessage } from '$lib/domains/coder/types';
  import { agentSessionQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { foldTimelineEvents } from '$lib/domains/agents/utils/eventTimeline';
  import { formatSessionDateTime, formatSessionDateTimeFull } from '$lib/domains/shared/utils';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import Inbox from '~icons/fa6-solid/inbox';

  const sessionId = $derived(page.params.id ?? '');
  const sessionQuery = createQuery(() => agentSessionQuery(sessionId));
  const detail = $derived(sessionQuery.data ?? null);

  const messages = $derived.by<ChatMessage[]>(() => {
    if (!detail) return [];
    return detail.messages.map((m) => ({ role: m.role, content: m.text }));
  });

  const timeline = $derived(detail ? foldTimelineEvents(detail.events) : []);
</script>

<svelte:head>
  <title>Agent Session - Portal Desktop</title>
</svelte:head>

<div class="h-full overflow-auto p-4 md:p-6">
  <div class="space-y-6">
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="sm" onclick={() => goto('/agents/sessions')}>
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back
      </Button>
    </div>

    {#if sessionQuery.isPending}
      <PageLoading message="Loading session…" />
    {:else if sessionQuery.isError}
      <PageError
        message={sessionQuery.error instanceof Error
          ? sessionQuery.error.message
          : 'Failed to load session'}
        onRetry={() => sessionQuery.refetch()}
      />
    {:else if !detail}
      <PageEmpty title="Session not found" description="It may have been pruned." icon={Inbox} />
    {:else}
      <PageHeader
        title={detail.session.backend_id}
        description={detail.session.cwd}
        badge={detail.session.status}
        badgeVariant={detail.session.status === 'running' ? 'default' : 'secondary'}
      />

      <Tabs value="agents">
        <TabsList>
          <TabsTrigger value="agents">Agents</TabsTrigger>
          <TabsTrigger value="conversation">Conversation</TabsTrigger>
          <TabsTrigger value="timeline">Timeline</TabsTrigger>
        </TabsList>

        <TabsContent value="agents" class="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Session</CardTitle>
            </CardHeader>
            <CardContent class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Backend</span>
                <span class="font-medium">{detail.session.backend_id}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Model</span>
                <span class="font-medium">{detail.session.model ?? '—'}</span>
              </div>
              <div class="flex justify-between gap-4">
                <span class="shrink-0 text-muted-foreground">Cwd</span>
                <span class="truncate text-right font-medium" title={detail.session.cwd}>
                  {detail.session.cwd}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Status</span>
                <Badge variant={detail.session.status === 'running' ? 'default' : 'secondary'}>
                  {detail.session.status}
                </Badge>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Started</span>
                <span
                  class="font-medium"
                  title={formatSessionDateTimeFull(detail.session.started_at)}
                >
                  {formatSessionDateTime(detail.session.started_at)}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Ended</span>
                <span
                  class="font-medium"
                  title={detail.session.ended_at
                    ? formatSessionDateTimeFull(detail.session.ended_at)
                    : ''}
                >
                  {detail.session.ended_at
                    ? formatSessionDateTime(detail.session.ended_at)
                    : 'Still running'}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Exit code</span>
                <span class="font-medium">
                  {detail.session.exit_code ?? '—'}
                </span>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="conversation">
          {#if messages.length === 0}
            <PageEmpty
              title="No messages"
              description="This session has no persisted conversation turns."
              icon={Inbox}
            />
          {:else}
            <ScrollArea class="h-[60vh]">
              <div class="max-w-3xl space-y-3 pr-4">
                {#each messages as message, i (i)}
                  {#if message.role === 'user'}
                    <CoderUserBlock {message} messageIndex={i} />
                  {:else}
                    <CoderAssistantBlock {message} />
                  {/if}
                {/each}
              </div>
            </ScrollArea>
          {/if}
        </TabsContent>

        <TabsContent value="timeline">
          {#if timeline.length === 0}
            <PageEmpty
              title="No events"
              description="This session has no persisted event log."
              icon={Inbox}
            />
          {:else}
            <ScrollArea class="h-[60vh]">
              <div class="space-y-2 pr-4">
                {#each timeline as item (item.id)}
                  <div class="rounded-md border p-2.5 text-xs">
                    <div class="flex items-center justify-between gap-2">
                      <span class="font-medium {item.isError ? 'text-destructive' : ''}">
                        {item.label}
                      </span>
                      <span class="shrink-0 text-muted-foreground">
                        {formatSessionDateTime(item.created_at)}
                      </span>
                    </div>
                    {#if item.detail}
                      <p class="mt-1 whitespace-pre-wrap text-muted-foreground">{item.detail}</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </ScrollArea>
          {/if}
        </TabsContent>
      </Tabs>
    {/if}
  </div>
</div>
