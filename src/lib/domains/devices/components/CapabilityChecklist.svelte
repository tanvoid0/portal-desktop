<script lang="ts">
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { bulkToggleable } from '../utils/capabilityChecklist';
  import type { CapabilityInfo } from '../types';

  interface Props {
    capabilities: CapabilityInfo[];
    selected: string[];
    disabled?: boolean;
  }

  let { capabilities, selected = $bindable([]), disabled = false }: Props = $props();

  // "Select all" only ever bulk-toggles the non-sensitive caps — terminal.exec,
  // agents.exec, remote.input and the rest of SENSITIVE_CAPS stay opt-in per
  // row even when every other box gets checked at once.
  const bulkCaps = $derived(bulkToggleable(capabilities));
  const allSelected = $derived(
    bulkCaps.length > 0 && bulkCaps.every((name) => selected.includes(name))
  );

  function toggle(cap: string, checked: boolean) {
    selected = checked ? [...selected, cap] : selected.filter((c) => c !== cap);
  }

  function toggleAll() {
    selected = allSelected
      ? selected.filter((c) => !bulkCaps.includes(c))
      : [...new Set([...selected, ...bulkCaps])];
  }
</script>

{#if capabilities.length === 0}
  <p class="text-sm text-muted-foreground">No capabilities are registered.</p>
{:else}
  <div class="space-y-2">
    <Button type="button" variant="ghost" size="sm" onclick={toggleAll} {disabled}>
      {allSelected ? 'Deselect all' : 'Select all'}
    </Button>
    <!-- Columns follow this element's width, not the viewport's: it also sits in half-width cards.
         13rem holds the longest cell (switch + "agents.history" + Sensitive); flex-wrap covers longer. -->
    <div class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))] gap-2">
      {#each capabilities as cap (cap.name)}
        <Label class="flex flex-wrap items-center gap-2 text-sm font-normal">
          <Switch
            checked={selected.includes(cap.name)}
            onCheckedChange={(v) => toggle(cap.name, v === true)}
            {disabled}
          />
          {cap.name}
          {#if cap.is_sensitive}
            <Badge variant="destructive" class="px-1.5 py-0 text-[10px] font-normal"
              >Sensitive</Badge
            >
          {/if}
        </Label>
      {/each}
    </div>
  </div>
{/if}
