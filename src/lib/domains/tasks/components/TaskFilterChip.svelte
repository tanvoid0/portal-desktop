<script lang="ts">
  import type { Snippet } from 'svelte';
  import FaXmark from '~icons/fa6-solid/xmark';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    removeLabel: string;
    onRemove: () => void;
    children: Snippet;
  }
  let { removeLabel, onRemove, children }: Props = $props();
</script>

<Badge variant="outline" class="text-xs">
  {@render children()}
  <Button
    onclick={onRemove}
    variant="ghost"
    size="sm"
    class="ml-1 h-4 w-4 p-0 hover:bg-muted"
    aria-label={removeLabel}
  >
    <FaXmark class="h-3 w-3" />
  </Button>
</Badge>
