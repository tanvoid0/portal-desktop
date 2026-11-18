<!-- Card with title/description header and a spaced content body; the standard settings section. -->
<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';

  interface Props {
    title: string;
    description?: string;
    icon?: Component<{ class?: string }>;
    contentClass?: string;
    children: Snippet;
  }

  let { title, description, icon: Icon, contentClass = 'space-y-4', children }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle class={Icon ? 'flex items-center gap-2' : ''}>
      {#if Icon}<Icon class="h-5 w-5" />{/if}
      {title}
    </CardTitle>
    {#if description}<CardDescription>{description}</CardDescription>{/if}
  </CardHeader>
  <CardContent class={contentClass}>
    {@render children()}
  </CardContent>
</Card>
