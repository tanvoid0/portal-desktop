<!-- Sidebar brand button: logo tile, app/SDK title, version and debug-build badge. -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { cn } from '$lib/utils';
  import SidebarTooltip from '$lib/components/shell/sidebar-tooltip.svelte';

  interface Props {
    isSdkPage: boolean;
    isDebugBuild: boolean;
    appVersion: string | null;
  }

  let { isSdkPage, isDebugBuild, appVersion }: Props = $props();
</script>

<SidebarTooltip
  label={`${isSdkPage ? 'SDK Manager' : 'Portal Desktop'}${appVersion ? ` v${appVersion}` : ''}${isDebugBuild ? ' (debug build)' : ''}`}
>
  {#snippet trigger({ props })}
    <Button
      {...props}
      type="button"
      variant="ghost"
      class="divider-edge-b divider-edge-full divider-edge-sidebar h-auto w-full justify-start gap-3 px-4 py-4 text-left transition-colors group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:gap-0 hover:bg-sidebar-accent/50"
      onclick={() => goto('/')}
    >
      <!-- Debug builds tint the logo tile so the rail says so even collapsed. -->
      <div
        class={cn(
          'flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary',
          isDebugBuild && 'bg-destructive/15 text-destructive ring-2 ring-destructive'
        )}
      >
        {#if isSdkPage}
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
            />
          </svg>
        {:else}
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        {/if}
      </div>
      <div class="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
        <span class="flex items-center gap-1.5">
          <span class="truncate text-sm font-semibold text-sidebar-foreground">
            {isSdkPage ? 'SDK Manager' : 'Portal Desktop'}
          </span>
          {#if isDebugBuild}
            <Badge variant="destructive" class="px-1.5 py-0 text-[10px]">DEBUG</Badge>
          {/if}
        </span>
        <span class="block truncate text-xs text-sidebar-foreground/60">
          <!-- Version first: the 16rem rail truncates the tagline, not it. -->
          {appVersion ? `v${appVersion} · ` : ''}{isSdkPage
            ? 'Development Tools'
            : 'Development Environment'}
        </span>
      </div>
    </Button>
  {/snippet}
</SidebarTooltip>
