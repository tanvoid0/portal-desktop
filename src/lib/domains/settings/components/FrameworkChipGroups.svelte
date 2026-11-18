<!-- Framework chips grouped by category. Clicking a chip picks it; custom ones can be edited/deleted. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import X from '~icons/fa6-solid/xmark';
  import Icon from '@iconify/svelte';
  import type { Framework } from '$lib/domains/ide';
  import type { FrameworkChip } from './frameworkChips';

  interface Props {
    groups: [string, FrameworkChip[]][];
    onPick: (name: string) => void;
    onEdit: (framework: Framework) => void;
    onDelete: (framework: Framework) => void;
  }

  let { groups, onPick, onEdit, onDelete }: Props = $props();
</script>

{#each groups as [category, categoryFrameworks]}
  <div>
    <p class="mb-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
      {category}
    </p>
    <div class="flex flex-wrap gap-2">
      {#each categoryFrameworks as framework}
        <div class="group inline-flex items-center gap-1">
          <Button variant="outline" size="sm" onclick={() => onPick(framework.name)} class="h-8">
            {#if framework.icon_type === 'file'}
              <img src={framework.icon} alt={framework.name} class="mr-1.5 h-3 w-3" />
            {:else if framework.icon}
              <Icon icon={framework.icon} class="mr-1.5 h-3 w-3" />
            {/if}
            {framework.name}
          </Button>
          {#if framework.isCustom && framework.source}
            {@const custom = framework.source}
            <!-- Beside the chip, not on top of it — an overlay here would cover the label
                 and swallow clicks meant for the chip's own button (QA-102). -->
            <div class="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
              <Button
                variant="ghost"
                size="icon-sm"
                class="h-6 w-6"
                aria-label={`Edit ${custom.name}`}
                onclick={() => onEdit(custom)}
              >
                <Edit class="h-3 w-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                class="h-6 w-6 text-destructive hover:text-destructive"
                aria-label={`Delete ${custom.name}`}
                onclick={() => onDelete(custom)}
              >
                <X class="h-3 w-3" />
              </Button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/each}
