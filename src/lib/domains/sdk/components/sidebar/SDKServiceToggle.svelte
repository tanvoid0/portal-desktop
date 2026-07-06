<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Play, Square } from "@lucide/svelte";
  import { Switch } from "$lib/components/ui/switch";
  import type { SDKSidebarVariant } from "./sdkSidebarTypes";

  interface Props {
    variant: SDKSidebarVariant;
    checked: boolean;
    disabled: boolean;
    onToggle: (next: boolean) => void;
  }

  const { variant, checked, disabled, onToggle }: Props = $props();

  function toggleCollapsed(e: MouseEvent) {
    // Keep the toggle from activating the parent row.
    e.stopPropagation();
    e.preventDefault();
    if (!disabled) onToggle(!checked);
  }
</script>

{#if variant === "collapsed"}
  <Button
    type="button"
    variant="ghost"
    size="icon"
    class="h-4 w-4"
    title={checked ? "Stop service" : "Start service"}
    disabled={disabled}
    onclick={toggleCollapsed}
  >
    {#if checked}
      <Square class="h-3 w-3 text-blue-500" />
    {:else}
      <Play class="h-3 w-3 text-blue-500" />
    {/if}
  </Button>
{:else}
  <Switch
    checked={checked}
    disabled={disabled}
    title={checked ? "Stop service" : "Start service"}
    onclick={(e) => e.stopPropagation()}
    onCheckedChange={(next) => onToggle(next)}
  />
{/if}

