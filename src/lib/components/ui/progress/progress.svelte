<script lang="ts">
  import { Progress as ProgressPrimitive } from 'bits-ui';
  import { cn, type WithoutChildrenOrChild } from '$lib/utils.js';

  let {
    ref = $bindable(null),
    class: className,
    max = 100,
    value,
    indicatorClass,
    ...restProps
  }: WithoutChildrenOrChild<ProgressPrimitive.RootProps> & { indicatorClass?: string } = $props();

  // Local addition: re-apply after `shadcn update`.
  const percent = $derived(Math.min(100, Math.max(0, (100 * (value ?? 0)) / (max || 1))));
</script>

<ProgressPrimitive.Root
  bind:ref
  data-slot="progress"
  class={cn('relative h-2 w-full overflow-hidden rounded-full bg-primary/20', className)}
  {value}
  {max}
  {...restProps}
>
  <div
    data-slot="progress-indicator"
    class={cn('h-full w-full flex-1 rounded-full bg-primary transition-all', indicatorClass)}
    style="transform: translateX(-{100 - percent}%)"
  ></div>
</ProgressPrimitive.Root>
