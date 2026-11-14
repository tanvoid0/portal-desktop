<!-- Raw persisted event log for a session — lifted from the old
     `sessions/[id]` route's Timeline tab (#546). -->
<script lang="ts">
  import { PageEmpty } from '$lib/components/shell';
  import { formatSessionDateTime } from '$lib/domains/shared/utils';
  import { foldTimelineEvents } from '../utils/eventTimeline.js';
  import type { AgentEventRow } from '../generated';
  import Inbox from '~icons/fa6-solid/inbox';

  interface Props {
    events: AgentEventRow[];
  }

  let { events }: Props = $props();

  const timeline = $derived(foldTimelineEvents(events));
</script>

{#if timeline.length === 0}
  <PageEmpty
    title="No events"
    description="This session has no persisted event log."
    icon={Inbox}
  />
{:else}
  <div class="space-y-2">
    {#each timeline as item (item.id)}
      <div class="rounded-md border p-2.5 text-xs">
        <div class="flex items-center justify-between gap-2">
          <span class="font-medium {item.isError ? 'text-destructive' : ''}">{item.label}</span>
          <span class="shrink-0 text-muted-foreground tabular-nums">
            {formatSessionDateTime(item.created_at)}
          </span>
        </div>
        {#if item.detail}
          <p class="mt-1 whitespace-pre-wrap text-muted-foreground">{item.detail}</p>
        {/if}
      </div>
    {/each}
  </div>
{/if}
