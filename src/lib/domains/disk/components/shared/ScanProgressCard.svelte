<!--
	Progress for a filesystem walk. Every scan in this domain reports the same
	two phases over the same `scan://progress` event, so they show the same card
	— a walk that looks different per tab reads as a different operation.
-->

<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import type { ScanProgress } from '../../types';
  import { fmtBytes, fmtDuration } from '../../utils';

  let {
    progress,
    label = 'Scanning',
    showPath = true,
  }: { progress: ScanProgress | null; label?: string; showPath?: boolean } = $props();

  const counting = $derived(progress?.phase === 'counting');
  const pct = $derived(
    progress && !counting && progress.totalFiles > 0
      ? Math.min(100, (progress.scannedFiles / progress.totalFiles) * 100)
      : 0
  );
</script>

{#if progress}
  <Card class="mb-4 gap-0 py-4">
    <CardContent class="px-4">
      <div class="mb-2 flex items-center justify-between text-sm">
        <span class="text-foreground">
          {counting ? 'Counting files…' : label}
          <span class="text-muted-foreground">
            {#if counting}
              · {progress.scannedFiles.toLocaleString()} found
            {:else}
              · {progress.scannedFiles.toLocaleString()} / {progress.totalFiles.toLocaleString()} · {fmtBytes(
                progress.totalBytes
              )}
            {/if}
          </span>
        </span>
        <span class="text-muted-foreground tabular-nums">
          {counting
            ? fmtDuration(progress.elapsedMs)
            : `${pct.toFixed(0)}% · ETA ${fmtDuration(progress.etaMs)}`}
        </span>
      </div>
      <Progress value={counting ? 33 : pct} class="h-2 {counting ? 'animate-pulse' : ''}" />
      {#if showPath && progress.currentPath}
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
