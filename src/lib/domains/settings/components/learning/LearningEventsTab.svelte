<!-- Recent learning events list. -->
<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import type { LearningEvent } from '$lib/domains/learning';
  import Clock from '~icons/fa6-solid/clock';
  import { formatDate, formatPatternData, getEventTypeLabel } from './format';

  interface Props {
    events: LearningEvent[];
  }

  let { events }: Props = $props();
</script>

<div class="max-h-[600px] space-y-2 overflow-y-auto">
  {#each events as event}
    <Card class="p-3">
      <div class="flex items-start justify-between gap-4">
        <div class="flex-1 space-y-1">
          <div class="flex items-center gap-2">
            <Badge variant="outline">
              {getEventTypeLabel(event.event_type)}
            </Badge>
            {#if event.outcome}
              <Badge variant={event.outcome === 'success' ? 'default' : 'destructive'}>
                {event.outcome}
              </Badge>
            {/if}
            {#if event.context}
              <Badge variant="secondary">
                {event.context}
              </Badge>
            {/if}
          </div>
          <div class="max-h-24 overflow-y-auto rounded bg-muted p-2 font-mono text-xs">
            {formatPatternData(event.event_data)}
          </div>
          <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <Clock class="h-3 w-3" />
            {formatDate(event.created_at)}
          </div>
        </div>
      </div>
    </Card>
  {/each}
</div>
