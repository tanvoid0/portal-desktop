<!--
  Displays frameworks, package managers, and languages for a project.
-->

<script lang="ts">
  import { onMount } from "svelte";
  import type { Project } from "$lib/domains/projects/types";
  import { Badge } from "$lib/components/ui/badge";
  import TechIcon from "$lib/components/ui/tech-icon.svelte";
  import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
  import { getFrameworkColor } from "$lib/domains/projects/utils/display";

  interface Props {
    project: Project;
    showEmpty?: boolean;
  }

  let { project, showEmpty = true }: Props = $props();

  let registryReady = $state(false);

  onMount(async () => {
    await projectIconRegistry.ensureLoaded();
    registryReady = true;
  });

  const frameworks = $derived(projectIconRegistry.resolveFrameworks(project));
  const packageManagers = $derived(
    projectIconRegistry.resolvePackageManagers(project),
  );
  const languages = $derived(projectIconRegistry.resolveLanguages(project));
  const hasTechStack = $derived(
    frameworks.length > 0 ||
      packageManagers.length > 0 ||
      languages.length > 0,
  );
</script>

{#if !registryReady}
  <p class="text-sm text-muted-foreground">Loading tech stack...</p>
{:else if hasTechStack}
  <div class="space-y-4">
    {#if frameworks.length > 0}
      <div>
        <p class="mb-2 text-sm font-medium text-muted-foreground">Frameworks</p>
        <div class="flex flex-wrap gap-2">
          {#each frameworks as item (item.name)}
            <Badge variant="outline" class="gap-1.5 {getFrameworkColor(item.name)}">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}
        </div>
      </div>
    {/if}

    {#if packageManagers.length > 0}
      <div>
        <p class="mb-2 text-sm font-medium text-muted-foreground">
          Package Managers
        </p>
        <div class="flex flex-wrap gap-2">
          {#each packageManagers as item (item.name)}
            <Badge variant="outline" class="gap-1.5">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}
        </div>
      </div>
    {/if}

    {#if languages.length > 0}
      <div>
        <p class="mb-2 text-sm font-medium text-muted-foreground">Languages</p>
        <div class="flex flex-wrap gap-2">
          {#each languages as item (item.name)}
            <Badge variant="secondary" class="gap-1.5">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{:else if showEmpty}
  <p class="text-sm text-muted-foreground">
    No frameworks, languages, or package managers assigned. Edit the project to
    add them.
  </p>
{/if}
