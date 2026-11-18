<script lang="ts">
  import type { CachedScan } from '../types';
  import { fmtBytes, fmtDate, fmtAgo } from '../utils';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';

  let {
    cached,
    onload,
    onrescan,
    ondiscard,
  }: {
    cached: CachedScan;
    onload: () => void;
    onrescan: () => void;
    ondiscard: () => void;
  } = $props();

  const partial = $derived(cached.status === 'partial');
  const pct = $derived(
    partial && cached.totalFiles > 0
      ? Math.min(100, Math.round((cached.scannedFiles / cached.totalFiles) * 100))
      : 100
  );
  const reclaimable = $derived(cached.summary.proposals.reduce((a, p) => a + p.sizeBytes, 0));
</script>

<Card class="mb-4 gap-0 {partial ? 'border-status-warning/30' : ''}">
  <CardContent class="flex items-start gap-3 px-4 py-3.5">
    <div
      class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-sm {partial
        ? 'bg-status-warning-bg text-status-warning'
        : 'bg-status-success-bg text-status-success'}"
    >
      {partial ? '◴' : '↻'}
    </div>
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-foreground"
          >{partial ? 'Interrupted scan can be restored' : 'Saved scan available'}</span
        >
        <Badge
          variant="outline"
          class={partial
            ? 'border-status-warning/30 bg-status-warning-bg text-status-warning'
            : 'border-status-success/30 bg-status-success-bg text-status-success'}
        >
          {partial ? `stopped at ${pct}%` : 'complete'}
        </Badge>
      </div>
      <div class="mt-1 text-xs text-muted-foreground">
        {fmtAgo(cached.ts)} · {fmtDate(cached.ts)} · {cached.scannedFiles.toLocaleString()}{partial &&
        cached.totalFiles > 0
          ? ` / ${cached.totalFiles.toLocaleString()}`
          : ''} files · {fmtBytes(reclaimable)} reclaimable
      </div>
      {#if partial}
        <Progress
          value={pct}
          class="mt-2 h-1.5 [&>[data-slot=progress-indicator]]:bg-status-warning"
        />
      {/if}
      <div class="mt-3 flex flex-wrap items-center gap-2">
        {#if partial}
          <Button size="sm" onclick={onrescan}>Resume — full scan</Button>
          <Button variant="outline" size="sm" onclick={onload}>Load partial results</Button>
        {:else}
          <Button size="sm" onclick={onload}>Load saved scan</Button>
          <Button variant="outline" size="sm" onclick={onrescan}>Rescan fresh</Button>
        {/if}
        <Button
          variant="outline"
          size="sm"
          onclick={ondiscard}
          class="text-destructive hover:text-destructive">Discard</Button
        >
      </div>
    </div>
  </CardContent>
</Card>
