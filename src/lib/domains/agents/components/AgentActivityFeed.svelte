<!-- Cross-session agent activity (#191): newest events from every run, paged older on demand. -->
<script lang="ts">
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { formatSessionDateTime } from '$lib/domains/shared/utils';
  import { agentEventsQuery } from '../queries/agentsQueries.js';
  import { foldTimelineEvents } from '../utils/eventTimeline.js';
  import type { AgentSessionSummary } from '../generated';

  interface Props {
    /** Sessions in scope (already backend-filtered); other sessions' events are hidden. */
    sessions: AgentSessionSummary[];
    onOpen?: (s: AgentSessionSummary) => void;
  }
  let { sessions, onOpen = () => {} }: Props = $props();

  const feed = createInfiniteQuery(agentEventsQuery);
  const byId = $derived(new Map(sessions.map((s) => [s.id, s])));
  const items = $derived(
    (feed.data?.pages.flat() ?? []).flatMap((e) => {
      const session = byId.get(e.session_id);
      const item = session
        ? foldTimelineEvents([
            { id: String(e.cursor), seq: e.cursor, created_at: e.created_at, event: e.event },
          ])[0]
        : undefined;
      return item ? [{ cursor: e.cursor, session: session!, item }] : [];
    })
  );
</script>

<section class="space-y-2">
  <h2 class="text-sm font-medium text-muted-foreground">Activity</h2>
  {#if feed.isError}
    <p class="text-sm text-destructive">{String(feed.error)}</p>
  {:else if feed.isPending}
    <p class="text-sm text-muted-foreground">Loading activity...</p>
  {:else if items.length === 0 && !feed.hasNextPage}
    <p class="text-sm text-muted-foreground">No activity yet.</p>
  {:else}
    <ul class="divide-y rounded-md border bg-card">
      {#each items as { cursor, session, item } (cursor)}
        <li>
          <Button
            variant="ghost"
            class="h-auto w-full min-w-0 justify-between gap-3 rounded-none px-3 py-2 text-left font-normal"
            onclick={() => onOpen(session)}
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm {item.isError ? 'text-destructive' : ''}">
                {item.label}{item.detail ? ` · ${item.detail}` : ''}
              </span>
              <span class="block truncate text-xs text-muted-foreground">
                {session.title || session.backend_id}
              </span>
            </span>
            <span class="shrink-0 text-xs text-muted-foreground">
              {formatSessionDateTime(item.created_at)}
            </span>
          </Button>
        </li>
      {/each}
    </ul>
    {#if feed.hasNextPage}
      <Button
        variant="outline"
        size="sm"
        disabled={feed.isFetchingNextPage}
        onclick={() => feed.fetchNextPage()}
      >
        Load older
      </Button>
    {/if}
  {/if}
</section>
