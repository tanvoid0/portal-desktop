<script lang="ts">
  import FaPlus from '~icons/fa6-solid/plus';
  import FaXmark from '~icons/fa6-solid/xmark';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    tags: string[];
  }
  let { tags = $bindable() }: Props = $props();

  let newTag = $state('');

  function addTag() {
    if (newTag.trim() && !tags.includes(newTag.trim())) {
      tags = [...tags, newTag.trim()];
      newTag = '';
    }
  }

  function handleTagKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      addTag();
    }
  }
</script>

<div class="flex gap-2">
  <Input placeholder="Add a tag..." bind:value={newTag} onkeydown={handleTagKeydown} />
  <Button type="button" variant="outline" onclick={addTag}>
    <FaPlus class="h-4 w-4" />
  </Button>
</div>
{#if tags.length > 0}
  <div class="flex flex-wrap gap-2">
    {#each tags as tag (tag)}
      <Badge variant="secondary" class="flex items-center gap-1">
        {tag}
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onclick={() => (tags = tags.filter((t) => t !== tag))}
          class="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
        >
          <FaXmark class="h-3 w-3" />
        </Button>
      </Badge>
    {/each}
  </div>
{/if}
