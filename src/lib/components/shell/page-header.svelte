<script lang="ts">
  import type { Snippet } from "svelte";
  import { Badge } from "$lib/components/ui/badge";

  interface Props {
    title: string;
    description?: string;
    badge?: string;
    actions?: Snippet;
  }

  let { title, description, badge, actions }: Props = $props();
</script>

<div class="flex items-center justify-between gap-4">
  <div>
    <div class="flex items-center gap-2">
      <h1 class="text-3xl font-bold">{title}</h1>
      {#if badge}
        <Badge variant="secondary">{badge}</Badge>
      {/if}
    </div>
    {#if description}
      <p class="text-muted-foreground">{description}</p>
    {/if}
  </div>
  {#if actions}
    <div class="flex shrink-0 items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</div>
