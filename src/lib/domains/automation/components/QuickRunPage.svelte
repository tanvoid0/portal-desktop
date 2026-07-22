<!--
	Quick Run — pick a folder, queue blocks, run automation
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import { Label } from "$lib/components/ui/label";
  import FilePicker from "$lib/components/ui/file-picker.svelte";
  import Select from "$lib/components/ui/select.svelte";
  import {
    Play,
    FolderOpen,
    Plus,
    Trash2,
    ChevronUp,
    ChevronDown,
    Eye,
    CheckCircle,
    XCircle,
    Loader2,
    SkipForward,
    Zap,
  } from "@lucide/svelte";
  import type { Block } from "$lib/domains/projects/pipelines";
  import {
    blockLibraryStore,
    blocks as blocksStore,
  } from "$lib/domains/projects/pipelines";
  import {
    automation,
    type AutomationRunResult,
    type AutomationStepRef,
    type ResolvedExecutionStep,
  } from "$lib/domains/automation";
  import { projectService } from "$lib/domains/projects/services/projectService";
  import type { Project } from "$lib/domains/projects/types";
  import { getProjectPackageManager } from "$lib/domains/projects/utils/display";
  import { toast } from "$lib/utils/toast";
  import { setBreadcrumbs } from "$lib/domains/shared/stores/breadcrumbStore";
  import { PageHeader, PageLoading } from "$lib/components/shell";

  setBreadcrumbs([
    { label: "Automation", href: "/automation" },
    { label: "Quick Run", href: "/automation/run" },
  ]);

  interface QueuedBlock {
    id: string;
    blockId: string;
    name: string;
  }

  let blocks = $state<Block[]>([]);
  let projects = $state<Project[]>([]);
  let loadingBlocks = $state(true);
  let cwd = $state("");
  let packageManager = $state("npm");
  let blockSearch = $state("");
  let queued = $state<QueuedBlock[]>([]);
  let previewSteps = $state<ResolvedExecutionStep[]>([]);
  let previewLoading = $state(false);
  let running = $state(false);
  let runResult = $state<AutomationRunResult | null>(null);
  let selectedProjectId = $state<string>("");

  onMount(async () => {
    try {
      await blockLibraryStore.loadBlocks();
      projects = await projectService.loadProjects();
    } catch (error) {
      toast.error("Failed to load automation data");
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

  const filteredBlocks = $derived.by(() => {
    if (!blockSearch.trim()) return blocks;
    const q = blockSearch.toLowerCase();
    return blocks.filter(
      (b) =>
        b.name.toLowerCase().includes(q) ||
        b.description.toLowerCase().includes(q) ||
        b.tags.some((t) => t.toLowerCase().includes(q)),
    );
  });

  const queuedBlockIds = $derived(new Set(queued.map((q) => q.blockId)));

  const canRun = $derived(
    cwd.trim().length > 0 && queued.length > 0 && !running,
  );

  function applyPreset(preset: AutomationStepRef[]) {
    queued = preset.map((ref) => {
      const blockId = typeof ref === "string" ? ref : ref.blockId;
      const block = blocks.find((b) => b.id === blockId);
      return {
        id: crypto.randomUUID(),
        blockId,
        name: block?.name ?? blockId,
      };
    });
    previewSteps = [];
    runResult = null;
  }

  function addBlock(block: Block) {
    if (queuedBlockIds.has(block.id)) return;
    queued = [
      ...queued,
      { id: crypto.randomUUID(), blockId: block.id, name: block.name },
    ];
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
      if (q.blockId === "install-npm") {
        return {
          blockId: q.blockId,
          parameters: { packageManager, installCommand: "install" },
        };
      }
      return q.blockId;
    });
  }

  async function handlePreview() {
    if (!cwd.trim()) {
      toast.error("Select a working directory first");
      return;
    }
    if (queued.length === 0) {
      toast.error("Add at least one block to the queue");
      return;
    }

    previewLoading = true;
    try {
      previewSteps = await automation.resolve({
        cwd: cwd.trim(),
        blocks: buildBlockRefs(),
        variables: { PACKAGE_MANAGER: packageManager },
      });
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to preview plan",
      );
      previewSteps = [];
    } finally {
      previewLoading = false;
    }
  }

  async function handleRun() {
    if (!canRun) return;

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
        const failed = result.steps.find((s) => s.status === "failed");
        toast.error(failed?.error ?? "Automation run failed");
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Run failed");
    } finally {
      running = false;
    }
  }

  function stepStatusIcon(status: string) {
    switch (status) {
      case "success":
        return CheckCircle;
      case "failed":
        return XCircle;
      case "skipped":
        return SkipForward;
      default:
        return Loader2;
    }
  }
</script>

<svelte:head>
  <title>Quick Run - Automation - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto space-y-6 p-6">
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

  <div class="grid gap-6 lg:grid-cols-2">
    <!-- Left: setup -->
    <div class="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2 text-lg">
            <FolderOpen class="h-5 w-5" />
            Working Directory
          </CardTitle>
          <CardDescription>
            Any project folder or directory on disk
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <FilePicker
            label="Directory"
            value={cwd}
            selectFolder={true}
            placeholder="D:/projects/my-app"
            onChange={(path) => {
              cwd = path;
              selectedProjectId = "";
              previewSteps = [];
              runResult = null;
            }}
          />

          {#if projects.length > 0}
            <div class="space-y-2">
              <Label>Or pick a registered project</Label>
              <Select
                options={[
                  { value: "", label: "— Custom path —" },
                  ...projects.map((p) => ({
                    value: p.id,
                    label: p.name,
                  })),
                ]}
                value={selectedProjectId}
                placeholder="Select project"
                onValueChange={handleProjectSelect}
              />
            </div>
          {/if}

          <div class="space-y-2">
            <Label for="package-manager">Package manager</Label>
            <Select
              id="package-manager"
              options={[
                { value: "npm", label: "npm" },
                { value: "pnpm", label: "pnpm" },
                { value: "yarn", label: "yarn" },
                { value: "bun", label: "bun" },
              ]}
              bind:value={packageManager}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2 text-lg">
            <Zap class="h-5 w-5" />
            Presets
          </CardTitle>
          <CardDescription>Common sequences — customize via queue after applying</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-wrap gap-2">
          <Button
            variant="outline"
            size="sm"
            onclick={() => applyPreset(automation.presets.install(packageManager))}
          >
            Install
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => applyPreset(automation.presets.test())}
          >
            Test
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => applyPreset(automation.presets.build())}
          >
            Build
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => applyPreset(automation.presets.ci(packageManager))}
          >
            Full CI
          </Button>
        </CardContent>
      </Card>
    </div>

    <!-- Right: queue + blocks -->
    <div class="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle class="text-lg">Run Queue</CardTitle>
          <CardDescription>
            {queued.length === 0
              ? "Add blocks below — they run in order"
              : `${queued.length} block(s) queued`}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {#if queued.length === 0}
            <p class="py-6 text-center text-sm text-muted-foreground">
              No blocks queued. Use a preset or add from the library.
            </p>
          {:else}
            <div class="space-y-2">
              {#each queued as item, index (item.id)}
                <div
                  class="flex items-center gap-2 rounded-md border bg-muted/30 px-3 py-2"
                >
                  <span
                    class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary"
                  >
                    {index + 1}
                  </span>
                  <span class="min-w-0 flex-1 truncate text-sm font-medium">
                    {item.name}
                  </span>
                  <div class="flex shrink-0 gap-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      class="h-7 w-7 p-0"
                      disabled={index === 0}
                      onclick={() => moveQueued(item.id, -1)}
                    >
                      <ChevronUp class="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      class="h-7 w-7 p-0"
                      disabled={index === queued.length - 1}
                      onclick={() => moveQueued(item.id, 1)}
                    >
                      <ChevronDown class="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      class="h-7 w-7 p-0 text-destructive"
                      onclick={() => removeQueued(item.id)}
                    >
                      <Trash2 class="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle class="text-lg">Block Library</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3">
          <Input
            bind:value={blockSearch}
            placeholder="Search blocks…"
            class="w-full"
          />
          {#if loadingBlocks}
            <PageLoading message="Loading blocks…" />
          {:else}
            <div class="max-h-64 space-y-1 overflow-y-auto">
              {#each filteredBlocks as block (block.id)}
                <Button
                  type="button"
                  variant="outline"
                  class="h-auto w-full justify-between rounded-md px-3 py-2 text-left transition-colors hover:bg-accent disabled:opacity-50"
                  disabled={queuedBlockIds.has(block.id)}
                  onclick={() => addBlock(block)}
                >
                  <div class="min-w-0">
                    <p class="truncate text-sm font-medium">{block.name}</p>
                    <p class="truncate text-xs text-muted-foreground">
                      {block.description}
                    </p>
                  </div>
                  <div class="ml-2 flex shrink-0 items-center gap-2">
                    <Badge variant="outline" class="text-xs">
                      {block.category}
                    </Badge>
                    {#if !queuedBlockIds.has(block.id)}
                      <Plus class="h-4 w-4 text-muted-foreground" />
                    {/if}
                  </div>
                </Button>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>
    </div>
  </div>

  <!-- Preview -->
  {#if previewLoading || previewSteps.length > 0}
    <Card>
      <CardHeader>
        <CardTitle class="text-lg">Preview</CardTitle>
        <CardDescription>Resolved commands for {cwd || "…"}</CardDescription>
      </CardHeader>
      <CardContent>
        {#if previewLoading}
          <PageLoading message="Resolving commands…" />
        {:else}
          <div class="space-y-2">
            {#each previewSteps as step, i}
              <div class="rounded-md border bg-muted/20 p-3">
                <p class="mb-1 text-sm font-medium">
                  {i + 1}. {step.name}
                </p>
                <code class="block break-all text-xs text-muted-foreground">
                  {step.command}
                </code>
              </div>
            {/each}
          </div>
        {/if}
      </CardContent>
    </Card>
  {/if}

  <!-- Results -->
  {#if runResult}
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2 text-lg">
          {#if runResult.success}
            <CheckCircle class="h-5 w-5 text-green-600" />
            Run completed
          {:else}
            <XCircle class="h-5 w-5 text-destructive" />
            Run finished with errors
          {/if}
        </CardTitle>
        <CardDescription>{runResult.cwd}</CardDescription>
      </CardHeader>
      <CardContent class="space-y-3">
        {#each runResult.steps as step, i}
          {@const Icon = stepStatusIcon(step.status)}
          <div class="rounded-md border p-3">
            <div class="mb-2 flex items-center gap-2">
              <Icon
                class="h-4 w-4 {step.status === 'success'
                  ? 'text-green-600'
                  : step.status === 'failed'
                    ? 'text-destructive'
                    : 'text-muted-foreground'}"
              />
              <span class="text-sm font-medium">
                {i + 1}. {step.name}
              </span>
              <Badge variant="outline" class="text-xs capitalize">
                {step.status}
              </Badge>
              {#if step.exitCode != null}
                <span class="text-xs text-muted-foreground">
                  exit {step.exitCode}
                </span>
              {/if}
            </div>
            <code class="mb-2 block break-all text-xs text-muted-foreground">
              {step.command}
            </code>
            {#if step.output}
              <pre
                class="max-h-40 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap"
              >{step.output}</pre>
            {/if}
            {#if step.error}
              <p class="mt-1 text-xs text-destructive">{step.error}</p>
            {/if}
          </div>
        {/each}
      </CardContent>
    </Card>
  {/if}
</div>
