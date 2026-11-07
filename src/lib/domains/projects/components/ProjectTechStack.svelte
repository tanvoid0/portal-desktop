<!--
  "What it's built with": badges for the detected stack.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import Layers from '~icons/fa6-solid/layer-group';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import type { Project, ProjectModule } from '$lib/domains/projects/types';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import TechIcon from '$lib/components/ui/tech-icon.svelte';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { getFrameworkColor } from '$lib/domains/projects/utils/display';
  import { stackIcon, summarizeStacks } from '$lib/domains/projects/utils/moduleStacks';

  interface Props {
    project: Project;
    modules?: ProjectModule[];
    refreshing?: boolean;
    onRedetect: () => void;
  }

  let { project, modules = [], refreshing = false, onRedetect }: Props = $props();

  let registryReady = $state(false);

  onMount(async () => {
    await projectIconRegistry.ensureLoaded();
    registryReady = true;
  });

  const frameworks = $derived(registryReady ? projectIconRegistry.resolveFrameworks(project) : []);
  const languages = $derived(registryReady ? projectIconRegistry.resolveLanguages(project) : []);
  const packageManagers = $derived(
    registryReady ? projectIconRegistry.resolvePackageManagers(project) : []
  );
  // The project's own stack wins; the module summary is the fallback for a container
  // repo with none recorded (a Tauri app's `src-tauri` must not hide Svelte/Tauri).
  const ownStack = $derived(frameworks.length + languages.length + packageManagers.length > 0);
  const isMonorepo = $derived(!ownStack && modules.length > 0);
  const hasStack = $derived(ownStack || isMonorepo);
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Layers class="h-5 w-5" />
      What it's built with
    </CardTitle>
  </CardHeader>
  <CardContent class="space-y-3">
    {#if !registryReady}
      <p class="text-sm text-muted-foreground">Loading…</p>
    {:else if hasStack}
      <div class="flex flex-wrap gap-2">
        {#if isMonorepo}
          {#each summarizeStacks(modules) as s (s.stack)}
            <Badge variant="secondary" class="gap-1.5">
              {#if stackIcon(s.stack)}<i class="{stackIcon(s.stack)} text-sm"></i>{/if}
              {s.count}
              {s.label}
            </Badge>
          {/each}
        {:else}
          {#each frameworks as item (item.name)}
            <Badge variant="outline" class="gap-1.5 {getFrameworkColor(item.name)}">
              <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
              {item.name}
            </Badge>
          {/each}
          {#each languages as item (item.name)}
            <Badge variant="secondary" class="gap-1.5">
              <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
              {item.name}
            </Badge>
          {/each}
          {#each packageManagers as item (item.name)}
            <Badge variant="outline" class="gap-1.5">
              <TechIcon icon={item.icon} iconType={item.icon_type} size="xs" alt={item.name} />
              {item.name}
            </Badge>
          {/each}
        {/if}
      </div>
    {:else}
      <div class="flex flex-wrap items-center gap-3">
        <p class="text-sm text-muted-foreground">Stack not detected yet.</p>
        <Button
          variant="outline"
          size="sm"
          onclick={onRedetect}
          disabled={refreshing}
          class="gap-2"
        >
          <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
          Re-detect
        </Button>
      </div>
    {/if}
  </CardContent>
</Card>
