<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import type { TriageDecision } from '$lib/domains/projects/api/discoveryApi';
  import { DECISIONS, DESTRUCTIVE_CLASS } from '$lib/domains/projects/utils/auditFormat';

  interface Props {
    count: number;
    onDecide: (d: TriageDecision) => void;
    onClear: () => void;
  }
  let { count, onDecide, onClear }: Props = $props();
</script>

<section class="space-y-2 border-t pt-3">
  <div class="flex items-center justify-between gap-2">
    <h3 class="text-xs font-semibold text-muted-foreground uppercase">Marked</h3>
    <span class="text-xs tabular-nums">{count} selected</span>
  </div>
  {#if count}
    <div class="grid grid-cols-2 gap-1">
      {#each DECISIONS as d (d)}
        <Button
          size="sm"
          variant="outline"
          class={d === 'delete' ? DESTRUCTIVE_CLASS : ''}
          onclick={() => onDecide(d)}>{d} all</Button
        >
      {/each}
    </div>
    <Button size="sm" variant="ghost" class="w-full" onclick={onClear}>clear</Button>
  {:else}
    <p class="text-xs text-muted-foreground">
      Press <span class="font-mono">x</span> on a row to tick it for a bulk decision.
    </p>
  {/if}
</section>
