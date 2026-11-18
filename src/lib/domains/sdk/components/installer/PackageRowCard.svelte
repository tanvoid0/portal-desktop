<!-- One installed / updatable package: name, source, a subtitle line and an actions slot. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';

  let {
    name,
    source,
    subtitle,
    actions,
  }: { name: string; source: string; subtitle: string; actions: Snippet } = $props();
</script>

<Card>
  <CardContent class="pt-6">
    <div class="flex items-center justify-between">
      <div class="flex-1">
        <div class="flex items-center gap-2">
          <h3 class="font-semibold">{name}</h3>
          <Badge variant="secondary" class="text-xs">{source}</Badge>
        </div>
        <p class="text-sm text-muted-foreground">{subtitle}</p>
      </div>
      <div class="flex items-center gap-2">{@render actions()}</div>
    </div>
  </CardContent>
</Card>
