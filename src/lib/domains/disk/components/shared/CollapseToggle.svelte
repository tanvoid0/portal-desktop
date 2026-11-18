<script lang="ts">
  import { Button } from '$lib/components/ui/button';

  let {
    collapsed,
    onclick,
    class: className = '',
  }: { collapsed: boolean; onclick: () => void; class?: string } = $props();
</script>

<Button
  type="button"
  variant="ghost"
  size="icon-sm"
  class="h-4 w-4 shrink-0 p-0 text-muted-foreground {className}"
  {onclick}>{collapsed ? '▸' : '▾'}</Button
>
