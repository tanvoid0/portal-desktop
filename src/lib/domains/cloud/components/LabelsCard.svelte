<!-- Card listing key=value pairs as outline badges (labels, selectors, annotations) -->
<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    title?: string;
    items: Record<string, unknown>;
    emptyText?: string;
  }
  let { title = 'Labels', items, emptyText = 'No labels configured' }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle>{title}</CardTitle>
  </CardHeader>
  <CardContent>
    {#if Object.keys(items).length > 0}
      <div class="flex flex-wrap gap-2">
        {#each Object.entries(items) as [key, value] (key)}
          <Badge variant="outline">{key}={value}</Badge>
        {/each}
      </div>
    {:else}
      <p class="text-muted-foreground">{emptyText}</p>
    {/if}
  </CardContent>
</Card>
