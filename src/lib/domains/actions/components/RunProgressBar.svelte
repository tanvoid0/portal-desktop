<!--
  Run progress: one equal-width segment per step. The full view labels every
  step; compact is a single row for run lists and release cards.
-->
<script lang="ts">
  import CircleCheck from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Ban from '~icons/fa6-solid/ban';
  import Forward from '~icons/fa6-solid/forward';
  import Spinner from '~icons/fa6-solid/spinner';
  import { cn } from '$lib/utils';
  import { durationText, etaText, isOverrun, segmentFraction } from '../runProgress';
  import type { RunProgress, RunSegment } from '../types';

  let { progress, compact = false }: { progress: RunProgress; compact?: boolean } = $props();

  const segs = $derived(progress.segments);
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
  const totalMs = $derived(segs.reduce((n, s) => n + (s.durationMs ?? 0), 0));

  const title = $derived.by(() => {
    if (failed) return `Failed at ${failed.name}`;
    if (cancelled) return `Cancelled at ${cancelled.name}`;
    if (!active) return 'Complete';
    return current?.label ?? current?.name ?? segs[index - 1]?.name ?? '';
  });
  const timing = $derived.by(() => {
    if (!active) return totalMs ? durationText(totalMs) : '';
    return over ? 'running over' : etaText(progress.etaAt, now);
  });
  const count = $derived(active ? `step ${index} of ${segs.length}` : '');
  const summary = $derived([title, count, timing].filter(Boolean).join(' · '));

  function stepTime(s: RunSegment): string {
    const expected = s.expectedMs ? `~${durationText(s.expectedMs)}` : '';
    if (s.status === 'pending') return expected;
    if (s.status === 'running') {
      if (!s.startedAt) return expected;
      const elapsed = durationText(Math.max(0, now - Date.parse(s.startedAt)));
      return expected ? `${elapsed} / ${expected}` : elapsed;
    }
    return s.durationMs != null ? durationText(s.durationMs) : s.status;
  }
  const tip = (s: RunSegment) => [s.name, s.status, stepTime(s)].filter(Boolean).join(' · ');
</script>

{#snippet bar(s: RunSegment)}
  <div
    class={cn(
      'overflow-hidden rounded-full bg-muted',
      compact ? 'h-1.5' : 'h-1',
      s.status === 'success' && 'bg-emerald-500',
      s.status === 'failed' && 'bg-destructive',
      s.status === 'skipped' && 'bg-muted-foreground/40',
      s.status === 'cancelled' && 'border border-muted-foreground/60 bg-transparent',
      s.status === 'running' && estimateless && 'motion-safe:animate-pulse'
    )}
  >
    {#if s.status === 'running'}
      <div
        class="h-full rounded-full bg-primary"
        style="width: {Math.round(segmentFraction(s, now) * 100)}%"
      ></div>
    {/if}
  </div>
{/snippet}

<div
  role="progressbar"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={percent}
  aria-valuetext={summary}
  class="w-full text-xs tabular-nums"
>
  {#if compact}
    <div class="flex items-center gap-2">
      <div class="flex w-28 shrink-0 gap-[3px]">
        {#each segs as s (s.stepId)}
          <div class="min-w-0 flex-1" title={tip(s)}>{@render bar(s)}</div>
        {/each}
      </div>
      <span class={cn('flex min-w-0 items-center gap-1 font-medium', failed && 'text-destructive')}>
        {#if failed}
          <XCircle class="h-3 w-3 shrink-0" />
        {:else if cancelled}
          <Ban class="h-3 w-3 shrink-0" />
        {/if}
        <span class="truncate">{title}</span>
      </span>
      {#if active}
        <span class="shrink-0 text-muted-foreground">{index}/{segs.length}</span>
      {/if}
      <span class="ml-auto shrink-0 text-muted-foreground">{timing}</span>
    </div>
  {:else}
    <div class="mb-2 flex items-baseline justify-between gap-3 text-sm">
      <span class={cn('min-w-0 truncate font-medium', failed && 'text-destructive')}>
        {title}
        {#if count}<span class="ml-1 font-normal text-muted-foreground">{count}</span>{/if}
      </span>
      <span class="shrink-0 text-muted-foreground">
        {active ? `${percent}% · ${timing}` : timing}
      </span>
    </div>
    <!-- Wraps onto more rows when there are too many steps for one. -->
    <ol class="grid grid-cols-[repeat(auto-fit,minmax(7rem,1fr))] gap-x-1.5 gap-y-3">
      {#each segs as s (s.stepId)}
        <li class="min-w-0" title={tip(s)}>
          {@render bar(s)}
          <div
            class={cn(
              'mt-1.5 flex items-center gap-1.5',
              s.status === 'pending' && 'text-muted-foreground',
              s.status === 'running' && 'font-medium',
              s.status === 'failed' && 'text-destructive'
            )}
          >
            {#if s.status === 'success'}
              <CircleCheck class="h-3 w-3 shrink-0 text-emerald-500" />
            {:else if s.status === 'failed'}
              <XCircle class="h-3 w-3 shrink-0" />
            {:else if s.status === 'cancelled'}
              <Ban class="h-3 w-3 shrink-0" />
            {:else if s.status === 'skipped'}
              <Forward class="h-3 w-3 shrink-0 text-muted-foreground" />
            {:else if s.status === 'running'}
              <Spinner class="h-3 w-3 shrink-0 text-primary motion-safe:animate-spin" />
            {:else}
              <span class="h-3 w-3 shrink-0 rounded-full border border-current"></span>
            {/if}
            <span class="truncate">{s.name}</span>
          </div>
          <div class="truncate text-muted-foreground">{stepTime(s)}</div>
        </li>
      {/each}
    </ol>
  {/if}
</div>
