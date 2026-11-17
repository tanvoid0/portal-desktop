<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Devicon from '$lib/components/ui/devicon.svelte';
  import { MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';
  import SDKInstallStatusIcon from './SDKInstallStatusIcon.svelte';
  import SDKServiceToggle from './SDKServiceToggle.svelte';
  import type { SDKItem, SDKSidebarVariant } from './sdkSidebarTypes';

  interface Props {
    variant: SDKSidebarVariant;
    sdk: SDKItem;
    selected: boolean;
    iconColor: string;
    serviceToggleDisabled: boolean;
    onSDKClick: () => void;
    onServiceToggle: (next: boolean) => void;
  }

  const {
    variant,
    sdk,
    selected,
    iconColor,
    serviceToggleDisabled,
    onSDKClick,
    onServiceToggle,
  }: Props = $props();
</script>

<!--
  The service toggle is a real control, so it is never a child of the control that selects the
  SDK: a nested button is invalid ARIA, the outer name swallows the toggle's own, and the
  toggle is not separately reachable (same class as QA-083).
  - expanded: the toggle is a sibling of the select Button.
  - collapsed: the 48px rail has no room for it. The old nested one sat inside the menu
    button's overflow-hidden box, clipped and unclickable but still a tab stop, so it is not
    rendered; a sibling would peek out of the rail as a clickable sliver.
-->
{#if variant === 'collapsed'}
  <SidebarMenuButton
    size="sm"
    isActive={selected}
    aria-label={sdk.displayName}
    aria-current={selected ? 'page' : undefined}
    tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
    onclick={onSDKClick}
    class="!w-auto !gap-1 !p-1"
  >
    <Devicon icon={sdk.icon} size="sm" class={iconColor} />
    <SDKInstallStatusIcon installed={sdk.installed} />
  </SidebarMenuButton>
{:else}
  <div
    class="flex items-center gap-2 rounded-md pr-1.5 transition-colors hover:bg-muted/50 {selected
      ? 'bg-muted/50'
      : ''}"
  >
    <!-- The row's left/top/bottom padding lives on the Button so the whole row stays clickable. -->
    <Button
      variant="ghost"
      aria-current={selected ? 'page' : undefined}
      class="h-auto min-w-0 flex-1 cursor-pointer justify-start p-1.5 pr-0 text-left font-normal tracking-normal whitespace-normal hover:bg-transparent dark:hover:bg-transparent"
      onclick={onSDKClick}
    >
      <Devicon icon={sdk.icon} size="sm" class={iconColor} />
      <div class="min-w-0 flex-1">
        <div class="truncate text-xs font-medium">{sdk.displayName}</div>

        {#if sdk.version}
          <div class="text-[11px] leading-3 text-muted-foreground">
            {sdk.version}
          </div>
        {:else if !sdk.installed}
          <div class="text-[11px] leading-3 text-muted-foreground">Not installed</div>
        {/if}

        {#if sdk.category === 'database' && sdk.port}
          <div class="text-[11px] leading-3 text-muted-foreground">
            Port: {sdk.port}
          </div>
        {/if}
      </div>
    </Button>

    <div class="flex flex-shrink-0 items-center gap-1">
      <SDKInstallStatusIcon installed={sdk.installed} />
      {#if sdk.hasService}
        <SDKServiceToggle
          variant="expanded"
          checked={sdk.serviceRunning === true}
          disabled={serviceToggleDisabled}
          onToggle={onServiceToggle}
        />
      {/if}
    </div>
  </div>
{/if}
