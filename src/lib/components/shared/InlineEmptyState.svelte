<!-- Card-less centered empty state: icon, headline, optional hint. -->
<script lang="ts">
  import type { Component } from 'svelte';
  import { cn } from '$lib/utils';

  interface Props {
    icon?: Component<{ class?: string }>;
    iconClass?: string;
    title: string;
    hint?: string;
    class?: string;
  }

  let {
    icon: Icon,
    iconClass = 'text-muted-foreground',
    title,
    hint,
    class: className = 'py-12',
  }: Props = $props();
</script>

<div class={cn('text-center', className)}>
  {#if Icon}<Icon class={cn('mx-auto mb-4 h-12 w-12', iconClass)} />{/if}
  <p class="text-muted-foreground {hint ? 'mb-2' : ''}">{title}</p>
  {#if hint}<p class="text-sm text-muted-foreground">{hint}</p>{/if}
</div>
