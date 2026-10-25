<!--
  One-line summary strip for the project detail page: stack, branch, path, module count.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { Check, Copy, GitBranch, Layers } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import type { Project, ProjectModule } from '$lib/domains/projects/types';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { getProjectGitBranch } from '$lib/domains/projects/utils/display';
  import { describeModules } from '$lib/domains/projects/utils/moduleStacks';

  interface Props {
    project: Project;
    modules: ProjectModule[];
  }

  let { project, modules }: Props = $props();

  const MAX_BADGES = 3;

  let registryReady = $state(false);
  let copied = $state(false);

  onMount(async () => {
    await projectIconRegistry.ensureLoaded();
    registryReady = true;
  });

  const techNames = $derived(
    registryReady
      ? [
          ...projectIconRegistry.resolveFrameworks(project),
          ...projectIconRegistry.resolveLanguages(project),
        ].map((t) => t.name)
      : []
  );
  const branch = $derived(getProjectGitBranch(project));

  async function copyPath() {
    await navigator.clipboard.writeText(project.path);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<div
  class="flex flex-wrap items-center gap-x-5 gap-y-2 rounded-lg border bg-card px-4 py-2.5 text-sm"
>
  <div class="flex min-w-0 items-center gap-2">
    {#if modules.length > 0}
      <span class="font-medium">{describeModules(modules)}</span>
    {:else}
      {#each techNames.slice(0, MAX_BADGES) as name}
        <Badge variant="secondary">{name}</Badge>
      {/each}
      {#if techNames.length > MAX_BADGES}
        <span class="text-muted-foreground">+{techNames.length - MAX_BADGES}</span>
      {/if}
      {#if techNames.length === 0}
        <span class="text-muted-foreground">No stack detected</span>
      {/if}
    {/if}
  </div>

  {#if branch}
    <span class="flex items-center gap-1.5 text-muted-foreground" title="Git branch">
      <GitBranch class="h-4 w-4" />
      {branch}
      {#if project.has_uncommitted_changes}
        <span class="h-2 w-2 rounded-full bg-amber-500" title="Uncommitted changes"></span>
      {/if}
    </span>
  {/if}

  <div class="flex max-w-full min-w-0 flex-1 items-center gap-1 text-muted-foreground">
    <span class="truncate font-mono text-xs" title={project.path}>{project.path}</span>
    <Button
      variant="ghost"
      size="sm"
      class="h-6 w-6 shrink-0 p-0"
      onclick={copyPath}
      title="Copy path"
    >
      {#if copied}
        <Check class="h-3.5 w-3.5" />
      {:else}
        <Copy class="h-3.5 w-3.5" />
      {/if}
    </Button>
  </div>

  {#if modules.length > 0}
    <span class="flex items-center gap-1.5 text-muted-foreground">
      <Layers class="h-4 w-4" />
      {modules.length} modules
    </span>
  {/if}
</div>
