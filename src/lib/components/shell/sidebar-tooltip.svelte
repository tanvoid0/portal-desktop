<script lang="ts" module>
  /**
   * Shared look for sidebar hover tooltips: a soft elevated chip, offset off the
   * icon, no arrow. Spread into `tooltipContentProps` on `SidebarMenuButton`, or
   * used by this component for buttons that aren't menu buttons.
   */
  export const sidebarTooltipProps = {
    sideOffset: 10,
    class:
      'rounded-lg border border-border/60 bg-popover px-2.5 py-1.5 text-xs font-medium tracking-tight text-popover-foreground shadow-lg shadow-black/20',
    arrowClasses: 'hidden',
  } as const;
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { useSidebar } from '$lib/components/ui/sidebar/context.svelte.js';

  interface Props {
    label: string;
    side?: 'top' | 'right' | 'bottom' | 'left';
    /** Renders the hovered element; spread `props` onto it. */
    trigger: Snippet<[{ props: Record<string, unknown> }]>;
  }

  let { label, side = 'right', trigger }: Props = $props();

  const sidebar = useSidebar();
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      {@render trigger({ props })}
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content
    {side}
    align="center"
    hidden={sidebar.state !== 'collapsed' || sidebar.isMobile}
    {...sidebarTooltipProps}
  >
    {label}
  </Tooltip.Content>
</Tooltip.Root>
