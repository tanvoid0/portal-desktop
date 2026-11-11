<script lang="ts">
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    capabilities: string[];
    selected: string[];
    disabled?: boolean;
  }

  let { capabilities, selected = $bindable([]), disabled = false }: Props = $props();

  const allSelected = $derived(
    capabilities.length > 0 && capabilities.every((c) => selected.includes(c))
  );

  function toggle(cap: string, checked: boolean) {
    selected = checked ? [...selected, cap] : selected.filter((c) => c !== cap);
  }

  function toggleAll() {
    selected = allSelected ? [] : [...capabilities];
  }
</script>

{#if capabilities.length === 0}
  <p class="text-sm text-muted-foreground">No capabilities are registered.</p>
{:else}
  <div class="space-y-2">
    <Button type="button" variant="ghost" size="sm" onclick={toggleAll} {disabled}>
      {allSelected ? 'Deselect all' : 'Select all'}
    </Button>
    <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
      {#each capabilities as cap}
        <Label class="flex items-center gap-2 text-sm font-normal">
          <Switch
            checked={selected.includes(cap)}
            onCheckedChange={(v) => toggle(cap, v === true)}
            {disabled}
          />
          {cap}
        </Label>
      {/each}
    </div>
  </div>
{/if}
