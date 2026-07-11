<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { ChevronDown } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";
  import {
    AGENT_MODES,
    autoModeSubtitle,
    getAgentModeConfig,
  } from "../config/agentModes.js";
  import type { CoderAgentMode } from "../types.js";

  interface Props {
    mode: CoderAgentMode;
    onModeChange: (mode: CoderAgentMode) => void;
    effectiveMode?: CoderAgentMode | null;
    disabled?: boolean;
  }

  let {
    mode,
    onModeChange,
    effectiveMode = null,
    disabled = false,
  }: Props = $props();

  const config = $derived(getAgentModeConfig(mode));
  const autoSubtitle = $derived(autoModeSubtitle(effectiveMode));
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    {disabled}
    class={cn(
      "inline-flex h-7 max-w-[128px] items-center gap-1 rounded-full border px-2 text-[11px] font-medium transition-colors",
      config.pillClass,
      disabled && "pointer-events-none opacity-50",
    )}
  >
    <span class={cn("h-1.5 w-1.5 shrink-0 rounded-full", config.dotClass)}></span>
    <span class="truncate">
      {config.label}{#if autoSubtitle}<span class="opacity-60"> · {autoSubtitle}</span>{/if}
    </span>
    <ChevronDown class="h-3 w-3 shrink-0 opacity-60" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-56 p-1">
    <DropdownMenu.RadioGroup
      value={mode}
      onValueChange={(v) => onModeChange(v as CoderAgentMode)}
    >
      {#each AGENT_MODES as m (m.value)}
        <DropdownMenu.RadioItem
          value={m.value}
          class={cn(
            "gap-2 rounded-md py-1.5 pl-7 pr-2 text-xs",
            m.itemClass,
          )}
        >
          {#snippet children({ checked: _checked })}
            <span class={cn("absolute left-2 h-2 w-2 rounded-full", m.dotClass)}></span>
            <span class="min-w-0 flex-1">
              <span class="font-medium">{m.label}</span>
              <span class="ml-1 text-[10px] text-muted-foreground">{m.hint}</span>
            </span>
          {/snippet}
        </DropdownMenu.RadioItem>
      {/each}
    </DropdownMenu.RadioGroup>
  </DropdownMenu.Content>
</DropdownMenu.Root>
