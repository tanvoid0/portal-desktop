<!--
	Quick Run — pick a folder, queue blocks, run automation
-->
<script lang="ts">
  import { generateId } from '$lib/domains/shared/utils';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import Play from '~icons/fa6-solid/play';
  import Eye from '~icons/fa6-solid/eye';
  import Loader2 from '~icons/fa6-solid/spinner';
  import type { Block } from '$lib/domains/projects/pipelines';
  import { blockLibraryStore, blocks as blocksStore } from '$lib/domains/projects/pipelines';
  import {
    automation,
    type AutomationRunResult,
    type AutomationStepRef,
    type ResolvedExecutionStep,
  } from '$lib/domains/automation';
  import { projectService } from '$lib/domains/projects/services/projectService';
  import type { Project } from '$lib/domains/projects/types';
  import { getProjectPackageManager } from '$lib/domains/projects/utils/display';
  import { toast } from '$lib/utils/toast';
  import { invokeClient } from '$lib/utils/invokeClient';
  import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';
  import { PageHeader } from '$lib/components/shell';
  import QuickRunPreviewCard from './QuickRunPreviewCard.svelte';
  import QuickRunSetupCards from './QuickRunSetupCards.svelte';
  import QuickRunQueueCard from './QuickRunQueueCard.svelte';
  import QuickRunBlockLibrary from './QuickRunBlockLibrary.svelte';
  import QuickRunResultCard from './QuickRunResultCard.svelte';

  setBreadcrumbs([
    { label: 'Automation', href: '/automation' },
    { label: 'Quick Run', href: '/automation/run' },
  ]);

  interface QueuedBlock {
    id: string;
    blockId: string;
    name: string;
  }

  let blocks = $state<Block[]>([]);
  let projects = $state<Project[]>([]);
  let loadingBlocks = $state(true);
  let cwd = $state('');
  let packageManager = $state('npm');
  let queued = $state<QueuedBlock[]>([]);
  let previewSteps = $state<ResolvedExecutionStep[]>([]);
  let previewLoading = $state(false);
  let running = $state(false);
  let runResult = $state<AutomationRunResult | null>(null);
  let selectedProjectId = $state<string>('');

  onMount(async () => {
    try {
      await blockLibraryStore.loadBlocks();
      projects = await projectService.loadProjects();
    } catch (error) {
      toast.error('Failed to load automation data');
      console.error(error);
    } finally {
      loadingBlocks = false;
    }
  });

  $effect(() => {
    const unsubscribe = blocksStore.subscribe((b) => {
      blocks = b;
    });
    return unsubscribe;
  });

  const queuedBlockIds = $derived(new Set(queued.map((q) => q.blockId)));

  const canRun = $derived(cwd.trim().length > 0 && queued.length > 0 && !running);

  function applyPreset(preset: AutomationStepRef[]) {
    queued = preset.map((ref) => {
      const blockId = typeof ref === 'string' ? ref : ref.blockId;
      const block = blocks.find((b) => b.id === blockId);
      return {
        id: generateId(),
        blockId,
        name: block?.name ?? blockId,
      };
    });
    previewSteps = [];
    runResult = null;
  }

  function addBlock(block: Block) {
    if (queuedBlockIds.has(block.id)) return;
    queued = [...queued, { id: generateId(), blockId: block.id, name: block.name }];
    previewSteps = [];
    runResult = null;
  }

  function removeQueued(id: string) {
    queued = queued.filter((q) => q.id !== id);
    previewSteps = [];
  }

  function moveQueued(id: string, direction: -1 | 1) {
    const idx = queued.findIndex((q) => q.id === id);
    if (idx < 0) return;
    const next = idx + direction;
    if (next < 0 || next >= queued.length) return;
    const copy = [...queued];
    [copy[idx], copy[next]] = [copy[next], copy[idx]];
    queued = copy;
    previewSteps = [];
  }

  function handleProjectSelect(projectId: string) {
    selectedProjectId = projectId;
    if (!projectId) return;
    const project = projects.find((p) => p.id === projectId);
    if (project?.path) {
      cwd = project.path;
      const pm = getProjectPackageManager(project);
      if (pm) packageManager = pm.toLowerCase();
    }
  }

  function buildBlockRefs(): AutomationStepRef[] {
    return queued.map((q) => {
      if (q.blockId === 'install-npm') {
        return {
          blockId: q.blockId,
          parameters: { packageManager, installCommand: 'install' },
        };
      }
      return q.blockId;
    });
  }

  /** Backend owns the "folder exists" check; toast + false when it fails. */
  async function cwdIsValid(): Promise<boolean> {
    try {
      await invokeClient.post('validate_project_path', { path: cwd.trim() });
      return true;
    } catch (error) {
      toast.error(`Working directory: ${error instanceof Error ? error.message : String(error)}`);
      return false;
    }
  }

  async function handlePreview() {
    if (!cwd.trim()) {
      toast.error('Select a working directory first');
      return;
    }
    if (queued.length === 0) {
      toast.error('Add at least one block to the queue');
      return;
    }

    previewLoading = true;
    try {
      if (!(await cwdIsValid())) {
        previewSteps = [];
        return;
      }
      previewSteps = await automation.resolve({
        cwd: cwd.trim(),
        blocks: buildBlockRefs(),
        variables: { PACKAGE_MANAGER: packageManager },
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to preview plan');
      previewSteps = [];
    } finally {
      previewLoading = false;
    }
  }

  async function handleRun() {
    if (!canRun || !(await cwdIsValid())) return;

    running = true;
    runResult = null;
    previewSteps = [];

    try {
      const result = await automation.run({
        cwd: cwd.trim(),
        blocks: buildBlockRefs(),
        variables: { PACKAGE_MANAGER: packageManager },
        stopOnError: true,
      });
      runResult = result;

      if (result.success) {
        toast.success(`Completed ${result.steps.length} step(s)`);
      } else {
        const failed = result.steps.find((s) => s.status === 'failed');
        toast.error(failed?.error ?? 'Automation run failed');
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Run failed');
    } finally {
      running = false;
    }
  }
</script>

<svelte:head>
  <title>Quick Run - Automation - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Quick Run"
    description="Pick any folder, queue blocks, and run — no pipeline setup required"
  >
    {#snippet actions()}
      <Button onclick={handlePreview} disabled={!canRun && !cwd.trim()} variant="outline">
        <Eye class="mr-2 h-4 w-4" />
        Preview
      </Button>
      <Button onclick={handleRun} disabled={!canRun}>
        {#if running}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          Running…
        {:else}
          <Play class="mr-2 h-4 w-4" />
          Run
        {/if}
      </Button>
    {/snippet}
  </PageHeader>

  {#if previewLoading || previewSteps.length > 0}
    <QuickRunPreviewCard {cwd} loading={previewLoading} steps={previewSteps} />
  {/if}

  <div class="grid gap-6 lg:grid-cols-2">
    <div class="space-y-6">
      <QuickRunSetupCards
        {cwd}
        bind:packageManager
        {projects}
        {selectedProjectId}
        onCwdChange={(path) => {
          cwd = path;
          selectedProjectId = '';
          previewSteps = [];
          runResult = null;
        }}
        onProjectSelect={handleProjectSelect}
        onPreset={applyPreset}
      />
    </div>

    <div class="space-y-6">
      <QuickRunQueueCard items={queued} onMove={moveQueued} onRemove={removeQueued} />
      <QuickRunBlockLibrary {blocks} loading={loadingBlocks} {queuedBlockIds} onAdd={addBlock} />
    </div>
  </div>

  {#if runResult}
    <QuickRunResultCard result={runResult} />
  {/if}
</div>
