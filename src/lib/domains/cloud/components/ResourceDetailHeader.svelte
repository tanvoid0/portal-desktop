<!-- Detail-page title block with Refresh and Back actions -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import RefreshCw from '~icons/fa6-solid/rotate';

  interface Props {
    kind: string;
    name: string;
    namespace: string;
    backHref: string;
    backLabel: string;
    onRefresh: () => void;
    /** Extra buttons between Refresh and Back. */
    children?: Snippet;
  }
  let { kind, name, namespace, backHref, backLabel, onRefresh, children }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <div>
    <h1 class="text-2xl font-bold">{kind}: {name}</h1>
    <p class="text-muted-foreground">Namespace: {namespace}</p>
  </div>
  <div class="flex items-center gap-2">
    <Button variant="outline" size="sm" onclick={onRefresh}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
    {@render children?.()}
    <Button variant="outline" size="sm" onclick={() => goto(backHref)}>
      <ArrowLeft class="mr-2 h-4 w-4" />
      {backLabel}
    </Button>
  </div>
</div>
