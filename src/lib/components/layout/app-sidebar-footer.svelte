<!-- Sidebar footer: theme toggle plus Settings (desktop) or Disconnect (mobile shell). -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import ThemeToggle from '$lib/components/ui/theme-toggle.svelte';
  import SidebarTooltip from '$lib/components/shell/sidebar-tooltip.svelte';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';
  import HostStats from './host-stats.svelte';
  import Gear from '~icons/fa6-solid/gear';
  import Unplug from '~icons/fa6-solid/link-slash';
</script>

<!-- Sidebar Footer — stacks into a single icon column when collapsed,
     otherwise the row overflows the 3rem rail and clips the buttons. -->
<div
  class="divider-edge-t divider-edge-full divider-edge-sidebar flex-shrink-0 p-4 group-data-[collapsible=icon]:p-2"
>
  {#if !isMobileShell}
    <HostStats />
  {/if}
  <div
    class="flex items-center justify-between gap-2 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1"
  >
    <ThemeToggle />
    <div
      class="flex items-center gap-1 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-1"
    >
      {#if isMobileShell}
        <Button
          variant="ghost"
          size="sm"
          class="h-8 gap-2 px-2"
          onclick={() => mobileConnection.disconnect()}
          aria-label="Disconnect"
        >
          <Unplug class="h-4 w-4" />
          <span class="max-w-32 truncate text-xs">
            Disconnect {mobileConnection.status?.name ?? ''}
          </span>
        </Button>
      {:else}
        <SidebarTooltip label="Settings">
          {#snippet trigger({ props })}
            <Button
              {...props}
              variant="ghost"
              size="sm"
              class="h-8 w-8 px-0"
              onclick={() => goto('/settings')}
              aria-label="Settings"
            >
              <Gear class="h-4 w-4" />
            </Button>
          {/snippet}
        </SidebarTooltip>
      {/if}
    </div>
  </div>
</div>
