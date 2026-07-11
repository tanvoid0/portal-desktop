<script lang="ts">
  import Icon from "@iconify/svelte";
  import {
    resolveShellIcon,
    resolveTabIcon,
    normalizeTerminalIcon,
    isIconifyIcon,
  } from "../utils/shellIcons";
  import { getShellIconSvg } from "../utils/shellIconSvgs";

  interface Props {
    icon?: string;
    shell?: string;
    tab?: {
      icon?: string;
      shell?: string;
      type?: string;
      resourceName?: string;
    };
    size?: "xs" | "sm" | "md";
    class?: string;
  }

  let {
    icon,
    shell,
    tab,
    size = "sm",
    class: className = "",
  }: Props = $props();

  const sizeClasses = {
    xs: "h-3.5 w-3.5",
    sm: "h-4 w-4",
    md: "h-5 w-5",
  };

  const resolvedIcon = $derived.by(() => {
    if (tab) return resolveTabIcon(tab);

    const normalized = normalizeTerminalIcon(icon);
    if (normalized && isIconifyIcon(normalized)) return normalized;

    return resolveShellIcon(shell ?? icon);
  });

  const svgData = $derived(getShellIconSvg(resolvedIcon));
</script>

{#if svgData}
  <svg
    viewBox={svgData.viewBox}
    class="{sizeClasses[size]} shrink-0 {className}"
    class:text-foreground={!svgData.preserveColors}
    aria-hidden="true"
    role="img"
  >
    {@html svgData.body}
  </svg>
{:else}
  <Icon icon={resolvedIcon} class="{sizeClasses[size]} shrink-0 {className}" />
{/if}
