<!--
	TechIcon - Renders framework, package manager, and language icons
	Supports Iconify logos, devicon CSS classes, and custom file images
-->

<script lang="ts">
  import Icon from "@iconify/svelte";

  interface Props {
    icon: string;
    iconType?: "devicon" | "file";
    size?: "xs" | "sm" | "md" | "lg" | "xl";
    class?: string;
    alt?: string;
  }

  let {
    icon,
    iconType,
    size = "md",
    class: className = "",
    alt = "",
  }: Props = $props();

  const sizeClasses: Record<string, string> = {
    xs: "h-3 w-3 text-xs",
    sm: "h-4 w-4 text-sm",
    md: "h-5 w-5 text-base",
    lg: "h-8 w-8 text-2xl",
    xl: "h-10 w-10 text-3xl",
  };

  const sizeClass = $derived(sizeClasses[size]);
  const isFileIcon = $derived(iconType === "file");
  const isIconify = $derived(icon.includes(":"));
  const isDevicon = $derived(
    iconType === "devicon" ||
      icon.startsWith("devicon-") ||
      icon.startsWith("devicon "),
  );
  const deviconClass = $derived(
    icon.startsWith("devicon-") ? `devicon ${icon}` : icon,
  );
</script>

{#if isFileIcon}
  <img src={icon} {alt} class="{sizeClass} object-contain {className}" />
{:else if isIconify}
  <Icon {icon} class="{sizeClass} {className}" />
{:else if isDevicon}
  <i class="{deviconClass} {sizeClass} {className}" aria-hidden="true"></i>
{:else}
  <Icon {icon} class="{sizeClass} {className}" />
{/if}
