<script lang="ts">
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { confirmAction } from '$lib/utils/confirm';
  import { Badge } from '$lib/components/ui/badge';
  import { bulkToggleable } from '../utils/capabilityChecklist';
  import type { CapabilityInfo } from '../types';

  interface Props {
    capabilities: CapabilityInfo[];
    selected: string[];
    disabled?: boolean;
  }

  let { capabilities, selected = $bindable([]), disabled = false }: Props = $props();

  const bulkCaps = $derived(bulkToggleable(capabilities));
  const sensitiveCaps = $derived(capabilities.filter((c) => c.is_sensitive).map((c) => c.name));
  const allSelected = $derived(
    capabilities.length > 0 && capabilities.every((c) => selected.includes(c.name))
  );

  function toggle(cap: string, checked: boolean) {
    selected = checked ? [...selected, cap] : selected.filter((c) => c !== cap);
  }

  // Turning "all" on asks about sensitive caps (terminal.exec, agents.exec, ...) —
  // yes grants them too, no grants only the non-sensitive ones. Turning it off clears everything.
  async function toggleAll(on: boolean) {
    if (!on) {
      selected = [];
      return;
    }
    const withSensitive =
      sensitiveCaps.length > 0 &&
      (await confirmAction(
        'Also enable the sensitive permissions (' +
          sensitiveCaps.join(', ') +
          ')? These allow running commands on this computer.',
        'Enable sensitive permissions?',
        { confirmLabel: 'Yes', cancelLabel: 'No', destructive: false }
      ));
    selected = [...new Set([...bulkCaps, ...(withSensitive ? sensitiveCaps : [])])];
  }
</script>

{#if capabilities.length === 0}
  <p class="text-sm text-muted-foreground">No capabilities are registered.</p>
{:else}
  <div class="space-y-2">
    <Label class="flex items-center gap-2 text-sm font-medium">
      <Switch checked={allSelected} onCheckedChange={(v) => toggleAll(v === true)} {disabled} />
      Select all
    </Label>
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
