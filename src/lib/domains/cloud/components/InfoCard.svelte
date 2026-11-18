<!-- Card of label/value rows with an optional leading status badge -->
<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';

  interface Field {
    label: string;
    value: string | number | null | undefined;
  }
  interface Props {
    title: string;
    fields: Field[];
    status?: { text: string; variant?: 'default' | 'secondary' };
    /** Class for each value line, e.g. "text-lg font-semibold" or "font-mono font-medium". */
    valueClass?: string;
  }
  let { title, fields, status, valueClass = 'font-medium' }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle>{title}</CardTitle>
  </CardHeader>
  <CardContent class="space-y-3">
    {#if status}
      <div>
        <p class="text-sm text-muted-foreground">Status</p>
        <Badge class="mt-1" variant={status.variant ?? 'default'}>{status.text}</Badge>
      </div>
    {/if}
    {#each fields as f (f.label)}
      <div>
        <p class="text-sm text-muted-foreground">{f.label}</p>
        <p class={valueClass}>{f.value}</p>
      </div>
    {/each}
  </CardContent>
</Card>
