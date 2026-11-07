<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Plus from '~icons/fa6-solid/plus';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Link2 from '~icons/fa6-solid/link';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import X from '~icons/fa6-solid/xmark';
  import Icon from '@iconify/svelte';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { logger } from '$lib/domains/shared';
  import {
    ideService,
    type IdeConfig,
    type FrameworkIdeMapping,
    type Framework,
    type FrameworkGroup,
  } from '$lib/domains/ide';
  import FrameworkIdeMappingDialog from './FrameworkIdeMappingDialog.svelte';
  import FrameworkEditDialog from './FrameworkEditDialog.svelte';

  interface Props {
    ides: IdeConfig[];
  }

  let { ides }: Props = $props();

  let mappings = $state<FrameworkIdeMapping[]>([]);
  let isLoadingMappings = $state(false);
  // Suggested frameworks from the backend, and user-defined ones from the database.
  let frameworkGroups = $state<FrameworkGroup[]>([]);
  let frameworks = $state<Framework[]>([]);
  let isLoadingFrameworks = $state(false);

  let mappingDialogOpen = $state(false);
  let editingMapping = $state<FrameworkIdeMapping | null>(null);
  let mappingPrefill = $state('');
  let frameworkDialogOpen = $state(false);
  let editingFramework = $state<Framework | null>(null);

  onMount(() => {
    void Promise.all([loadMappings(), loadSuggestedFrameworks(), loadFrameworks()]);
  });

  /** Reload mappings (e.g. after an IDE was deleted and its mappings went with it). */
  export function reload() {
    return loadMappings();
  }

  async function loadMappings() {
    try {
      isLoadingMappings = true;
      mappings = await ideService.getAllFrameworkIdeMappings();
      logger.info('Framework IDE mappings loaded', {
        context: 'IdeSettings',
        count: mappings.length,
      });
    } catch (error) {
      toast.error('Failed to load framework IDE mappings', error);
    } finally {
      isLoadingMappings = false;
    }
  }

  async function loadSuggestedFrameworks() {
    try {
      isLoadingFrameworks = true;
      frameworkGroups = await ideService.getSuggestedFrameworks();
      logger.info('Suggested frameworks loaded', {
        context: 'IdeSettings',
        count: frameworkGroups.length,
      });
    } catch (error) {
      // Not critical, so no toast.
      logger.error('Failed to load suggested frameworks', { context: 'IdeSettings', error });
    } finally {
      isLoadingFrameworks = false;
    }
  }

  async function loadFrameworks() {
    try {
      frameworks = await ideService.getAllFrameworks();
      logger.info('Frameworks loaded', { context: 'IdeSettings', count: frameworks.length });
    } catch (error) {
      toast.error('Failed to load frameworks', error);
    }
  }

  function openMappingDialog(mapping: FrameworkIdeMapping | null, framework = '') {
    editingMapping = mapping;
    mappingPrefill = framework;
    mappingDialogOpen = true;
  }

  function openFrameworkDialog(framework: Framework | null) {
    editingFramework = framework;
    frameworkDialogOpen = true;
  }

  async function deleteFramework(framework: Framework) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${framework.name}"?`,
      'Delete framework'
    );
    if (!confirmed) return;

    try {
      await ideService.deleteFramework(framework.id);
      toast.success('Framework deleted successfully');
      await loadFrameworks();
    } catch (error: any) {
      toast.error('Failed to delete framework', error);
    }
  }

  async function deleteMapping(mapping: FrameworkIdeMapping) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete the mapping for "${mapping.framework}"?`,
      'Delete mapping'
    );
    if (!confirmed) return;

    try {
      await ideService.deleteFrameworkIdeMapping(mapping.framework);
      toast.success('Mapping deleted successfully');
      await loadMappings();
    } catch (error: any) {
      toast.error('Failed to delete mapping', error);
    }
  }

  function getIdeName(ideId: number): string {
    return ides.find((i) => i.id === ideId)?.name ?? `IDE #${ideId}`;
  }

  const mappedNames = $derived(new Set(mappings.map((m) => m.framework.toLowerCase())));

  const frameworksByCategory = $derived.by(() => {
    const byCategory: Record<string, Framework[]> = {};
    for (const fw of frameworks) (byCategory[fw.category || 'Other'] ??= []).push(fw);
    return Object.entries(byCategory);
  });

  // Suggested + user-defined, flattened, for icon lookup and the "add more" list.
  const allFrameworks = $derived([
    ...frameworkGroups.flatMap((g) => g.frameworks.map((fw) => ({ ...fw, isCustom: false }))),
    ...frameworks.map((fw) => ({
      name: fw.name,
      icon: fw.icon,
      category: fw.category,
      isCustom: true,
    })),
  ]);

  const unmappedByCategory = $derived.by(() => {
    const byCategory: Record<string, typeof allFrameworks> = {};
    for (const fw of allFrameworks) {
      if (!mappedNames.has(fw.name.toLowerCase())) (byCategory[fw.category] ??= []).push(fw);
    }
    return Object.entries(byCategory);
  });
</script>

<div class="flex items-center justify-between">
  <div>
    <h2 class="flex items-center gap-2 text-xl font-bold tracking-tight">
      <Link2 class="h-5 w-5" />
      Framework IDE Mappings
    </h2>
    <p class="text-sm text-muted-foreground">
      Map frameworks to their preferred IDEs for automatic project opening
    </p>
  </div>
  <div class="flex gap-2">
    <Button variant="outline" size="sm" onclick={() => openFrameworkDialog(null)}>
      <Plus class="mr-2 h-4 w-4" />
      Create Framework
    </Button>
    <Button variant="outline" size="sm" onclick={loadMappings} disabled={isLoadingMappings}>
      <RefreshCw class="mr-2 h-4 w-4" />
      Refresh
    </Button>
    <Button
      variant="outline"
      size="sm"
      onclick={() => openMappingDialog(null)}
      disabled={ides.length === 0}
    >
      <Plus class="mr-2 h-4 w-4" />
      Add Mapping
    </Button>
  </div>
</div>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Link2 class="h-5 w-5" />
      Framework IDE Mappings
      <Badge variant="outline">{mappings.length}</Badge>
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if isLoadingMappings}
      <div class="flex items-center justify-center py-8">
        <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
        <span class="ml-2 text-muted-foreground">Loading mappings...</span>
      </div>
    {:else if ides.length === 0}
      <div class="py-8 text-center">
        <Link2 class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
        <p class="mb-2 text-muted-foreground">No IDEs configured</p>
        <p class="mb-4 text-sm text-muted-foreground">
          Please add an IDE first before creating framework mappings
        </p>
      </div>
    {:else if mappings.length === 0}
      <!-- No mappings and at least one IDE: nothing is mapped yet, so every framework is offered. -->
      <div class="space-y-6">
        <div class="py-4 text-center">
          <Link2 class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
          <p class="mb-2 font-medium text-muted-foreground">No framework IDE mappings configured</p>
          <p class="mb-4 text-sm text-muted-foreground">
            Quickly add common frameworks below or create a custom mapping
          </p>
        </div>

        {#if isLoadingFrameworks}
          <div class="py-4 text-center">
            <Loader2 class="mx-auto h-4 w-4 animate-spin text-muted-foreground" />
          </div>
        {:else if frameworkGroups.length > 0 || frameworks.length > 0}
          <div class="space-y-4">
            <p class="text-sm font-medium text-muted-foreground">Frameworks</p>

            {#each frameworksByCategory as [category, categoryFrameworks]}
              <div>
                <p class="mb-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
                  {category}
                </p>
                <div class="flex flex-wrap gap-2">
                  {#each categoryFrameworks as framework}
                    <div class="group relative">
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => openMappingDialog(null, framework.name)}
                        class="h-8 pr-8"
                      >
                        {#if framework.icon_type === 'file'}
                          <img src={framework.icon} alt={framework.name} class="mr-1.5 h-3 w-3" />
                        {:else}
                          <Icon icon={framework.icon} class="mr-1.5 h-3 w-3" />
                        {/if}
                        {framework.name}
                      </Button>
                      <div
                        class="absolute top-1 right-1 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100"
                      >
                        <Button
                          variant="ghost"
                          size="sm"
                          class="h-6 w-6 p-0"
                          onclick={() => openFrameworkDialog(framework)}
                        >
                          <Edit class="h-3 w-3" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          class="h-6 w-6 p-0 text-destructive hover:text-destructive"
                          onclick={() => deleteFramework(framework)}
                        >
                          <X class="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}

            {#each frameworkGroups as group}
              {#if group.frameworks.length > 0}
                <div>
                  <p class="mb-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
                    {group.category}
                  </p>
                  <div class="flex flex-wrap gap-2">
                    {#each group.frameworks as framework}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => openMappingDialog(null, framework.name)}
                        class="h-8"
                      >
                        <Icon icon={framework.icon} class="mr-1.5 h-3 w-3" />
                        {framework.name}
                      </Button>
                    {/each}
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <div class="space-y-4">
        <div class="space-y-3">
          {#each mappings as mapping}
            {@const frameworkInfo = allFrameworks.find(
              (f) => f.name.toLowerCase() === mapping.framework.toLowerCase()
            )}
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
                <span class="text-sm font-medium">{getIdeName(mapping.ide_id)}</span>
              </div>

              <div class="flex items-center gap-2">
                <Button variant="ghost" size="sm" onclick={() => openMappingDialog(mapping)}>
                  <Edit class="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => deleteMapping(mapping)}
                  class="text-destructive hover:text-destructive"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </div>
          {/each}
        </div>

        {#if unmappedByCategory.length > 0}
          <div class="divider-edge-t divider-edge-full space-y-4 pt-4">
            <p class="mb-3 text-sm font-medium text-muted-foreground">Add More Frameworks</p>
            {#each unmappedByCategory as [category, categoryFrameworks]}
              <div>
                <p class="mb-2 text-xs font-medium tracking-wide text-muted-foreground uppercase">
                  {category}
                </p>
                <div class="flex flex-wrap gap-2">
                  {#each categoryFrameworks as framework}
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => openMappingDialog(null, framework.name)}
                      class="h-8"
                    >
                      {#if framework.icon && !framework.icon.startsWith('http') && !framework.icon.startsWith('/') && !framework.icon.includes(':')}
                        <img src={framework.icon} alt={framework.name} class="mr-1.5 h-3 w-3" />
                      {:else if framework.icon}
                        <Icon icon={framework.icon} class="mr-1.5 h-3 w-3" />
                      {/if}
                      {framework.name}
                    </Button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </CardContent>
</Card>

<FrameworkIdeMappingDialog
  bind:open={mappingDialogOpen}
  mapping={editingMapping}
  initialFramework={mappingPrefill}
  {ides}
  onSaved={loadMappings}
/>

<FrameworkEditDialog
  bind:open={frameworkDialogOpen}
  framework={editingFramework}
  onSaved={loadFrameworks}
/>
