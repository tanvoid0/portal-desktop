<!--
	Block library grid — filtering, the card, and the list-level states.
-->
<script lang="ts">
  import type { Component } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Download from '~icons/fa6-solid/download';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import Package from '~icons/fa6-solid/box-open';
  import Code from '~icons/fa6-solid/code';
  import Rocket from '~icons/fa6-solid/rocket';
  import Wrench from '~icons/fa6-solid/wrench';
  import Settings from '~icons/fa6-solid/gear';
  import type { Block } from '$lib/domains/projects/pipelines';
  import { PageLoading, PageError, PageEmpty } from '$lib/components/shell';

  interface Props {
    blocks: Block[];
    search: string;
    category: Block['category'] | null;
    loading?: boolean;
    error?: string | null;
    onRetry?: () => void;
    onCreate?: () => void;
    onClearFilters?: () => void;
    onEdit?: (block: Block) => void;
    onExport?: (block: Block) => void;
    onDelete?: (blockId: string) => void;
  }

  let {
    blocks,
    search,
    category,
    loading = false,
    error = null,
    onRetry = () => {},
    onCreate = () => {},
    onClearFilters = () => {},
    onEdit = () => {},
    onExport = () => {},
    onDelete = () => {},
  }: Props = $props();

  const filteredBlocks = $derived.by(() => {
    let filtered: Block[] = blocks;

    if (search) {
      const query = search.toLowerCase();
      filtered = filtered.filter(
        (block: Block) =>
          block.name.toLowerCase().includes(query) ||
          block.description.toLowerCase().includes(query) ||
          (block.tags || []).some((tag: string) => tag.toLowerCase().includes(query))
      );
    }

    if (category) {
      filtered = filtered.filter((block: Block) => block.category === category);
    }

    return filtered;
  });

  const filterSummary = $derived.by(() => {
    const bits: string[] = [];
    if (search.trim()) bits.push(`“${search.trim()}”`);
    if (category) bits.push(`category “${category}”`);
    return bits.join(' and ');
  });

  const categoryIcons: Record<Block['category'], Component> = {
    build: Package,
    test: Wrench,
    deploy: Rocket,
    utility: Settings,
    custom: Code,
  };

  const categoryColors: Record<Block['category'], string> = {
    build: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    test: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    deploy: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
    utility: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
    custom: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  };
</script>

{#if loading}
  <PageLoading message="Loading blocks..." />
{:else if error}
  <PageError title="Failed to load blocks" message={error} onRetry={() => onRetry()} />
{:else if blocks.length === 0}
  <PageEmpty
    title="No blocks yet"
    description="Create your first block to get started."
    icon={Package}
    actionLabel="Create Your First Block"
    onAction={() => onCreate()}
  />
{:else if filteredBlocks.length === 0}
  <!--
    The library is not empty — the search box (shared with the Templates tab) or
    the category filter hid everything. Say which, and offer the way out.
  -->
  <PageEmpty
    title="No matching blocks"
    description={`No block matches ${filterSummary}. The library holds ${blocks.length} block${
      blocks.length === 1 ? '' : 's'
    }.`}
    icon={Package}
    actionLabel="Clear filters"
    onAction={() => onClearFilters()}
  />
{:else}
  <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
    {#each filteredBlocks as block (block.id)}
      <!-- ?? Code / custom: the category is a backend string, not necessarily in the map. -->
      {@const CategoryIcon = categoryIcons[block.category] ?? Code}
      <Card class="gap-0 p-3">
        <div class="flex items-start gap-2">
          <div class="min-w-0 flex-1">
            <div class="flex min-w-0 items-center gap-2">
              <h3 class="truncate text-sm font-medium">{block.name}</h3>
              <Badge
                class="{categoryColors[block.category] ?? categoryColors.custom} shrink-0 text-xs"
              >
                <CategoryIcon class="mr-1 h-3 w-3" />
                {block.category}
              </Badge>
            </div>
            <p class="line-clamp-1 text-xs text-muted-foreground">{block.description}</p>
          </div>
          <div class="flex shrink-0 items-center gap-0.5">
            <Button
              size="icon-sm"
              variant="ghost"
              aria-label={`Edit block ${block.name}`}
              onclick={() => onEdit(block)}
            >
              <Edit class="h-3.5 w-3.5" />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              aria-label={`Export block ${block.name}`}
              onclick={() => onExport(block)}
            >
              <Download class="h-3.5 w-3.5" />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              class="text-destructive hover:text-destructive"
              aria-label={`Delete block ${block.name}`}
              onclick={() => onDelete(block.id)}
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        {#if block.parameters.length > 0 || (block.tags && block.tags.length > 0)}
          <div class="mt-2 flex flex-wrap items-center gap-1">
            {#if block.parameters.length > 0}
              <span class="text-xs text-muted-foreground">
                {block.parameters.length} param{block.parameters.length === 1 ? '' : 's'}
              </span>
            {/if}
            {#each (block.tags || []).slice(0, 3) as tag}
              <Badge variant="outline" class="text-xs">{tag}</Badge>
            {/each}
            {#if (block.tags || []).length > 3}
              <Badge variant="outline" class="text-xs">+{(block.tags || []).length - 3}</Badge>
            {/if}
          </div>
        {/if}
      </Card>
    {/each}
  </div>
{/if}
