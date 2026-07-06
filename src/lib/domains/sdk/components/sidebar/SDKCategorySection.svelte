<script lang="ts">
  import { ChevronDown } from "@lucide/svelte";
  import { MenuButton as SidebarMenuButton } from "$lib/components/ui/sidebar";
  import { Button } from "$lib/components/ui/button";
  import SDKSidebarItem from "./SDKSidebarItem.svelte";
  import type { SDKItem, SDKSidebarVariant } from "./sdkSidebarTypes";

  interface Props {
    variant: SDKSidebarVariant;
    title: string;
    iconComponent: any;
    items: SDKItem[];
    selectedItemId: string | null;
    collapsible?: boolean;
    isOpen?: boolean;
    onToggle?: () => void;
    getSDKIconColor: (sdkId: string) => string;
    isServiceToggleDisabled: (sdk: SDKItem) => boolean;
    onSDKClick: (sdk: SDKItem) => void;
    onServiceToggle: (sdk: SDKItem, next: boolean) => void;
  }

  const {
    variant,
    title,
    iconComponent,
    items,
    selectedItemId,
    collapsible = false,
    isOpen = true,
    onToggle,
    getSDKIconColor,
    isServiceToggleDisabled,
    onSDKClick,
    onServiceToggle,
  }: Props = $props();

  function toggle() {
    if (!collapsible) return;
    onToggle?.();
  }
</script>

{#if variant === "collapsed"}
  <div class="space-y-1">
    <SidebarMenuButton
      size="sm"
      tooltipContent={title}
      aria-expanded={isOpen}
      onclick={toggle}
      class="!w-auto !p-1 !gap-1"
    >
      {@const Icon = iconComponent}
      <Icon class="h-4 w-4" />
      {#if collapsible}
        <ChevronDown
          class="h-3 w-3 shrink-0 transition-transform {isOpen ? 'rotate-0' : '-rotate-90'}"
        />
      {/if}
    </SidebarMenuButton>

    {#if !collapsible || isOpen}
      <div class="grid grid-cols-2 gap-1 pl-1">
        {#each items as sdk}
          <SDKSidebarItem
            variant="collapsed"
            sdk={sdk}
            selected={selectedItemId === sdk.id}
            iconColor={getSDKIconColor(sdk.id)}
            serviceToggleDisabled={isServiceToggleDisabled(sdk)}
            onSDKClick={() => onSDKClick(sdk)}
            onServiceToggle={(next) => onServiceToggle(sdk, next)}
          />
        {/each}
      </div>
    {/if}
  </div>
{:else}
  {@const Icon = iconComponent}
  <div class="space-y-1">
    <Button
      type="button"
      variant="ghost"
      class="flex h-auto w-full items-center gap-2 rounded-md px-1 py-0.5 text-left"
      aria-expanded={isOpen}
      onclick={toggle}
    >
      <Icon class="h-4 w-4" />
      <h3 class="text-xs font-medium leading-none">{title}</h3>
      {#if collapsible}
        <ChevronDown
          class="ml-auto h-3 w-3 transition-transform {isOpen ? 'rotate-0' : '-rotate-90'}"
        />
      {/if}
    </Button>

    {#if !collapsible || isOpen}
      <div class="space-y-0.5">
        {#each items as sdk}
          <SDKSidebarItem
            variant="expanded"
            sdk={sdk}
            selected={selectedItemId === sdk.id}
            iconColor={getSDKIconColor(sdk.id)}
            serviceToggleDisabled={isServiceToggleDisabled(sdk)}
            onSDKClick={() => onSDKClick(sdk)}
            onServiceToggle={(next) => onServiceToggle(sdk, next)}
          />
        {/each}
      </div>
    {/if}
  </div>
{/if}

