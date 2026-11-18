<!-- Card with the usual icon + title + description header, optional header-right actions and body. -->
<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { cn } from '$lib/utils';

  interface Props {
    title: string;
    description?: string | Snippet;
    icon?: Component<{ class?: string }>;
    /** Right side of the header row (buttons, badges). */
    actions?: Snippet;
    children?: Snippet;
    class?: string;
    headerClass?: string;
    titleClass?: string;
    contentClass?: string;
  }

  let {
    title,
    description,
    icon: Icon,
    actions,
    children,
    class: className,
    headerClass,
    titleClass = 'text-lg',
    contentClass,
  }: Props = $props();
</script>

<Card class={className}>
  <CardHeader class={headerClass}>
    <div class="flex items-start justify-between gap-2">
      <div class="min-w-0">
        <CardTitle class={cn('flex items-center gap-2', titleClass)}>
          {#if Icon}<Icon class="h-5 w-5" />{/if}
          {title}
        </CardTitle>
        {#if description}
          <CardDescription>
            {#if typeof description === 'string'}{description}{:else}{@render description()}{/if}
          </CardDescription>
        {/if}
      </div>
      {@render actions?.()}
    </div>
  </CardHeader>
  {#if children}
    <CardContent class={contentClass}>{@render children()}</CardContent>
  {/if}
</Card>
