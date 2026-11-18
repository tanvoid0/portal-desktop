<!-- Tag editor: add by button/Enter, remove via the badge's X. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Badge } from '$lib/components/ui/badge';
  import Plus from '~icons/fa6-solid/plus';
  import X from '~icons/fa6-solid/xmark';

  interface Props {
    tags: string[];
    disabled?: boolean;
  }

  let { tags = $bindable(), disabled: loading = false }: Props = $props();

  let newTag = $state('');

  function handleAddTag() {
    if (newTag.trim() && !tags.includes(newTag.trim())) {
      tags = [...tags, newTag.trim()];
      newTag = '';
    }
  }

  function handleRemoveTag(tagToRemove: string) {
    tags = tags.filter((tag) => tag !== tagToRemove);
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      handleAddTag();
    }
  }
</script>

<div class="space-y-2">
  <Label>Tags</Label>
  <div class="flex gap-2">
    <Input
      placeholder="Add tag"
      bind:value={newTag}
      onkeypress={handleKeyPress}
      disabled={loading}
    />
    <Button
      type="button"
      variant="outline"
      onclick={handleAddTag}
      disabled={loading || !newTag.trim()}
    >
      <Plus class="h-4 w-4" />
    </Button>
  </div>
  {#if tags.length > 0}
    <div class="flex flex-wrap gap-1">
      {#each tags as tag (tag)}
        <Badge variant="secondary" class="flex items-center gap-1">
          {tag}
          <X class="h-3 w-3 cursor-pointer" onclick={() => handleRemoveTag(tag)} />
        </Badge>
      {/each}
    </div>
  {/if}
</div>
