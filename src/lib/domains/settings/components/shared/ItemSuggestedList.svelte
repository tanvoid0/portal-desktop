<!-- Suggested items grouped by category, with per-item and add-all actions. -->
<script lang="ts" generics="TSuggested extends BaseSuggestedItem">
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Plus from '~icons/fa6-solid/plus';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import TrendingUp from '~icons/fa6-solid/arrow-trend-up';
  import Icon from '@iconify/svelte';
  import type { Component } from 'svelte';
  import type { BaseSuggestedItem, BaseItemGroup } from './types';

  interface Props {
    groups: BaseItemGroup<TSuggested>[];
    hasAnySuggestions: boolean;
    itemName: string;
    itemNamePlural: string;
    emptyIcon: Component<{ class?: string }>;
    isLoading: boolean;
    onAddAll: (group: BaseItemGroup<TSuggested>) => void;
    onAdd: (item: TSuggested) => void;
  }

  let {
    groups,
    hasAnySuggestions,
    itemName,
    itemNamePlural,
    emptyIcon: EmptyIcon,
    isLoading,
    onAddAll,
    onAdd,
  }: Props = $props();
</script>

{#if groups.length === 0}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-12">
      <EmptyIcon class="mb-4 h-12 w-12 text-muted-foreground" />
      <p class="text-muted-foreground">
        {#if !hasAnySuggestions}
          No suggested {itemNamePlural.toLowerCase()} available
        {:else}
          All suggested {itemNamePlural.toLowerCase()} have been added
        {/if}
      </p>
    </CardContent>
  </Card>
{:else}
  {#each groups as group}
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            {#if group.category.includes('Recommended')}
              <Sparkles class="h-5 w-5 text-primary" />
            {/if}
            <div>
              <CardTitle>{group.category}</CardTitle>
              <CardDescription>
                {group.items.length}
                {itemName.toLowerCase()}{group.items.length > 1 ? 's' : ''} available
              </CardDescription>
            </div>
          </div>
          <Button variant="outline" size="sm" onclick={() => onAddAll(group)} disabled={isLoading}>
            <Plus class="mr-2 h-4 w-4" />
            Add All
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
          {#each group.items as item}
            <div
              class="flex items-center justify-between rounded-lg border p-3 transition-colors hover:bg-accent/50"
            >
              <div class="flex min-w-0 flex-1 items-center gap-3">
                <Icon icon={item.icon} class="h-6 w-6 flex-shrink-0" />
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1 truncate font-medium">
                    {item.name}
                    {#if group.category.includes('Recommended')}
                      <Badge
                        variant="outline"
                        class="border-primary bg-primary/5 text-xs text-primary"
                      >
                        <TrendingUp class="mr-1 h-3 w-3" />
                        Recommended
                      </Badge>
                    {/if}
                  </div>
                  <div class="text-xs text-muted-foreground">
                    {item.category}
                  </div>
                </div>
              </div>
              <Button variant="ghost" size="sm" onclick={() => onAdd(item)}>
                <Plus class="h-4 w-4" />
              </Button>
            </div>
          {/each}
        </div>
      </CardContent>
    </Card>
  {/each}
{/if}
