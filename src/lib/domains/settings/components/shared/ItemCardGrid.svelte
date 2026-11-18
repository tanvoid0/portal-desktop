<!-- Grid of the user's items with edit/delete, or an empty state. -->
<script lang="ts" generics="T extends BaseItem">
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import Plus from '~icons/fa6-solid/plus';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import Icon from '@iconify/svelte';
  import type { Component } from 'svelte';
  import type { BaseItem } from './types';

  interface Props {
    items: T[];
    isSearching: boolean;
    itemName: string;
    emptyMessage: string;
    emptySearchMessage: string;
    emptyIcon: Component<{ class?: string }>;
    onAdd: () => void;
    onEdit: (item: T) => void;
    onDelete: (id: number, name: string) => void;
  }

  let {
    items,
    isSearching,
    itemName,
    emptyMessage,
    emptySearchMessage,
    emptyIcon: EmptyIcon,
    onAdd,
    onEdit,
    onDelete,
  }: Props = $props();
</script>

{#if items.length === 0}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-12">
      <EmptyIcon class="mb-4 h-12 w-12 text-muted-foreground" />
      <p class="mb-2 text-muted-foreground">
        {#if isSearching}
          {emptySearchMessage}
        {:else}
          {emptyMessage}
        {/if}
      </p>
      <Button variant="outline" onclick={() => onAdd()}>
        <Plus class="mr-2 h-4 w-4" />
        Add Your First {itemName}
      </Button>
    </CardContent>
  </Card>
{:else}
  <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
    {#each items as item}
      <Card class="relative">
        <CardHeader>
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-3">
              {#if item.icon_type === 'devicon'}
                <Icon icon={item.icon} class="h-8 w-8" />
              {:else}
                <img src={item.icon} alt={item.name} class="h-8 w-8" />
              {/if}
              <div>
                <CardTitle class="text-lg">{item.name}</CardTitle>
                <CardDescription>{item.category}</CardDescription>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div class="mt-2 flex items-center justify-end gap-2">
            <Button variant="outline" size="sm" onclick={() => onEdit(item)}>
              <Edit class="mr-1 h-3 w-3" />
              Edit
            </Button>
            <Button
              variant="outline"
              size="sm"
              onclick={() => onDelete(item.id, item.name)}
              class="text-red-500 hover:text-red-700"
            >
              <Trash2 class="mr-1 h-3 w-3" />
              Delete
            </Button>
          </div>
        </CardContent>
      </Card>
    {/each}
  </div>
{/if}
