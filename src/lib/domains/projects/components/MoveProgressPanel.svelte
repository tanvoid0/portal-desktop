<!-- In-flight move: phase label, progress bar and file/byte counters. -->
<script lang="ts">
  import { Progress } from '$lib/components/ui/progress';
  import { fmtBytes } from '$lib/domains/disk/utils';
  import type { MoveProgress } from '../api/projectMoveApi';

  interface Props {
    progress: MoveProgress | null;
    pct?: number;
    startError: string | null;
  }

  let { progress, pct, startError }: Props = $props();
</script>

<div class="space-y-2">
  <p class="text-sm font-medium">
    {progress?.phase === 'verifying' ? 'Verifying…' : 'Copying…'}
  </p>
  <Progress value={pct} class="w-full" />
  {#if progress}
    <p class="text-xs text-muted-foreground tabular-nums">
      {progress.files_done} / {progress.file_count} files · {fmtBytes(progress.bytes_done)} /
      {fmtBytes(progress.total_bytes)}
    </p>
  {:else}
    <p class="text-xs text-muted-foreground">Starting…</p>
  {/if}
</div>
{#if startError}
  <div
    class="rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
  >
    {startError}
  </div>
{/if}
