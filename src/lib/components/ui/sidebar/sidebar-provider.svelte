<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { cn, type WithElementRef } from "$lib/utils.js";
  import type { HTMLAttributes } from "svelte/elements";
  import {
    SIDEBAR_COOKIE_MAX_AGE,
    SIDEBAR_COOKIE_NAME,
  } from "./constants.js";
  import { setSidebar } from "./context.svelte.js";

  let {
    ref = $bindable(null),
    open = $bindable(true),
    onOpenChange = () => {},
    cookieName = SIDEBAR_COOKIE_NAME,
    enableShortcut = true,
    class: className,
    style,
    children,
    ...restProps
  }: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    cookieName?: string;
    enableShortcut?: boolean;
  } = $props();

  const sidebar = setSidebar({
    open: () => open,
    setOpen: (value: boolean) => {
      open = value;
      onOpenChange(value);

      document.cookie = `${cookieName}=${open}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
    },
  });
</script>

<svelte:window
  onkeydown={(e) => enableShortcut && sidebar.handleShortcutKeydown(e)}
/>

<Tooltip.Provider delayDuration={0}>
  <div
    data-slot="sidebar-wrapper"
    style={style}
    class={cn(
      "group/sidebar-wrapper has-data-[variant=inset]:bg-sidebar flex min-h-svh w-full",
      className,
    )}
    bind:this={ref}
    {...restProps}
  >
    {@render children?.()}
  </div>
</Tooltip.Provider>
