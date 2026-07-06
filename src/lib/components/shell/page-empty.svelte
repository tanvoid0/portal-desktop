<script lang="ts">
  import type { Component } from "svelte";
  import { Inbox } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";

  interface Props {
    title: string;
    description: string;
    filteredDescription?: string;
    isFiltered?: boolean;
    icon?: Component;
    actionLabel?: string;
    onAction?: () => void;
  }

  let {
    title,
    description,
    filteredDescription,
    isFiltered = false,
    icon: Icon = Inbox,
    actionLabel,
    onAction,
  }: Props = $props();

  const displayDescription = $derived(
    isFiltered && filteredDescription ? filteredDescription : description,
  );
</script>

<Card>
  <CardContent class="flex flex-col items-center justify-center py-12">
    <Icon class="mb-4 h-12 w-12 text-muted-foreground" />
    <h3 class="mb-2 text-lg font-semibold">{title}</h3>
    <p class="mb-4 max-w-md text-center text-muted-foreground">
      {displayDescription}
    </p>
    {#if actionLabel && onAction && !isFiltered}
      <Button onclick={onAction}>{actionLabel}</Button>
    {/if}
  </CardContent>
</Card>
