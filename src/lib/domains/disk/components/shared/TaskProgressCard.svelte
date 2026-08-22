<!--
	Progress for the removal half of the job (Recycle Bin moves, Docker prunes) —
	a simple done/total, distinct from the file walk that proposed the items.
-->

<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';

  let {
    progress,
    label = 'Moving to Recycle Bin',
  }: {
    progress: { done: number; total: number; currentPath?: string } | null;
    label?: string;
  } = $props();

  const pct = $derived(
    progress && progress.total > 0 ? Math.min(100, (progress.done / progress.total) * 100) : 0
  );
</script>

{#if progress}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">
          {label}<span class="text-muted-foreground"> · {progress.done} / {progress.total}</span>
        </span>
        <span class="tabular-nums text-muted-foreground">{pct.toFixed(0)}%</span>
      </div>
      <Progress value={pct} class="h-2" />
      {#if progress.currentPath}
        <div
          class="mt-1.5 truncate font-mono text-xs text-muted-foreground"
          title={progress.currentPath}
        >
          {progress.currentPath}
        </div>
      {/if}
    </CardContent>
  </Card>
{/if}
