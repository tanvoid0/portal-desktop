<!-- Searchable block library; clicking a block queues it. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import Plus from '~icons/fa6-solid/plus';
  import { PageLoading } from '$lib/components/shell';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { Block } from '$lib/domains/projects/pipelines';

  interface Props {
    blocks: Block[];
    loading: boolean;
    queuedBlockIds: Set<string>;
    onAdd: (block: Block) => void;
  }

  let { blocks, loading, queuedBlockIds, onAdd }: Props = $props();

  let search = $state('');

  const filtered = $derived.by(() => {
    if (!search.trim()) return blocks;
    const q = search.toLowerCase();
    return blocks.filter(
      (b) =>
        b.name.toLowerCase().includes(q) ||
        b.description.toLowerCase().includes(q) ||
        b.tags.some((t) => t.toLowerCase().includes(q))
    );
  });
</script>

<SectionCard title="Block Library" contentClass="space-y-3">
  <Input bind:value={search} placeholder="Search blocks…" class="w-full" />
  {#if loading}
    <PageLoading message="Loading blocks…" />
  {:else}
    <div class="max-h-64 space-y-1 overflow-y-auto">
      {#each filtered as block (block.id)}
        <Button
          type="button"
          variant="outline"
          class="h-auto w-full justify-between rounded-md px-3 py-2 text-left transition-colors hover:bg-accent disabled:opacity-50"
          disabled={queuedBlockIds.has(block.id)}
          onclick={() => onAdd(block)}
        >
          <div class="min-w-0">
            <p class="truncate text-sm font-medium">{block.name}</p>
            <p class="truncate text-xs text-muted-foreground">
              {block.description}
            </p>
          </div>
          <div class="ml-2 flex shrink-0 items-center gap-2">
            <Badge variant="outline" class="text-xs">
              {block.category}
            </Badge>
            {#if !queuedBlockIds.has(block.id)}
              <Plus class="h-4 w-4 text-muted-foreground" />
            {/if}
          </div>
        </Button>
      {/each}
    </div>
  {/if}
</SectionCard>
