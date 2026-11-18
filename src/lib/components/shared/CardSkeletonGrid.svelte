<!-- Grid of pulsing placeholder cards shown while a card list loads -->
<script lang="ts">
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import { cn } from '$lib/utils';

  interface Props {
    count: number;
    class?: string;
  }
  let { count, class: className = 'md:grid-cols-2' }: Props = $props();
</script>

<div class={cn('grid gap-4', className)}>
  {#each Array(count) as _}
    <Card>
      <CardHeader>
        <div class="h-4 animate-pulse rounded bg-muted"></div>
        <div class="h-3 w-2/3 animate-pulse rounded bg-muted"></div>
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <div class="h-3 animate-pulse rounded bg-muted"></div>
          <div class="h-3 w-1/2 animate-pulse rounded bg-muted"></div>
        </div>
      </CardContent>
    </Card>
  {/each}
</div>
