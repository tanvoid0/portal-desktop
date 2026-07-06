<script lang="ts">
  import type { Component } from "svelte";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";

  export interface PageStat {
    label: string;
    value: string | number;
    icon?: Component;
    description?: string;
  }

  interface Props {
    stats: PageStat[];
    columns?: 2 | 3 | 4;
  }

  let { stats, columns = 4 }: Props = $props();

  const gridClass = $derived(
    columns === 2
      ? "md:grid-cols-2"
      : columns === 3
        ? "md:grid-cols-3"
        : "md:grid-cols-4",
  );
</script>

<div class="grid grid-cols-1 gap-4 {gridClass}">
  {#each stats as stat}
    <Card>
      <CardHeader class="px-4 pb-1">
        <CardTitle class="flex items-center gap-2 text-sm font-medium">
          {#if stat.icon}
            {@const Icon = stat.icon}
            <Icon class="h-4 w-4 text-muted-foreground" />
          {/if}
          {stat.label}
        </CardTitle>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <div class="text-xl font-bold leading-none">{stat.value}</div>
        {#if stat.description}
          <p class="mt-1 text-xs text-muted-foreground">{stat.description}</p>
        {/if}
      </CardContent>
    </Card>
  {/each}
</div>
