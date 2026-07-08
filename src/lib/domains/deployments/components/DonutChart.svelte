<script lang="ts">
  export interface DonutSegment {
    label: string;
    value: number;
    color: string;
  }

  let {
    segments,
    total,
    centerLabel,
    centerSubLabel = "",
    size = 140,
    stroke = 20,
  }: {
    segments: DonutSegment[];
    total: number;
    centerLabel: string;
    centerSubLabel?: string;
    size?: number;
    stroke?: number;
  } = $props();

  const r = $derived((size - stroke) / 2);
  const circ = $derived(2 * Math.PI * r);

  function dashArray(value: number) {
    const len = (total > 0 ? value / total : 0) * circ;
    return `${len} ${circ - len}`;
  }

  function dashOffset(idx: number) {
    let off = 0;
    for (let i = 0; i < idx; i++) {
      off += (total > 0 ? segments[i].value / total : 0) * circ;
    }
    return -off;
  }
</script>

<div class="flex items-center gap-4">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" class="shrink-0">
    <g transform="rotate(-90 {size / 2} {size / 2})">
      <circle
        cx={size / 2}
        cy={size / 2}
        {r}
        fill="none"
        class="stroke-muted"
        stroke-width={stroke}
      />
      {#each segments as s, i (s.label)}
        {#if s.value > 0}
          <circle
            cx={size / 2}
            cy={size / 2}
            {r}
            fill="none"
            stroke={s.color}
            stroke-width={stroke}
            stroke-dasharray={dashArray(s.value)}
            stroke-dashoffset={dashOffset(i)}
          />
        {/if}
      {/each}
    </g>
    <text
      x="50%"
      y={centerSubLabel ? "44%" : "50%"}
      text-anchor="middle"
      dominant-baseline="middle"
      class="fill-foreground"
      style="font-size:16px;font-weight:600"
    >
      {centerLabel}
    </text>
    {#if centerSubLabel}
      <text
        x="50%"
        y="58%"
        text-anchor="middle"
        class="fill-muted-foreground"
        style="font-size:9px;letter-spacing:0.06em"
      >
        {centerSubLabel}
      </text>
    {/if}
  </svg>

  <div class="min-w-0 flex-1 space-y-1.5">
    {#each segments as s (s.label)}
      <div class="flex items-center gap-2 text-xs">
        <span
          class="h-2.5 w-2.5 shrink-0 rounded-sm"
          style="background-color:{s.color}"
        ></span>
        <span class="truncate text-foreground">{s.label}</span>
        <span class="ml-auto tabular-nums text-muted-foreground">{s.value}</span>
      </div>
    {/each}
  </div>
</div>
