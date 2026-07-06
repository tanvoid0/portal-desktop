<script lang="ts">
  import Devicon from "$lib/components/ui/devicon.svelte";
  import { MenuButton as SidebarMenuButton } from "$lib/components/ui/sidebar";
  import SDKInstallStatusIcon from "./SDKInstallStatusIcon.svelte";
  import SDKServiceToggle from "./SDKServiceToggle.svelte";
  import type { SDKItem, SDKSidebarVariant } from "./sdkSidebarTypes";

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSDKClick();
    }
  }
</script>

{#if variant === "collapsed"}
  <SidebarMenuButton
    size="sm"
    isActive={selected}
    tooltipContent={
      sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName
    }
    onclick={onSDKClick}
    class="!w-auto !p-1 !gap-1"
  >
    <Devicon icon={sdk.icon} size="sm" class={iconColor} />
    <SDKInstallStatusIcon installed={sdk.installed} />
    {#if sdk.hasService}
      <SDKServiceToggle
        variant="collapsed"
        checked={sdk.serviceRunning === true}
        disabled={serviceToggleDisabled}
        onToggle={onServiceToggle}
      />
    {/if}
  </SidebarMenuButton>
{:else}
  <div
    class="flex cursor-pointer items-center gap-2 rounded-md p-1.5 transition-colors hover:bg-muted/50 {selected
      ? 'bg-muted/50'
      : ''}"
    role="button"
    tabindex="0"
    onclick={onSDKClick}
    onkeydown={handleKeydown}
  >
    <Devicon icon={sdk.icon} size="sm" class={iconColor} />
    <div class="min-w-0 flex-1">
      <div class="truncate text-xs font-medium">{sdk.displayName}</div>

      {#if sdk.version}
        <div class="text-[11px] text-muted-foreground leading-3">
          {sdk.version}
        </div>
      {:else if !sdk.installed}
        <div class="text-[11px] text-muted-foreground leading-3">
          Not installed
        </div>
      {/if}

      {#if sdk.category === "database" && sdk.port}
        <div class="text-[11px] text-muted-foreground leading-3">
          Port: {sdk.port}
        </div>
      {/if}
    </div>

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

