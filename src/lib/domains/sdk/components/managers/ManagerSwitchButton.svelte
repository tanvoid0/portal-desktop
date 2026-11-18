<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Play from '~icons/fa6-solid/play';

  let { switching, onclick }: { switching: boolean; onclick: () => void } = $props();
</script>

<Button variant="outline" size="sm" {onclick} disabled={switching}>
  {#if switching}
    <RefreshCw class="mr-1 h-4 w-4 animate-spin" />
  {:else}
    <Play class="mr-1 h-4 w-4" />
  {/if}
  Switch
</Button>
