<!--
	Edit Script Page
	Same form as /scripts/new, saving over an existing script.
-->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { PageHeader } from '$lib/components/shell';
  import Save from '~icons/fa6-solid/floppy-disk';
  import type { CreateBlockRequest } from '$lib/domains/projects/pipelines';
  import { blockLibraryService } from '$lib/domains/projects/pipelines';
  import { ScriptForm } from '$lib/domains/scripts';
  import { toast } from '$lib/utils/toast';
  import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';

  let loading = $state(true);
  let saving = $state(false);

  let formData = $state<CreateBlockRequest>({
    name: '',
    description: '',
    category: 'utility',
    parameters: [],
    command: '',
    executionType: 'script',
    defaultConfig: {},
    tags: [],
  });

  const scriptId = $derived(page.params.id ?? '');
  const isValid = $derived(formData.name && formData.description && formData.command);

  onMount(async () => {
    try {
      const script = await blockLibraryService.getBlock(scriptId);
      if (!script) {
        toast.error('Script not found');
        goto('/scripts');
        return;
      }

      setBreadcrumbs([
        { label: 'Scripts', href: '/scripts' },
        { label: script.name, href: `/scripts/${scriptId}` },
        { label: 'Edit', href: `/scripts/${scriptId}/edit` },
      ]);

      formData = {
        name: script.name,
        description: script.description,
        category: script.category,
        parameters: [...script.parameters],
        command: script.command,
        executionType: script.executionType,
        defaultConfig: { ...script.defaultConfig },
        tags: [...(script.tags || [])],
      };
    } catch (error) {
      console.error('Failed to load script', error);
      toast.error('Failed to load script');
    } finally {
      loading = false;
    }
  });

  async function handleSave() {
    if (saving) return;
    if (!formData.name || !formData.description || !formData.command) {
      toast.error('Please fill in all required fields');
      return;
    }

    saving = true;
    try {
      // Build defaultConfig from parameters with defaultValue
      const defaultConfig: Record<string, unknown> = {};
      formData.parameters.forEach((p) => {
        if (p.defaultValue !== undefined && p.defaultValue !== '') {
          defaultConfig[p.name] = p.defaultValue;
        }
      });
      formData.defaultConfig = defaultConfig;

      await blockLibraryService.updateBlock(scriptId, formData);
      toast.success('Script updated successfully');
      goto(`/scripts/${scriptId}`);
    } catch (error) {
      console.error('Failed to update script', error);
      toast.error('Failed to update script');
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>Edit {formData.name || 'Script'} - Portal Desktop</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
  <PageHeader title="Edit script" description={formData.name}>
    {#snippet actions()}
      <Button variant="outline" size="sm" onclick={() => goto(`/scripts/${scriptId}`)}>
        Cancel
      </Button>
      <Button size="sm" onclick={handleSave} disabled={!isValid || saving}>
        <Save class="mr-2 h-4 w-4" />
        {saving ? 'Saving...' : 'Save'}
      </Button>
    {/snippet}
  </PageHeader>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <p class="text-muted-foreground">Loading script...</p>
    </div>
  {:else}
    <ScriptForm bind:formData />
  {/if}
</div>
