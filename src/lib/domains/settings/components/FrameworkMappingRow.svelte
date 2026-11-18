<!-- One framework -> IDE mapping row with edit/delete actions (and framework edit/delete if custom). -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import X from '~icons/fa6-solid/xmark';
  import Icon from '@iconify/svelte';
  import type { Framework, FrameworkIdeMapping } from '$lib/domains/ide';
  import type { FrameworkChip } from './frameworkChips';

  interface Props {
    mapping: FrameworkIdeMapping;
    ideName: string;
    frameworkInfo?: FrameworkChip;
    onEditMapping: (m: FrameworkIdeMapping) => void;
    onDeleteMapping: (m: FrameworkIdeMapping) => void;
    onEditFramework: (f: Framework) => void;
    onDeleteFramework: (f: Framework) => void;
  }

  let {
    mapping,
    ideName,
    frameworkInfo,
    onEditMapping,
    onDeleteMapping,
    onEditFramework,
    onDeleteFramework,
  }: Props = $props();
</script>

<div
  class="flex items-center justify-between rounded-md border p-4 transition-colors hover:bg-accent"
>
  <div class="flex items-center gap-4">
    {#if frameworkInfo?.icon}
      {#if !frameworkInfo.isCustom || frameworkInfo.icon.includes(':')}
        <Icon icon={frameworkInfo.icon} class="h-5 w-5" />
      {:else}
        <img src={frameworkInfo.icon} alt={mapping.framework} class="h-5 w-5" />
      {/if}
    {/if}
    <Badge variant="secondary" class="font-medium">{mapping.framework}</Badge>
    <ArrowRight class="h-4 w-4 text-muted-foreground" />
    <span class="text-sm font-medium">{ideName}</span>
  </div>

  <div class="flex items-center gap-2">
    {#if frameworkInfo?.isCustom && frameworkInfo.source}
      {@const custom = frameworkInfo.source}
      <Button
        variant="ghost"
        size="sm"
        aria-label={`Edit ${custom.name}`}
        title="Edit the framework definition"
        onclick={() => onEditFramework(custom)}
      >
        <Edit class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        aria-label={`Delete ${custom.name}`}
        title="Delete the framework"
        class="text-destructive hover:text-destructive"
        onclick={() => onDeleteFramework(custom)}
      >
        <X class="h-3.5 w-3.5" />
      </Button>
    {/if}
    <Button
      variant="ghost"
      size="sm"
      aria-label={`Edit mapping for ${mapping.framework}`}
      onclick={() => onEditMapping(mapping)}
    >
      <Edit class="h-4 w-4" />
    </Button>
    <Button
      variant="ghost"
      size="sm"
      aria-label={`Delete mapping for ${mapping.framework}`}
      onclick={() => onDeleteMapping(mapping)}
      class="text-destructive hover:text-destructive"
    >
      <Trash2 class="h-4 w-4" />
    </Button>
  </div>
</div>
