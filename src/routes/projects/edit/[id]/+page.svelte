<!--
	Project edit page - Uses unified ProjectForm component
	Allows editing existing projects with auto-detection capabilities
-->

<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import ProjectForm from "$lib/components/projects/ProjectForm.svelte";
  import {
    projectService,
    createProjectQuery,
  } from "$lib/domains/projects";
  import { logger } from "$lib/domains/shared/services/logger";
  import type { CreateProjectRequest } from "$lib/domains/projects/types";
  import { breadcrumbActions } from "$lib/domains/shared/stores/breadcrumbStore";
  import { PageHeader, PageLoading, PageError } from "$lib/components/shell";

  const log = logger.createScoped("ProjectEditPage");

  const projectId = $derived($page.params.id);
  const projectQuery = createProjectQuery(() => projectId);

  let isLoading = $state(false);

  const project = $derived(projectQuery.data ?? null);
  const loading = $derived(projectQuery.isPending);
  const loadError = $derived(
    projectQuery.isError
      ? "Failed to load project"
      : projectQuery.isSuccess && !projectQuery.data
        ? "Project not found"
        : null,
  );

  const initialData = $derived<Partial<CreateProjectRequest>>(
    project
      ? {
          name: project.name,
          description: project.description,
          path: project.path,
          framework_ids: project.framework_ids,
          package_manager_ids: project.package_manager_ids,
          language_ids: project.language_ids,
          build_command: project.build_command,
          start_command: project.start_command,
          test_command: project.test_command,
          output_directory: project.output_directory,
          dev_port: project.dev_port,
          prod_port: project.prod_port,
        }
      : {},
  );

  $effect(() => {
    if (project) {
      breadcrumbActions.setProjectEditBreadcrumbs(project.name);
    }
  });

  async function handleSubmit(data: CreateProjectRequest) {
    if (!projectId) {
      log.error("Project ID is required for update");
      return;
    }

    try {
      isLoading = true;
      await projectService.updateProject(projectId, data);
      goto("/projects");
    } catch (err) {
      log.error("Failed to update project", err);
      throw err;
    } finally {
      isLoading = false;
    }
  }

  function handleCancel() {
    goto("/projects");
  }

  function reloadProject() {
    void projectQuery.refetch();
  }
</script>

<div class="container mx-auto space-y-6 px-4 py-8">
  <PageHeader
    title="Edit Project"
    description="Update your project details. You can change the path to re-analyze the project structure."
  />

  {#if loading}
    <PageLoading message="Loading project..." />
  {:else if loadError}
    <PageError
      title="Failed to load project"
      message={loadError}
      onRetry={reloadProject}
    />
  {:else if project && projectId}
    <ProjectForm
      projectId={parseInt(projectId)}
      {initialData}
      onSubmit={handleSubmit}
      onCancel={handleCancel}
      {isLoading}
    />
  {/if}
</div>
