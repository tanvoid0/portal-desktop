<!-- Black/green console surface for run output. Caller supplies the content; `el` is bindable for auto-scroll. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  interface Props {
    el?: HTMLDivElement | null;
    class?: string;
    children?: Snippet;
  }

  let { el = $bindable(null), class: className, children }: Props = $props();
</script>

<div
  bind:this={el}
  class={cn('overflow-auto bg-black p-4 font-mono text-sm text-green-400', className)}
>
  {@render children?.()}
</div>
