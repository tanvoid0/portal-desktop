<!--
	IDE Settings - IDE configuration and framework -> IDE mappings
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { Separator } from '$lib/components/ui/separator';
  import { toast } from '$lib/utils/toast';
  import { logger } from '$lib/domains/shared';
  import { ideService, type IdeConfig } from '$lib/domains/ide';
  import IdeList from './IdeList.svelte';
  import FrameworkIdeMappings from './FrameworkIdeMappings.svelte';

  let ides = $state<IdeConfig[]>([]);
  let isLoadingIdes = $state(false);
  let mappingsSection: FrameworkIdeMappings | undefined = $state();

  async function loadIdes() {
    try {
      isLoadingIdes = true;
      ides = await ideService.getAllIdes();
      logger.info('IDEs loaded', { context: 'IdeSettings', count: ides.length });
    } catch (error) {
      toast.error('Failed to load IDEs', error);
    } finally {
      isLoadingIdes = false;
    }
  }

  onMount(() => {
    void loadIdes();
  });
</script>

<div class="space-y-6">
  <IdeList
    {ides}
    isLoading={isLoadingIdes}
    onRefresh={loadIdes}
    onChanged={loadIdes}
    onDeleted={() => mappingsSection?.reload()}
  />

  <Separator class="my-6" />

  <FrameworkIdeMappings bind:this={mappingsSection} {ides} />
</div>
