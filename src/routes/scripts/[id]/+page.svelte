<!--
	Script Detail Page
	Runs a script, shows its history and its definition. Editing lives at /scripts/[id]/edit.
-->
<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Label } from '$lib/components/ui/label';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import Trash2 from '~icons/fa6-solid/trash';
  import FileCode from '~icons/fa6-solid/file-code';
  import Download from '~icons/fa6-solid/download';
  import Copy from '~icons/fa6-solid/copy';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import History from '~icons/fa6-solid/clock-rotate-left';
  import type { Block } from '$lib/domains/projects/pipelines';
  import { blockLibraryService, blockLibraryStore } from '$lib/domains/projects/pipelines';
  import {
    ScriptExecutionHistory,
    ScriptRunPanel,
    type ScriptExecutionInfo,
  } from '$lib/domains/scripts';
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';

  let script: Block | null = $state(null);
  let loading = $state(true);
  let currentExecution: ScriptExecutionInfo | null = $state(null);
  let showHistory = $state(false);
  let historyKey = $state(0);

  const scriptId = $derived(page.params.id ?? '');

  onMount(async () => {
    // Old links opened the inline editor with ?edit=true; it is a route now.
    if (page.url.searchParams.get('edit') === 'true') {
      goto(`/scripts/${scriptId}/edit`, { replaceState: true });
      return;
    }
    await loadScript();
  });

  async function loadScript() {
    loading = true;
    try {
      await blockLibraryStore.loadBlocks();
      script = await blockLibraryService.getBlock(scriptId);

      if (script) {
        setBreadcrumbs([
          { label: 'Scripts', href: '/scripts' },
          { label: script.name, href: `/scripts/${scriptId}` },
        ]);
      } else {
        toast.error('Script not found');
        goto('/scripts');
      }
    } catch (error) {
      console.error('Failed to load script', error);
      toast.error('Failed to load script');
    } finally {
      loading = false;
    }
  }

  async function handleDelete() {
    const confirmed = await confirmAction(
      'Are you sure you want to delete this script?',
      'Delete script',
      { confirmLabel: 'Delete', destructive: true }
    );
    if (!confirmed) return;

    try {
      await blockLibraryService.deleteBlock(scriptId);
      toast.success('Script deleted successfully');
      goto('/scripts');
    } catch (error) {
      console.error('Failed to delete script', error);
      toast.error('Failed to delete script');
    }
  }

  function handleExport() {
    if (!script) return;
    try {
      const dataStr = JSON.stringify(script, null, 2);
      const dataBlob = new Blob([dataStr], { type: 'application/json' });
      const url = URL.createObjectURL(dataBlob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${script.name.toLowerCase().replace(/\s+/g, '-')}.block.json`;
      link.click();
      URL.revokeObjectURL(url);
      toast.success('Script exported successfully');
    } catch (error) {
      console.error('Failed to export script', error);
      toast.error('Failed to export script');
    }
  }

  function handleCopyCommand() {
    if (!script) return;
    navigator.clipboard.writeText(script.command);
    toast.success('Command copied to clipboard');
  }
</script>

<svelte:head>
  <title>{script?.name || 'Script'} - Portal Desktop</title>
</svelte:head>

<div class="mx-auto max-w-4xl">
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <p class="text-muted-foreground">Loading script...</p>
    </div>
  {:else if script}
    <!-- Header -->
    <div class="mb-6 flex items-center gap-4">
      <Button
        variant="ghost"
        size="icon"
        aria-label="Back to scripts"
        onclick={() => goto('/scripts')}
      >
        <ArrowLeft class="h-5 w-5" />
      </Button>
      <div class="flex-1">
        <h1 class="flex items-center gap-2 text-2xl font-bold">
          <FileCode class="h-6 w-6" />
          {script.name}
        </h1>
        <p class="text-muted-foreground">{script.description}</p>
      </div>
      <div class="flex gap-2">
        <Button variant="outline" onclick={() => (showHistory = !showHistory)}>
          <History class="mr-2 h-4 w-4" />
          History
        </Button>
        <Button variant="outline" onclick={handleExport}>
          <Download class="mr-2 h-4 w-4" />
          Export
        </Button>
        <Button variant="outline" onclick={() => goto(`/scripts/${scriptId}/edit`)}>
          <Edit class="mr-2 h-4 w-4" />
          Edit
        </Button>
        <Button variant="destructive" onclick={handleDelete}>
          <Trash2 class="mr-2 h-4 w-4" />
          Delete
        </Button>
      </div>
    </div>

    <div class="space-y-6">
      {#if showHistory}
        <ScriptExecutionHistory
          {scriptId}
          refreshKey={historyKey}
          selectedId={currentExecution?.id ?? null}
          onSelect={(execution) => {
            currentExecution = execution;
            showHistory = false;
          }}
        />
      {/if}

      <ScriptRunPanel {script} bind:currentExecution onExecuted={() => historyKey++} />

      <!-- Command -->
      <Card>
        <CardHeader>
          <CardTitle>Command</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="group relative rounded-lg bg-muted p-4">
            <pre class="font-mono text-sm whitespace-pre-wrap">{script.command}</pre>
            <Button
              variant="ghost"
              size="sm"
              class="absolute top-2 right-2 opacity-0 transition-opacity group-hover:opacity-100"
              onclick={handleCopyCommand}
            >
              <Copy class="mr-1 h-4 w-4" />
              Copy
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Details -->
      <Card>
        <CardHeader>
          <CardTitle>Details</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="grid grid-cols-3 gap-6">
            <div>
              <Label class="text-xs text-muted-foreground">Category</Label>
              <p class="mt-1 font-medium">{script.category}</p>
            </div>
            <div>
              <Label class="text-xs text-muted-foreground">Execution Type</Label>
              <p class="mt-1 font-medium">{script.executionType}</p>
            </div>
            <div>
              <Label class="text-xs text-muted-foreground">Version</Label>
              <p class="mt-1 font-medium">{script.version}</p>
            </div>
          </div>
          {#if script.tags && script.tags.length > 0}
            <div class="mt-4">
              <Label class="text-xs text-muted-foreground">Tags</Label>
              <div class="mt-2 flex flex-wrap gap-1">
                {#each script.tags as tag}
                  <Badge variant="secondary">{tag}</Badge>
                {/each}
              </div>
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- Parameters Reference -->
      {#if script.parameters.length > 0}
        <Card>
          <CardHeader>
            <CardTitle>Parameters Reference ({script.parameters.length})</CardTitle>
          </CardHeader>
          <CardContent>
            <div class="divide-y rounded-lg border">
              {#each script.parameters as param}
                <div class="p-4">
                  <div class="flex items-center gap-2">
                    <code class="font-mono font-medium">{param.name}</code>
                    <Badge variant="outline">{param.type}</Badge>
                    {#if param.required}
                      <Badge variant="destructive" class="text-xs">required</Badge>
                    {/if}
                  </div>
                  {#if param.description}
                    <p class="mt-1 text-sm text-muted-foreground">
                      {param.description}
                    </p>
                  {/if}
                  {#if param.defaultValue !== undefined && param.defaultValue !== ''}
                    <p class="mt-1 text-xs text-muted-foreground">
                      Default: <code class="rounded bg-muted px-1">{param.defaultValue}</code>
                    </p>
                  {/if}
                  {#if param.options && param.options.length > 0}
                    <p class="mt-1 text-xs text-muted-foreground">
                      Options: {param.options.join(', ')}
                    </p>
                  {/if}
                </div>
              {/each}
            </div>
          </CardContent>
        </Card>
      {/if}

      <!-- JSON -->
      <Card>
        <CardHeader>
          <CardTitle>JSON Definition</CardTitle>
          <CardDescription>Full script configuration as JSON</CardDescription>
        </CardHeader>
        <CardContent>
          <pre
            class="max-h-80 overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">{JSON.stringify(
              script,
              null,
              2
            )}</pre>
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
