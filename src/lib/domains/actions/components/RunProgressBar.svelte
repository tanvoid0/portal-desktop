<script lang="ts">
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Ban from '~icons/fa6-solid/ban';
  import { etaText, isOverrun, segmentFraction } from '../runProgress';
  import type { RunProgress, RunSegment } from '../types';

  let { progress, compact = false }: { progress: RunProgress; compact?: boolean } = $props();

  const segs = $derived(progress.segments);
  const total = $derived(segs.reduce((n, s) => n + s.weight, 0) || 1);
  const failed = $derived(segs.find((s) => s.status === 'failed'));
  const cancelled = $derived(segs.find((s) => s.status === 'cancelled'));
  const current = $derived(segs.find((s) => s.status === 'running'));
  const active = $derived(
    segs.some((s) => s.status === 'running' || s.status === 'pending') && !failed && !cancelled
  );

  // Ticks only while the run is live; the backend event refreshes `progress` between ticks.
  let now = $state(Date.now());
  $effect(() => {
    if (!active) return;
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const index = $derived(
    Math.min(
      segs.findIndex((s) => s.status === 'running' || s.status === 'pending') + 1 || segs.length,
      segs.length
    )
  );
  const over = $derived(current ? isOverrun(current, now) : false);
  const estimateless = $derived(active && !progress.etaAt);
  const percent = $derived(Math.round(progress.percent));

  const summary = $derived.by(() => {
    if (failed) return `Failed at ${failed.name}`;
    if (cancelled) return `Cancelled at ${cancelled.name}`;
    if (!active) return 'Complete';
    const step = current?.label ?? current?.name ?? segs[index - 1]?.name ?? '';
    return `Step ${index} of ${segs.length} · ${step} · ${over ? 'running over' : etaText(progress.etaAt, now)}`;
  });

  const secs = (ms: number) => `${Math.round(ms / 1000)}s`;
  function tip(s: RunSegment) {
    const took =
      s.startedAt && s.status !== 'pending'
        ? secs((s.status === 'running' ? now : Date.parse(s.startedAt)) - Date.parse(s.startedAt))
        : null;
    return [
      s.name,
      s.status,
      took && `took ${took}`,
      s.expectedMs && `expected ${secs(s.expectedMs)}`,
    ]
      .filter(Boolean)
      .join(' · ');
  }
</script>

<div
  role="progressbar"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={percent}
  aria-valuetext={summary}
  class="w-full space-y-1"
>
  <div class="flex w-full gap-[2px] {compact ? 'h-1.5' : 'h-2.5'}">
    {#each segs as s (s.stepId)}
      <div
        title={tip(s)}
        class="relative min-w-[6px] overflow-hidden rounded-[4px] {s.status === 'success'
          ? 'bg-emerald-500'
          : s.status === 'failed'
            ? 'bg-destructive'
            : s.status === 'cancelled'
              ? 'border border-muted-foreground/60'
              : 'bg-muted'} {s.status === 'running' && estimateless
          ? 'motion-safe:animate-pulse'
          : ''}"
        style="flex: {s.weight / total} 1 0"
      >
        {#if s.status === 'running'}
          <div
            class="h-full bg-primary"
            style="width: {Math.round(segmentFraction(s, now) * 100)}%"
          ></div>
        {/if}
      </div>
    {/each}
  </div>
  <p
    class="flex items-center gap-1 text-xs text-muted-foreground tabular-nums {failed
      ? 'text-destructive'
      : ''}"
  >
    {#if failed}<XCircle class="h-3 w-3 shrink-0" />{:else if cancelled}<Ban
        class="h-3 w-3 shrink-0"
      />{/if}
    <span class="truncate">{summary}</span>
  </p>
</div>
