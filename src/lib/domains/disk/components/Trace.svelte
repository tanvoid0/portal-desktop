<!-- Two-series sparkline over the last minute, newest at the right edge. -->
<script lang="ts">
  import { WINDOW_MS, type Sample } from '../state/history.svelte';

  interface Props {
    samples: Sample[];
    /** Value at the top edge. */
    max: number;
    label: string;
  }
  let { samples, max, label }: Props = $props();

  const lines = $derived.by(() => {
    const latest = samples.at(-1)?.at ?? 0;
    const points = (key: 'a' | 'b') =>
      samples
        .map(
          (s) =>
            `${100 - ((latest - s.at) / WINDOW_MS) * 100},${100 - Math.min(1, s[key] / max) * 98 - 1}`
        )
        .join(' ');
    return { a: points('a'), b: points('b') };
  });
</script>

<svg
  viewBox="0 0 100 100"
  preserveAspectRatio="none"
  class="h-24 w-full rounded-md bg-muted/40"
  role="img"
  aria-label={label}
>
  {#each [['text-primary', lines.a], ['text-status-warning', lines.b]] as [color, points] (color)}
    <polyline
      {points}
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
      class={color}
    />
  {/each}
</svg>
