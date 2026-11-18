<script lang="ts">
  import FaSquareCheck from '~icons/fa6-solid/square-check';
  import FaCheckDouble from '~icons/fa6-solid/check-double';
  import FaXmark from '~icons/fa6-solid/xmark';
  import FaTrash from '~icons/fa6-solid/trash';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    count: number;
    onSelectAll: () => void;
    onClear: () => void;
    onDelete: () => void;
    onCancel: () => void;
  }
  let { count, onSelectAll, onClear, onDelete, onCancel }: Props = $props();
</script>

<div
  class="flex flex-wrap items-center gap-3 rounded-lg border border-warning-200 bg-warning-50 px-4 py-2 dark:border-warning-800 dark:bg-warning-900/20"
>
  <Badge
    variant="secondary"
    class="bg-warning-100 text-warning-800 dark:bg-warning-800 dark:text-warning-100"
  >
    <FaSquareCheck class="mr-1 h-3 w-3" />
    {count} selected
  </Badge>
  <div class="flex items-center gap-2">
    <Button onclick={onSelectAll} variant="outline" size="sm" class="h-7 px-2 text-xs">
      <FaCheckDouble class="mr-1 h-3 w-3" />
      Select All
    </Button>
    <Button onclick={onClear} variant="outline" size="sm" class="h-7 px-2 text-xs">
      <FaXmark class="mr-1 h-3 w-3" />
      Clear
    </Button>
    <Button
      onclick={onDelete}
      disabled={count === 0}
      variant="destructive"
      size="sm"
      class="h-7 px-2 text-xs"
    >
      <FaTrash class="mr-1 h-3 w-3" />
      Delete ({count})
    </Button>
    <Button onclick={onCancel} variant="ghost" size="sm" class="h-7 px-2 text-xs">
      <FaXmark class="mr-1 h-3 w-3" />
      Cancel
    </Button>
  </div>
</div>
