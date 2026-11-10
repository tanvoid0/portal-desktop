<script lang="ts">
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';

  interface Props {
    capabilities: string[];
    selected: string[];
    disabled?: boolean;
  }

  let { capabilities, selected = $bindable([]), disabled = false }: Props = $props();

  function toggle(cap: string, checked: boolean) {
    selected = checked ? [...selected, cap] : selected.filter((c) => c !== cap);
  }
</script>

{#if capabilities.length === 0}
  <p class="text-sm text-muted-foreground">No capabilities are registered.</p>
{:else}
  <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
    {#each capabilities as cap}
      <Label class="flex items-center gap-2 text-sm font-normal">
        <Checkbox
          checked={selected.includes(cap)}
          onCheckedChange={(v) => toggle(cap, v === true)}
          {disabled}
        />
        {cap}
      </Label>
    {/each}
  </div>
{/if}
