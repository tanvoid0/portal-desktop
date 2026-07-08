<!--
	Projects page - Project management interface
	Showcases the projects domain with full CRUD operations
-->

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { FolderOpen, CheckCircle, Database, Code } from "@lucide/svelte";
  import ProjectCard from "$lib/domains/projects/components/ProjectCard.svelte";
  import Select from "$lib/components/ui/select.svelte";
  import { projectService } from "$lib/domains/projects";
  import { createProjectsQuery } from "$lib/domains/projects/queries/projectQueries";
  import type { Project } from "$lib/domains/projects/types";
  import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
  import { logger } from "$lib/domains/shared/services/logger";
  import { confirmAction } from "$lib/utils/confirm";
  import { formatBytes } from "$lib/domains/shared/utils";
  import { getProjectFramework } from "$lib/domains/projects/utils/display";
  import { breadcrumbActions } from "$lib/domains/shared/stores/breadcrumbStore";
  import { WorkflowTrigger, WorkflowResults } from "$lib/domains/automation";
  import type { WorkflowResult } from "$lib/domains/automation/types";
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
  } from "$lib/components/shell";
  import { Plus } from "@lucide/svelte";

  const log = logger.createScoped("ProjectsPage");

  const projectsQuery = createProjectsQuery();

  let searchQuery = $state("");
  let selectedProject = $state<any>(null);
  let showAutomation = $state(false);
  let workflowResult = $state<WorkflowResult | null>(null);

  // Filter state
  let filterFramework = $state<string | null>(null);
  let filterStatus = $state<string | null>(null);
  let filterPackageManager = $state<string | null>(null);

  // Sort state
  type SortOption = "name" | "last_opened" | "size" | "created_at";
  let sortBy = $state<SortOption>("name");
  let sortDirection = $state<"asc" | "desc">("asc");
  let registryReady = $state(false);

  const projects = $derived(projectsQuery.data ?? []);
  const isLoading = $derived(projectsQuery.isPending);
  const pageError = $derived(
    projectsQuery.isError ? "Failed to load projects" : null,
  );

  $effect(() => {
    if (projectsQuery.isSuccess) {
      void projectIconRegistry.ensureLoaded().then(() => {
        registryReady = true;
      });
      breadcrumbActions.setProjectBreadcrumbs();
    }
  });

  function projectHasFramework(project: Project, framework: string): boolean {
    return projectIconRegistry
      .resolveFrameworks(project)
      .some((item) => item.name === framework);
  }

  function projectHasPackageManager(
    project: Project,
    packageManager: string,
  ): boolean {
    return projectIconRegistry
      .resolvePackageManagers(project)
      .some((item) => item.name === packageManager);
  }

  const filteredProjects = $derived(
    [...projects]
      .filter((project) => {
        if (searchQuery.trim()) {
          const query = searchQuery.toLowerCase();
          if (
            !project.name.toLowerCase().includes(query) &&
            !project.description?.toLowerCase().includes(query) &&
            !project.path.toLowerCase().includes(query)
          ) {
            return false;
          }
        }
        if (filterFramework && !projectHasFramework(project, filterFramework)) {
          return false;
        }
        if (filterStatus && project.status !== filterStatus) {
          return false;
        }
        if (
          filterPackageManager &&
          !projectHasPackageManager(project, filterPackageManager)
        ) {
          return false;
        }
        return true;
      })
      .sort((a, b) => {
        let comparison = 0;

        switch (sortBy) {
          case "name":
            comparison = a.name.localeCompare(b.name);
            break;
          case "last_opened":
            comparison =
              (a.last_opened ? new Date(a.last_opened).getTime() : 0) -
              (b.last_opened ? new Date(b.last_opened).getTime() : 0);
            break;
          case "size":
            comparison = a.size - b.size;
            break;
          case "created_at":
            comparison =
              (a.created_at ? new Date(a.created_at).getTime() : 0) -
              (b.created_at ? new Date(b.created_at).getTime() : 0);
            break;
        }

        return sortDirection === "asc" ? comparison : -comparison;
      }),
  );

  const uniqueFrameworks = $derived.by(() => {
    if (!registryReady) return [];

    const names = new Set<string>();
    for (const project of projects) {
      for (const framework of projectIconRegistry.resolveFrameworks(project)) {
        names.add(framework.name);
      }
    }
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  const uniquePackageManagers = $derived.by(() => {
    if (!registryReady) return [];

    const names = new Set<string>();
    for (const project of projects) {
      for (const manager of projectIconRegistry.resolvePackageManagers(project)) {
        names.add(manager.name);
      }
    }
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  // Handle project click - navigate to details page
  const handleProjectClick = (project: any) => {
    goto(`/projects/${project.id}`);
  };

  // Handle automation
  function handleAutomationClick(project: any, event: Event) {
    event.stopPropagation();
    selectedProject = project;
    showAutomation = true;
    workflowResult = null;
  }

  function handleWorkflowComplete(result: WorkflowResult) {
    workflowResult = result;
    log.info("Workflow completed", { result });
  }

  function closeAutomation() {
    showAutomation = false;
    selectedProject = null;
    workflowResult = null;
  }

  // Handle project delete
  const handleProjectDelete = async (project: any) => {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${project.name}"?`,
    );
    if (confirmed) {
      try {
        await projectService.deleteProject(project.id);
        log.info("Project deleted", { id: project.id, name: project.name });
      } catch (error) {
        log.error("Failed to delete project", error);
      }
    }
  };
</script>

<svelte:head>
  <title>Projects - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <PageHeader title="Projects" description="Manage your development projects">
    {#snippet actions()}
    <Button onclick={() => goto("/projects/create")}>
      <Plus class="mr-2 h-4 w-4" />
      New Project
    </Button>
    {/snippet}
  </PageHeader>

  <!-- Stats Cards -->
  <div class="grid grid-cols-1 gap-4 md:grid-cols-4">
    <Card>
      <CardHeader class="pb-1 px-4">
        <CardTitle class="flex items-center gap-2 text-sm font-medium">
          <FolderOpen class="h-4 w-4 text-muted-foreground" />
          Total Projects
        </CardTitle>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <div class="text-xl font-bold leading-none">{projects.length}</div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-1 px-4">
        <CardTitle class="flex items-center gap-2 text-sm font-medium">
          <CheckCircle class="h-4 w-4 text-muted-foreground" />
          Active Projects
        </CardTitle>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <div class="text-xl font-bold leading-none">
          {projects.filter((p) => p.status === "active").length}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-1 px-4">
        <CardTitle class="flex items-center gap-2 text-sm font-medium">
          <Database class="h-4 w-4 text-muted-foreground" />
          Total Size
        </CardTitle>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <div class="text-xl font-bold leading-none">
          {formatBytes(
            projects.reduce((sum, p) => sum + p.size, 0),
          )}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader class="pb-1 px-4">
        <CardTitle class="flex items-center gap-2 text-sm font-medium">
          <Code class="h-4 w-4 text-muted-foreground" />
          Most Used Type
        </CardTitle>
      </CardHeader>
      <CardContent class="px-4 py-3">
        <div class="text-xl font-bold leading-none">
          {Object.entries(
            projects.reduce(
              (acc, p) => {
                const framework = getProjectFramework(p) || "Unknown";
                acc[framework] = (acc[framework] || 0) + 1;
                return acc;
              },
              {} as Record<string, number>,
            ),
          ).sort(([, a], [, b]) => (b as number) - (a as number))[0]?.[0] ||
            "none"}
        </div>
      </CardContent>
    </Card>
  </div>

  <!-- Filters -->
  <div class="flex flex-wrap items-center gap-4">
    <Input
      placeholder="Search projects..."
      bind:value={searchQuery}
      class="max-w-sm"
    />

    <Select
      defaultValue={filterFramework || ""}
      placeholder="All Frameworks"
      options={uniqueFrameworks.map((f) => ({ value: f, label: f }))}
      onSelect={(value) => (filterFramework = value || null)}
      class="min-w-[150px]"
    />

    <Select
      defaultValue={filterStatus || ""}
      placeholder="All Statuses"
      options={[
        { value: "active", label: "Active" },
        { value: "archived", label: "Archived" },
        { value: "deleted", label: "Deleted" },
      ]}
      onSelect={(value) => (filterStatus = value || null)}
      class="min-w-[130px]"
    />

    <Select
      defaultValue={filterPackageManager || ""}
      placeholder="All Package Managers"
      options={uniquePackageManagers.map((m) => ({ value: m, label: m }))}
      onSelect={(value) => (filterPackageManager = value || null)}
      class="min-w-[180px]"
    />

    <Select
      defaultValue={sortBy}
      placeholder="Sort by"
      options={[
        { value: "name", label: "Name" },
        { value: "last_opened", label: "Last Opened" },
        { value: "size", label: "Size" },
        { value: "created_at", label: "Created Date" },
      ]}
      onSelect={(value) => (sortBy = value as SortOption)}
      class="min-w-[140px]"
    />

    <Button
      variant="outline"
      size="sm"
      onclick={() => (sortDirection = sortDirection === "asc" ? "desc" : "asc")}
      class="flex items-center gap-1"
      title={`Sort ${sortDirection === "asc" ? "Ascending" : "Descending"}`}
    >
      <svg
        class="h-4 w-4 {sortDirection === 'desc' ? 'rotate-180' : ''}"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M5 15l7-7 7 7"
        />
      </svg>
      {sortDirection === "asc" ? "Asc" : "Desc"}
    </Button>

    {#if filterFramework || filterStatus || filterPackageManager}
      <Button
        variant="ghost"
        size="sm"
        onclick={() => {
          filterFramework = null;
          filterStatus = null;
          filterPackageManager = null;
        }}
        class="text-muted-foreground"
      >
        Clear Filters
      </Button>
    {/if}
  </div>

  <!-- Projects Grid -->
  {#if isLoading}
    <PageLoading message="Loading projects..." />
  {:else if pageError}
    <PageError
      title="Failed to Load Projects"
      message={pageError}
      onRetry={() => projectsQuery.refetch()}
    />
  {:else if filteredProjects.length === 0}
    <PageEmpty
      title="No projects found"
      description="Create your first project to get started"
      filteredDescription="Try adjusting your search criteria"
      isFiltered={Boolean(searchQuery.trim() || filterFramework || filterStatus || filterPackageManager)}
      icon={FolderOpen}
      actionLabel="Create Project"
      onAction={() => goto("/projects/create")}
    />
  {:else}
    <div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredProjects as project (project.id)}
        <ProjectCard
          {project}
          onClick={(p) => handleProjectClick(p)}
          onEdit={(p) => goto(`/projects/${p.id}`)}
          onDelete={handleProjectDelete}
          showActions={true}
        />
      {/each}
    </div>
  {/if}

  <!-- Automation Modal -->
  <Dialog.Root
    open={showAutomation && !!selectedProject}
    onOpenChange={(isOpen) => {
      if (!isOpen) closeAutomation();
    }}
  >
    <Dialog.Content class="max-h-[90vh] max-w-2xl overflow-y-auto">
      <Dialog.Header>
        <Dialog.Title>Automate: {selectedProject?.name}</Dialog.Title>
      </Dialog.Header>

      {#if workflowResult}
        <WorkflowResults result={workflowResult} />
      {:else if selectedProject}
        <WorkflowTrigger
          project={selectedProject}
          onWorkflowComplete={handleWorkflowComplete}
        />
      {/if}
    </Dialog.Content>
  </Dialog.Root>
</div>
