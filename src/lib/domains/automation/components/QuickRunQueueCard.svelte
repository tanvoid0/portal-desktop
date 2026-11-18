<!-- Ordered list of queued blocks with move / remove controls. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Trash2 from '~icons/fa6-solid/trash';
  import ChevronUp from '~icons/fa6-solid/chevron-up';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';

  interface Props {
    items: { id: string; name: string }[];
    onMove: (id: string, direction: -1 | 1) => void;
    onRemove: (id: string) => void;
  }

  let { items, onMove, onRemove }: Props = $props();
</script>

<SectionCard
  title="Run Queue"
  description={items.length === 0
    ? 'Add blocks below — they run in order'
    : `${items.length} block(s) queued`}
>
  {#if items.length === 0}
    <p class="py-6 text-center text-sm text-muted-foreground">
      No blocks queued. Use a preset or add from the library.
    </p>
  {:else}
    <div class="space-y-2">
      {#each items as item, index (item.id)}
        <div class="flex items-center gap-2 rounded-md border bg-muted/30 px-3 py-2">
          <span
            class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary"
          >
            {index + 1}
          </span>
          <span class="min-w-0 flex-1 truncate text-sm font-medium">
            {item.name}
          </span>
          <div class="flex shrink-0 gap-1">
            <Button
              variant="ghost"
              size="sm"
              class="h-7 w-7 p-0"
              disabled={index === 0}
              onclick={() => onMove(item.id, -1)}
            >
              <ChevronUp class="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 w-7 p-0"
              disabled={index === items.length - 1}
              onclick={() => onMove(item.id, 1)}
            >
              <ChevronDown class="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 w-7 p-0 text-destructive"
              onclick={() => onRemove(item.id)}
            >
              <Trash2 class="h-4 w-4" />
            </Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</SectionCard>
