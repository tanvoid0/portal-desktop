<!--
	ProjectList component for displaying a list of projects
	Uses Svelte 5 runes and Tailwind CSS
-->

<script lang="ts">
  import type { Project } from "$lib/domains/projects/types";
  import { confirmAction } from "$lib/utils/confirm";
  import ProjectCard from "./ProjectCard.svelte";

  interface Props {
    projects: Project[];
    searchQuery?: string;
    onProjectClick?: (project: Project) => void;
    onProjectEdit?: (project: Project) => void;
    onProjectDelete?: (project: Project) => void;
    showActions?: boolean;
    emptyMessage?: string;
  }

  let {
    projects = $bindable(),
    searchQuery = "",
    onProjectClick = () => {},
    onProjectEdit = () => {},
    onProjectDelete = () => {},
    showActions = true,
    emptyMessage = "No projects found",
  }: Props = $props();

  // Filter projects based on type and search query
  const filteredProjects = $derived(() => {
    let filtered = projects;

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (project) =>
          project.name.toLowerCase().includes(query) ||
          project.description?.toLowerCase().includes(query) ||
          project.path.toLowerCase().includes(query),
      );
    }

    return filtered;
  });

  const handleProjectDelete = async (project: Project) => {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${project.name}"?`,
    );
    if (confirmed) {
      onProjectDelete(project);
    }
  };
</script>

<div class="space-y-4">
  {#if filteredProjects().length === 0}
    <div class="py-12 text-center">
      <div class="mb-2 text-neutral-400 dark:text-neutral-500">
        <svg
          class="mx-auto h-12 w-12"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
          />
        </svg>
      </div>
      <h3
        class="mb-1 text-lg font-medium text-neutral-900 dark:text-neutral-100"
      >
        {emptyMessage}
      </h3>
      {#if searchQuery}
        <p class="text-sm text-neutral-500 dark:text-neutral-400">
          Try adjusting your search terms
        </p>
      {/if}
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredProjects() as project (project.id)}
        <ProjectCard
          {project}
          onClick={onProjectClick}
          onEdit={onProjectEdit}
          onDelete={handleProjectDelete}
          {showActions}
        />
      {/each}
    </div>
  {/if}
</div>
