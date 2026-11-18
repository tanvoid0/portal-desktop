<!-- Post-move summary with the unrewritten stored paths and the remove-old-folder action. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Loader2 from '~icons/fa6-solid/spinner';
  import type { MoveSummary } from '../api/projectMoveApi';

  interface Props {
    summary: MoveSummary;
    removing: boolean;
    onRemoveSource: () => void;
  }

  let { summary, removing, onRemoveSource }: Props = $props();
</script>

<div class="rounded-md border border-border bg-muted/30 px-3 py-2 text-sm">
  Moved to <span class="font-mono text-xs">{summary.dest_path}</span>.
</div>
{#if summary.unrewritten.length > 0}
  <div
    class="rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-200"
  >
    <p class="mb-1 font-medium">These stored paths couldn't be rewritten automatically</p>
    <ul class="list-disc space-y-0.5 pl-4 font-mono text-xs">
      {#each summary.unrewritten as path (path)}
        <li>{path}</li>
      {/each}
    </ul>
    <p class="mt-1">Fix them by hand where they're used.</p>
  </div>
{/if}
<Button variant="outline" class="w-fit gap-2" disabled={removing} onclick={onRemoveSource}>
  {#if removing}<Loader2 class="h-4 w-4 animate-spin" />{/if}
  Remove old folder…
</Button>
