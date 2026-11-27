<!-- Grid of pulsing placeholder cards shown while a card list loads -->
<script lang="ts">
  import { Skeleton } from '$lib/components/ui/skeleton';
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
        <Skeleton class="h-4 rounded bg-muted" />
        <Skeleton class="h-3 w-2/3 rounded bg-muted" />
      </CardHeader>
      <CardContent>
        <div class="space-y-2">
          <Skeleton class="h-3 rounded bg-muted" />
          <Skeleton class="h-3 w-1/2 rounded bg-muted" />
        </div>
      </CardContent>
    </Card>
  {/each}
</div>
