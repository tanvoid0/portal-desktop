<script lang="ts">
  import { Collapsible as CollapsiblePrimitive } from 'bits-ui';
  import { cn } from '$lib/utils.js';

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: CollapsiblePrimitive.ContentProps = $props();
</script>

<!--
  bits-ui only sets `hidden` once its presence layer settles, and here it never
  does: a closed section kept its full height on screen, so every collapse
  toggle in the app looked broken. `data-state` flips synchronously, so hide off
  that instead. No caller animates the collapse, so nothing loses an exit
  transition to the `display: none`.
-->
<CollapsiblePrimitive.Content
  bind:ref
  data-slot="collapsible-content"
  class={cn('data-[state=closed]:hidden', className)}
  {...restProps}
/>
