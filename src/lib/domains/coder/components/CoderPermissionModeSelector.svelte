<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { ChevronDown, Shield, Zap } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import {
    getPermissionModeConfig,
    PERMISSION_MODES,
  } from "../config/permissionModes.js";
  import type { PermissionMode } from "../types.js";

  interface Props {
    mode: PermissionMode;
    onModeChange: (mode: PermissionMode) => void;
    disabled?: boolean;
    /** When true, permission has no effect (e.g. Ask agent mode). */
    inert?: boolean;
  }

  let { mode, onModeChange, disabled = false, inert = false }: Props = $props();

  const config = $derived(getPermissionModeConfig(mode));
  const isDisabled = $derived(disabled || inert);
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    disabled={isDisabled}
    title={inert ? "Not applicable in Ask mode" : config.hint}
    class={cn(
      "inline-flex h-7 max-w-[96px] items-center gap-1 rounded-full border px-2 text-[11px] font-medium transition-colors",
      mode === "auto-accept-all"
        ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
        : "border-border/80 bg-muted/40 text-muted-foreground hover:text-foreground",
      isDisabled && "pointer-events-none opacity-40",
    )}
  >
    {#if mode === "auto-accept-all"}
      <Zap class="h-3 w-3 shrink-0" />
    {:else}
      <Shield class="h-3 w-3 shrink-0" />
    {/if}
    <span class="truncate">{config.label}</span>
    <ChevronDown class="h-3 w-3 shrink-0 opacity-60" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-44 p-1">
    <DropdownMenu.RadioGroup
      value={mode}
      onValueChange={(v) => onModeChange(v as PermissionMode)}
    >
      {#each PERMISSION_MODES as m (m.value)}
        <DropdownMenu.RadioItem value={m.value} class="rounded-md py-1.5 text-xs">
          {#snippet children({ checked: _checked })}
            <span class="font-medium">{m.label}</span>
            <span class="ml-1 text-[10px] text-muted-foreground">{m.hint}</span>
          {/snippet}
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
  </DropdownMenu.Content>
</DropdownMenu.Root>
