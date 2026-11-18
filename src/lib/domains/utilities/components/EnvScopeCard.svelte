<script lang="ts">
  import type { Snippet } from 'svelte';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Plus from '~icons/fa6-solid/plus';
  import Search from '~icons/fa6-solid/magnifying-glass';

  interface Props {
    title: string;
    description: string;
    search: string;
    searchPlaceholder: string;
    addDisabled?: boolean;
    onadd: () => void;
    children?: Snippet;
  }
  let {
    title,
    description,
    search = $bindable(),
    searchPlaceholder,
    addDisabled = false,
    onadd,
    children,
  }: Props = $props();
</script>

<Card>
  <CardHeader class="gap-4 sm:flex-row sm:items-start sm:justify-between">
    <div class="space-y-1">
      <CardTitle>{title}</CardTitle>
      <CardDescription>{description}</CardDescription>
    </div>
    <div class="flex w-full flex-wrap gap-2 sm:w-auto">
      <div class="relative min-w-[12rem] flex-1 sm:flex-none">
        <Search class="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input bind:value={search} placeholder={searchPlaceholder} class="pl-8" />
      </div>
      <Button variant="outline" onclick={onadd} disabled={addDisabled}>
        <Plus class="mr-2 h-4 w-4" />
        Add variable
      </Button>
    </div>
  </CardHeader>
  <CardContent class="space-y-3">{@render children?.()}</CardContent>
</Card>
