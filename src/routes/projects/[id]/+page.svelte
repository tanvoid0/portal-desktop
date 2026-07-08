<!--
	Project Details Page
	Displays detailed information about a specific project
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { buildTabUrl, resolveUrlTab } from "$lib/utils/url-tabs";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Edit,
    FolderOpen,
    Star,
    Calendar,
    Code,
    Package,
    Terminal,
    ExternalLink,
    ArrowLeft,
    RefreshCw,
    Check,
    X,
    HardDrive,
    FileText,
    Eye,
    GitBranch,
    Layers,
    Globe,
    Clock,
    Play,
    Loader2,
  } from "@lucide/svelte";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import { ProjectTerminal } from "$lib/domains/terminal";
  import {
    projectService,
    createProjectQuery,
    createProjectsQuery,
  } from "$lib/domains/projects";
  import { breadcrumbActions } from "$lib/domains/shared/stores/breadcrumbStore";
  import { logger } from "$lib/domains/shared/services/logger";
  import { confirmAction } from "$lib/utils/confirm";
  import type { Project } from "$lib/domains/projects/types";
  import PipelineBuilder from "$lib/domains/projects/pipelines/components/PipelineBuilder.svelte";
  import PipelineRunHistoryList from "$lib/domains/projects/pipelines/components/PipelineRunHistoryList.svelte";
  import type {
    Pipeline,
    PipelineExecutionListItem,
  } from "$lib/domains/projects/pipelines";
  import {
    pipelineService,
    executionService,
    frameworkStarterService,
    type StarterPackStatus,
  } from "$lib/domains/projects/pipelines";
  import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
  import ProjectTechStack from "$lib/domains/projects/components/ProjectTechStack.svelte";
  import {
    getProjectGitBranch,
    getProjectPackageManager,
  } from "$lib/domains/projects/utils/display";
  import {
    getProjectCommand,
    findPipelineForCommand,
    runShellCommand,
    commandKindLabel,
    type ProjectCommandKind,
  } from "$lib/domains/projects/utils/projectCommandRunner";
  import { formatRelativeTime } from "$lib/domains/shared/utils";
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    PageStats,
    type PageStat,
  } from "$lib/components/shell";
  import { toast } from "$lib/utils/toast";
  import { Sparkles } from "@lucide/svelte";

  const log = logger.createScoped("ProjectDetailsPage");

  const projectIdParam = $derived($page.params.id);
  const projectsQuery = createProjectsQuery();

  const resolvedProjectId = $derived.by((): string | null => {
    if (!projectIdParam) return null;

    const numericId = parseInt(projectIdParam, 10);
    if (!isNaN(numericId)) {
      return String(numericId);
    }

    const list = projectsQuery.data;
    if (list) {
      const projectByName = list.find((p) => p.name === projectIdParam);
      if (projectByName) {
        log.warn("Project ID was a name, redirecting to numeric ID", {
          name: projectIdParam,
          id: projectByName.id,
        });
        goto(`/projects/${projectByName.id}`, { replaceState: true });
        return String(projectByName.id);
      }
    }

    return null;
  });

  const PROJECT_TABS = [
    "overview",
    "dependencies",
    "pipelines",
    "terminal",
  ] as const;
  type ProjectTab = (typeof PROJECT_TABS)[number];

  const activeTab = $derived(
    resolveUrlTab($page.url.searchParams, PROJECT_TABS, "overview"),
  );

  function setActiveTab(tab: ProjectTab) {
    goto(buildTabUrl($page.url.pathname, $page.url.searchParams, tab), {
      replaceState: true,
      noScroll: true,
    });
  }

  const projectQuery = createProjectQuery(() => resolvedProjectId);

  // State
  let project = $state<Project | null>(null);
  let error = $state("");
  let refreshing = $state(false);

  const loading = $derived(
    projectQuery.isPending || (resolvedProjectId === null && projectsQuery.isPending),
  );

  const hasDependencies = $derived(
    Boolean(
      project?.metadata?.dependencies &&
        (Object.keys(project.metadata.dependencies.dependencies || {}).length >
          0 ||
          Object.keys(project.metadata.dependencies.devDependencies || {})
            .length > 0 ||
          project.metadata.dependencies.packageManager),
    ),
  );

  const detailStats = $derived.by((): PageStat[] => {
    if (!project) return [];

    const stats: PageStat[] = [
      {
        label: "Project Size",
        value: formatFileSize(project.size),
        icon: HardDrive,
      },
      {
        label: "Files",
        value: project.file_count,
        icon: FileText,
      },
      {
        label: "Opens",
        value: project.open_count,
        icon: Eye,
      },
    ];

    const gitBranch = getProjectGitBranch(project);
    if (gitBranch) {
      stats.push({
        label: "Git Branch",
        value: gitBranch,
        icon: GitBranch,
      });
    } else if (project.last_opened) {
      stats.push({
        label: "Last Opened",
        value: formatRelativeTime(project.last_opened),
        icon: Clock,
        description: formatDate(project.last_opened),
      });
    } else if (project.dev_port) {
      stats.push({
        label: "Dev Port",
        value: `:${project.dev_port}`,
        icon: Globe,
      });
    }

    return stats;
  });

  $effect(() => {
    if (projectQuery.data) {
      project = projectQuery.data;
      breadcrumbActions.setProjectDetailsBreadcrumbs(projectQuery.data.name);
    }
  });

  $effect(() => {
    if (projectQuery.isError) {
      error = "Failed to load project details. Please try again.";
    } else if (
      projectQuery.isSuccess &&
      projectQuery.data === null &&
      resolvedProjectId
    ) {
      error = "Project not found";
    } else if (
      resolvedProjectId === null &&
      projectIdParam &&
      projectsQuery.isFetched &&
      !projectsQuery.isPending
    ) {
      error = "Invalid project ID";
      setTimeout(() => goto("/projects"), 2000);
    } else if (projectQuery.isSuccess && projectQuery.data) {
      error = "";
    }
  });

  // Inline editing state
  let editingField = $state<string | null>(null);
  let editValues = $state<Record<string, string>>({});
  let savingField = $state<string | null>(null);

  // Pipeline state
  let pipelines = $state<Pipeline[]>([]);
  let selectedPipeline: Pipeline | null = $state(null);
  let showBuilder = $state(false);
  let pipelinesLoading = $state(false);
  let provisioningPipelines = $state(false);
  let starterPackStatus = $state<StarterPackStatus | null>(null);
  let autoProvisionAttempted = $state(false);
  let runningCommand = $state<ProjectCommandKind | null>(null);
  let runHistory = $state<PipelineExecutionListItem[]>([]);
  let runHistoryLoading = $state(false);

  // Validate and normalize project ID for action handlers
  function getValidProjectId(idParam: string | undefined): string | null {
    if (!idParam) return null;
    if (resolvedProjectId) return resolvedProjectId;

    const numericId = parseInt(idParam, 10);
    if (!isNaN(numericId)) {
      return String(numericId);
    }

    return null;
  }

  async function reloadProject() {
    await projectQuery.refetch();
  }

  // Format date for display
  function formatDate(date: Date | string | null | undefined): string {
    if (!date) return "Never";
    const d = new Date(date);
    return d.toLocaleDateString() + " " + d.toLocaleTimeString();
  }

  // Format file size
  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function toGitHubUrl(remote?: string): string | null {
    if (!remote) return null;
    const trimmed = remote.trim().replace(/\.git$/, "");
    if (trimmed.startsWith("https://github.com/")) {
      return trimmed;
    }
    if (trimmed.startsWith("git@github.com:")) {
      return `https://github.com/${trimmed.slice("git@github.com:".length)}`;
    }
    return null;
  }

  // Navigate to edit page
  function handleEdit() {
    if (project) {
      goto(`/projects/edit/${project.id}`);
    }
  }

  // Navigate back to projects list
  function handleBack() {
    goto("/projects");
  }

  // Open project in file explorer
  async function handleOpenInExplorer() {
    if (!project) return;

    try {
      await projectService.openProjectInExplorer(project.path);
    } catch (err) {
      log.error("Failed to open project in explorer", err);
    }
  }

  // Switch to terminal tab
  function switchToTerminal() {
    activeTab = "terminal";
    const projectId = getValidProjectId(projectIdParam);
    log.info("Switched to terminal tab for project", { projectId });
  }

  // Refresh project metadata
  async function handleRefreshMetadata() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project) return;

    try {
      refreshing = true;
      log.info("Refreshing project metadata", { projectId });

      // Refresh metadata in the backend
      await projectService.refreshProjectMetadata(projectId);
      await projectQuery.refetch();

      toast.success("Project metadata and commands refreshed");
      log.info("Project metadata refreshed successfully", { projectId });
    } catch (err) {
      log.error("Failed to refresh project metadata", err);
      error = "Failed to refresh project metadata. Please try again.";
    } finally {
      refreshing = false;
    }
  }

  // Toggle star status
  async function handleToggleStar() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project) return;

    try {
      const newStarredStatus = !project.starred;
      log.info("Toggling star status", {
        projectId,
        starred: newStarredStatus,
      });

      // Update project with new starred status
      // Note: This uses updateProject, but starred is not in UpdateProjectRequest type
      // We'll need to update the project locally and potentially extend the update request
      project = { ...project, starred: newStarredStatus };

      // Optionally update in backend if there's a star endpoint
      // For now, we'll update locally. If backend persistence is needed,
      // we may need to add a specific toggleStar method to the service

      log.info("Star status toggled successfully", {
        projectId,
        starred: newStarredStatus,
      });
    } catch (err) {
      log.error("Failed to toggle star status", err);
      error = "Failed to toggle star status. Please try again.";
    }
  }

  // Start inline editing
  function startEditing(field: string, currentValue: string | undefined) {
    editingField = field;
    editValues[field] = currentValue || "";
  }

  // Cancel editing
  function cancelEditing() {
    editingField = null;
    editValues = {};
  }

  // Save inline edit
  async function saveField(field: string) {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project) return;

    try {
      savingField = field;
      const newValue = editValues[field]?.trim() || undefined;

      log.info("Saving field", { projectId, field, newValue });

      // Update project via service
      const updates: Record<string, any> = { [field]: newValue };
      await projectService.updateProject(projectId, updates);
      await projectQuery.refetch();

      // Clear editing state
      editingField = null;
      editValues = {};

      log.info("Field saved successfully", { projectId, field });
    } catch (err) {
      log.error("Failed to save field", err);
      error = `Failed to save ${field}. Please try again.`;
    } finally {
      savingField = null;
    }
  }

  async function refreshStarterPackStatus() {
    if (!project) {
      starterPackStatus = null;
      return;
    }
    await projectIconRegistry.ensureLoaded();
    starterPackStatus = frameworkStarterService.getStarterPackForProject(
      project,
      pipelines,
    );
  }

  async function loadRunHistory() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId) return;

    runHistoryLoading = true;
    try {
      runHistory = await executionService.getExecutionsByProject(projectId, 25);
    } catch (err) {
      log.error("Failed to load pipeline run history", err);
    } finally {
      runHistoryLoading = false;
    }
  }

  async function loadPipelines() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId) return;
    pipelinesLoading = true;
    try {
      pipelines = await pipelineService.getPipelines(projectId);
      await refreshStarterPackStatus();
      await loadRunHistory();
    } catch (error) {
      log.error("Failed to load pipelines", error);
    } finally {
      pipelinesLoading = false;
    }
  }

  async function tryAutoProvisionPipelines() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project || autoProvisionAttempted) return;

    await projectIconRegistry.ensureLoaded();
    const status = frameworkStarterService.getStarterPackForProject(
      project,
      pipelines,
    );
    starterPackStatus = status;

    if (pipelines.length > 0 || !status || status.missingKeys.length === 0) {
      autoProvisionAttempted = true;
      return;
    }

    autoProvisionAttempted = true;
    provisioningPipelines = true;
    try {
      const created =
        await frameworkStarterService.autoProvisionStarterPipelines(
          projectId,
          project,
          pipelines,
        );
      await loadPipelines();
      if (created.length > 0) {
        toast.success(
          `Set up ${created.length} ${status.pack.displayName} pipeline(s)`,
        );
      }
    } catch (err) {
      log.error("Auto-provision starter pipelines failed", err);
      toast.error("Failed to set up default pipelines");
    } finally {
      provisioningPipelines = false;
    }
  }

  function handleCreatePipeline() {
    selectedPipeline = null;
    showBuilder = true;
  }

  function handleEditPipeline(pipeline: Pipeline) {
    selectedPipeline = pipeline;
    showBuilder = true;
  }

  async function handleDeletePipeline(pipelineId: string) {
    const confirmed = await confirmAction(
      "Are you sure you want to delete this pipeline?",
      "Delete pipeline",
    );
    if (!confirmed) return;

    try {
      await pipelineService.deletePipeline(pipelineId);
      await loadPipelines();
    } catch (error) {
      log.error("Failed to delete pipeline", error);
    }
  }

  async function handleExecutePipeline(pipelineId: string) {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId) return;

    try {
      const execution = await executionService.executePipeline({
        pipelineId,
      });
      void loadRunHistory();
      goto(`/projects/${projectId}/pipelines/run/${execution.id}`);
    } catch (error) {
      log.error("Failed to execute pipeline", error);
      toast.error("Failed to start pipeline");
    }
  }

  function handleViewRun(execution: PipelineExecutionListItem) {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId) return;
    goto(`/projects/${projectId}/pipelines/run/${execution.id}`);
  }

  async function handleSetupDefaultPipelines() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project) return;

    try {
      provisioningPipelines = true;
      const created = await frameworkStarterService.provisionStarterPipelines(
        projectId,
        project,
        pipelines,
      );
      await loadPipelines();
      if (created.length > 0) {
        toast.success(`Created ${created.length} default pipeline(s)`);
      } else {
        toast.info("Default pipelines are already set up");
      }
    } catch (error) {
      log.error("Failed to provision starter pipelines", error);
      toast.error("Failed to set up default pipelines");
    } finally {
      provisioningPipelines = false;
    }
  }

  function categoryBadgeVariant(
    category: string | undefined,
  ): "default" | "secondary" | "outline" {
    switch (category) {
      case "install":
        return "secondary";
      case "dev":
        return "default";
      case "build":
        return "outline";
      default:
        return "outline";
    }
  }

  function runButtonLabel(pipeline: Pipeline): string {
    return pipeline.category === "dev" ? "Start" : "Run";
  }

  function handleBuilderClose() {
    showBuilder = false;
    selectedPipeline = null;
    loadPipelines();
  }

  async function handleRunCommand(kind: ProjectCommandKind) {
    if (!project || runningCommand) return;

    const command = getProjectCommand(project, kind);
    if (!command) {
      toast.error(`No ${commandKindLabel(kind).toLowerCase()} command configured`);
      return;
    }

    const pipeline = findPipelineForCommand(pipelines, kind);
    if (pipeline) {
      await handleExecutePipeline(pipeline.id);
      return;
    }

    if (kind === "start") {
      activeTab = "terminal";
      toast.info(`No dev pipeline found. Run in terminal: ${command}`);
      return;
    }

    runningCommand = kind;
    try {
      const result = await runShellCommand(project.path, command);
      if (result.success) {
        toast.success(`${commandKindLabel(kind)} completed`);
        if (result.output.trim()) {
          log.info(`${commandKindLabel(kind)} output`, { output: result.output });
        }
      } else {
        toast.error(`${commandKindLabel(kind)} failed`, result.output);
      }
    } catch (err) {
      log.error(`Failed to run ${kind} command`, err);
      toast.error(`Failed to run ${commandKindLabel(kind).toLowerCase()} command`);
    } finally {
      runningCommand = null;
    }
  }

  function hasAnyCommands(proj: Project): boolean {
    return Boolean(
      proj.build_command ||
        proj.start_command ||
        proj.test_command ||
        proj.output_directory ||
        proj.dev_port ||
        proj.prod_port,
    );
  }

  $effect(() => {
    resolvedProjectId;
    autoProvisionAttempted = false;
  });

  $effect(() => {
    if (
      activeTab === "pipelines" &&
      project &&
      !pipelinesLoading &&
      !provisioningPipelines
    ) {
      void tryAutoProvisionPipelines();
    }
  });

  onMount(() => {
    void projectIconRegistry.ensureLoaded();
    const tabParam = $page.url.searchParams.get("tab");
    if (
      tabParam &&
      ["overview", "dependencies", "pipelines", "terminal"].includes(tabParam)
    ) {
      activeTab = tabParam;
    }
    loadPipelines();
  });
</script>

<svelte:head>
  <title
    >{project ? `${project.name} - Project Details` : "Project Details"} - Portal
    Desktop</title
  >
</svelte:head>

<div class="space-y-6">
  <Button
    variant="ghost"
    size="sm"
    onclick={handleBack}
    class="w-fit gap-2"
  >
    <ArrowLeft class="h-4 w-4" />
    Back to Projects
  </Button>

  {#if loading}
    <PageLoading message="Loading project details..." />
  {:else if error}
    <PageError
      title="Failed to load project"
      message={error}
      onRetry={reloadProject}
    />
  {:else if project}
    {@const currentProject = project}
    {@const githubUrl = toGitHubUrl(currentProject.git_repository)}
    <PageHeader
      title={currentProject.name}
      description={currentProject.description}
      badge={currentProject.status}
    >
      {#snippet actions()}
        <Button
          variant={currentProject.starred ? "default" : "outline"}
          onclick={handleToggleStar}
          class="gap-2"
          title={currentProject.starred ? "Unstar project" : "Star project"}
        >
          <Star class="h-4 w-4 {currentProject.starred ? 'fill-current' : ''}" />
          {currentProject.starred ? "Starred" : "Star"}
        </Button>
        <Button
          variant="outline"
          onclick={handleRefreshMetadata}
          disabled={refreshing}
          class="gap-2"
          title="Refresh project metadata, commands, size, and git info"
        >
          <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
          Refresh
        </Button>
        <Button
          variant="outline"
          onclick={handleOpenInExplorer}
          class="gap-2"
        >
          <ExternalLink class="h-4 w-4" />
          Explorer
        </Button>
        {#if githubUrl}
          <a href={githubUrl} target="_blank" rel="noreferrer">
            <Button variant="outline" class="gap-2">
              <ExternalLink class="h-4 w-4" />
              GitHub
            </Button>
          </a>
        {/if}
        <Button variant="outline" onclick={switchToTerminal} class="gap-2">
          <Terminal class="h-4 w-4" />
          Terminal
        </Button>
        <Button onclick={handleEdit} class="gap-2">
          <Edit class="h-4 w-4" />
          Edit
        </Button>
      {/snippet}
    </PageHeader>

    {#if currentProject.starred}
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <Star class="h-4 w-4 fill-yellow-400 text-yellow-400" />
        Starred project
      </div>
    {/if}

    <PageStats stats={detailStats} columns={4} />

    <Tabs
      value={activeTab}
      onValueChange={(v) => setActiveTab(v as ProjectTab)}
      class="w-full"
    >
      <TabsList class="grid w-full grid-cols-4">
        <TabsTrigger value="overview" class="gap-2">
          <FolderOpen class="h-4 w-4" />
          Overview
        </TabsTrigger>
        <TabsTrigger value="dependencies" class="gap-2">
          <Package class="h-4 w-4" />
          Dependencies
        </TabsTrigger>
        <TabsTrigger value="pipelines" class="gap-2">
          <Code class="h-4 w-4" />
          Pipelines
        </TabsTrigger>
        <TabsTrigger value="terminal" class="gap-2">
          <Terminal class="h-4 w-4" />
          Terminal
        </TabsTrigger>
      </TabsList>

        <TabsContent value="overview" class="mt-6 space-y-6">
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Layers class="h-5 w-5" />
                Tech Stack
              </CardTitle>
              <CardDescription>
                Frameworks, languages, and package managers assigned to this
                project
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ProjectTechStack {project} />
            </CardContent>
          </Card>

          <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <FolderOpen class="h-5 w-5" />
                  Project Details
                </CardTitle>
              </CardHeader>
              <CardContent class="space-y-4">
                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    Project Path
                  </p>
                  <p
                    class="mt-1 break-all rounded-md bg-muted p-2 font-mono text-sm"
                  >
                    {project.path}
                  </p>
                </div>

                {#if getProjectPackageManager(project)}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Detected Package Manager
                    </p>
                    <p class="mt-1 text-sm">
                      {getProjectPackageManager(project)}
                    </p>
                  </div>
                {/if}

                {#if project.created_at}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Created
                    </p>
                    <p class="mt-1 text-sm">
                      {formatDate(project.created_at)}
                    </p>
                  </div>
                {/if}

                {#if project.updated_at}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Last Updated
                    </p>
                    <p class="mt-1 text-sm">
                      {formatDate(project.updated_at)}
                    </p>
                  </div>
                {/if}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <Calendar class="h-5 w-5" />
                  Activity
                </CardTitle>
              </CardHeader>
              <CardContent class="space-y-4">
                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    Open Count
                  </p>
                  <p class="mt-1 text-sm">{project.open_count} times</p>
                </div>

                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    Project Size
                  </p>
                  <p class="mt-1 text-sm">{formatFileSize(project.size)}</p>
                </div>

                <div>
                  <p class="text-sm font-medium text-muted-foreground">
                    File Count
                  </p>
                  <p class="mt-1 text-sm">{project.file_count} files</p>
                </div>

                {#if project.last_opened}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Last Opened
                    </p>
                    <p class="mt-1 text-sm">
                      {formatDate(project.last_opened)}
                    </p>
                  </div>
                {/if}
              </CardContent>
            </Card>
          </div>

          <!-- Commands & Configuration -->
          <Card>
            <CardHeader>
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <CardTitle class="flex items-center gap-2">
                    <Terminal class="h-5 w-5" />
                    Commands & Configuration
                  </CardTitle>
                  <CardDescription class="mt-1">
                    Scripts detected from your project. Run via matching pipelines
                    when available, or directly for build/test.
                  </CardDescription>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onclick={handleRefreshMetadata}
                  disabled={refreshing}
                  class="gap-2"
                >
                  <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
                  Re-detect
                </Button>
              </div>
            </CardHeader>
            <CardContent class="space-y-4">
              {#if !hasAnyCommands(project) && editingField === null}
                <div class="rounded-md border border-dashed p-6 text-center">
                  <p class="mb-3 text-sm text-muted-foreground">
                    No commands configured yet. Re-detect from your project folder
                    or add them manually.
                  </p>
                  <div class="flex flex-wrap justify-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={handleRefreshMetadata}
                      disabled={refreshing}
                      class="gap-2"
                    >
                      <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
                      Detect from project
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onclick={() => startEditing("build_command", "")}
                    >
                      Add manually
                    </Button>
                  </div>
                </div>
              {/if}

              {#if project.build_command || editingField === "build_command"}
                <div>
                  <div class="mb-1 flex items-center justify-between gap-2">
                    <Label class="text-sm font-medium text-muted-foreground"
                      >Build Command</Label
                    >
                    {#if editingField !== "build_command"}
                      <div class="flex items-center gap-1">
                        {#if project.build_command}
                          <Button
                            variant="outline"
                            size="sm"
                            onclick={() => handleRunCommand("build")}
                            disabled={runningCommand !== null}
                            class="h-7 gap-1 px-2"
                          >
                            {#if runningCommand === "build"}
                              <Loader2 class="h-3 w-3 animate-spin" />
                            {:else}
                              <Play class="h-3 w-3" />
                            {/if}
                            Run
                          </Button>
                        {/if}
                        <Button
                          variant="ghost"
                          size="sm"
                          onclick={() =>
                            project &&
                            startEditing("build_command", project.build_command)}
                          class="h-7 px-2"
                        >
                          <Edit class="h-3 w-3" />
                        </Button>
                      </div>
                    {/if}
                  </div>
                  {#if editingField === "build_command"}
                    <div class="flex items-center gap-2">
                      <Input
                        bind:value={editValues.build_command}
                        class="font-mono text-sm"
                        placeholder="npm run build"
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => saveField("build_command")}
                        disabled={savingField === "build_command"}
                        class="h-8"
                      >
                        <Check class="h-4 w-4 text-green-600" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={cancelEditing}
                        disabled={savingField === "build_command"}
                        class="h-8"
                      >
                        <X class="h-4 w-4 text-red-600" />
                      </Button>
                    </div>
                  {:else}
                    <p class="rounded bg-muted p-2 font-mono text-sm">
                      {project.build_command}
                    </p>
                  {/if}
                </div>
              {/if}

              {#if project.start_command || editingField === "start_command"}
                <div>
                  <div class="mb-1 flex items-center justify-between gap-2">
                    <Label class="text-sm font-medium text-muted-foreground"
                      >Start Command</Label
                    >
                    {#if editingField !== "start_command"}
                      <div class="flex items-center gap-1">
                        {#if project.start_command}
                          <Button
                            variant="outline"
                            size="sm"
                            onclick={() => handleRunCommand("start")}
                            disabled={runningCommand !== null}
                            class="h-7 gap-1 px-2"
                          >
                            {#if runningCommand === "start"}
                              <Loader2 class="h-3 w-3 animate-spin" />
                            {:else}
                              <Play class="h-3 w-3" />
                            {/if}
                            Run
                          </Button>
                        {/if}
                        <Button
                          variant="ghost"
                          size="sm"
                          onclick={() =>
                            project &&
                            startEditing("start_command", project.start_command)}
                          class="h-7 px-2"
                        >
                          <Edit class="h-3 w-3" />
                        </Button>
                      </div>
                    {/if}
                  </div>
                  {#if editingField === "start_command"}
                    <div class="flex items-center gap-2">
                      <Input
                        bind:value={editValues.start_command}
                        class="font-mono text-sm"
                        placeholder="npm start"
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => saveField("start_command")}
                        disabled={savingField === "start_command"}
                        class="h-8"
                      >
                        <Check class="h-4 w-4 text-green-600" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={cancelEditing}
                        disabled={savingField === "start_command"}
                        class="h-8"
                      >
                        <X class="h-4 w-4 text-red-600" />
                      </Button>
                    </div>
                  {:else}
                    <p class="rounded bg-muted p-2 font-mono text-sm">
                      {project.start_command}
                    </p>
                  {/if}
                </div>
              {/if}

              {#if project.test_command || editingField === "test_command"}
                <div>
                  <div class="mb-1 flex items-center justify-between gap-2">
                    <Label class="text-sm font-medium text-muted-foreground"
                      >Test Command</Label
                    >
                    {#if editingField !== "test_command"}
                      <div class="flex items-center gap-1">
                        {#if project.test_command}
                          <Button
                            variant="outline"
                            size="sm"
                            onclick={() => handleRunCommand("test")}
                            disabled={runningCommand !== null}
                            class="h-7 gap-1 px-2"
                          >
                            {#if runningCommand === "test"}
                              <Loader2 class="h-3 w-3 animate-spin" />
                            {:else}
                              <Play class="h-3 w-3" />
                            {/if}
                            Run
                          </Button>
                        {/if}
                        <Button
                          variant="ghost"
                          size="sm"
                          onclick={() =>
                            project &&
                            startEditing("test_command", project.test_command)}
                          class="h-7 px-2"
                        >
                          <Edit class="h-3 w-3" />
                        </Button>
                      </div>
                    {/if}
                  </div>
                  {#if editingField === "test_command"}
                    <div class="flex items-center gap-2">
                      <Input
                        bind:value={editValues.test_command}
                        class="font-mono text-sm"
                        placeholder="npm test"
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => saveField("test_command")}
                        disabled={savingField === "test_command"}
                        class="h-8"
                      >
                        <Check class="h-4 w-4 text-green-600" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={cancelEditing}
                        disabled={savingField === "test_command"}
                        class="h-8"
                      >
                        <X class="h-4 w-4 text-red-600" />
                      </Button>
                    </div>
                  {:else}
                    <p class="rounded bg-muted p-2 font-mono text-sm">
                      {project.test_command}
                    </p>
                  {/if}
                </div>
              {/if}

              {#if project.output_directory || editingField === "output_directory"}
                <div>
                  <div class="mb-1 flex items-center justify-between">
                    <Label class="text-sm font-medium text-muted-foreground"
                      >Output Directory</Label
                    >
                    {#if editingField !== "output_directory"}
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() =>
                          project &&
                          startEditing(
                            "output_directory",
                            project.output_directory,
                          )}
                        class="h-6 px-2"
                      >
                        <Edit class="h-3 w-3" />
                      </Button>
                    {/if}
                  </div>
                  {#if editingField === "output_directory"}
                    <div class="flex items-center gap-2">
                      <Input
                        bind:value={editValues.output_directory}
                        class="font-mono text-sm"
                        placeholder="dist"
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => saveField("output_directory")}
                        disabled={savingField === "output_directory"}
                        class="h-8"
                      >
                        <Check class="h-4 w-4 text-green-600" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={cancelEditing}
                        disabled={savingField === "output_directory"}
                        class="h-8"
                      >
                        <X class="h-4 w-4 text-red-600" />
                      </Button>
                    </div>
                  {:else}
                    <p class="rounded bg-muted p-2 font-mono text-sm">
                      {project.output_directory}
                    </p>
                  {/if}
                </div>
              {/if}

              <!-- Add New Command Section -->
              {#if hasAnyCommands(project) && editingField === null}
                <div class="border-t pt-2">
                  <div class="flex flex-wrap gap-2">
                    {#if !project.build_command && editingField !== "build_command"}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => startEditing("build_command", "")}
                      >
                        + Build
                      </Button>
                    {/if}
                    {#if !project.start_command && editingField !== "start_command"}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => startEditing("start_command", "")}
                      >
                        + Start
                      </Button>
                    {/if}
                    {#if !project.test_command && editingField !== "test_command"}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => startEditing("test_command", "")}
                      >
                        + Test
                      </Button>
                    {/if}
                    {#if !project.output_directory && editingField !== "output_directory"}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => startEditing("output_directory", "")}
                      >
                        + Output Dir
                      </Button>
                    {/if}
                  </div>
                </div>
              {/if}

              {#if project.dev_port || project.prod_port}
                <div>
                  <p class="text-sm font-medium text-muted-foreground">Ports</p>
                  <div class="mt-1 flex gap-2">
                    {#if project.dev_port}
                      <Badge variant="outline">Dev: {project.dev_port}</Badge>
                    {/if}
                    {#if project.prod_port}
                      <Badge variant="outline">Prod: {project.prod_port}</Badge>
                    {/if}
                  </div>
                </div>
              {/if}
            </CardContent>
          </Card>

          <!-- Git Information -->
          {#if project.git_repository || project.git_branch}
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <Code class="h-5 w-5" />
                  Git Information
                </CardTitle>
              </CardHeader>
              <CardContent class="space-y-4">
                {#if project.git_repository}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Repository
                    </p>
                    <p
                      class="mt-1 break-all rounded bg-muted p-2 font-mono text-sm"
                    >
                      {project.git_repository}
                    </p>
                    {#if toGitHubUrl(project.git_repository)}
                      <a
                        href={toGitHubUrl(project.git_repository) ?? "#"}
                        target="_blank"
                        rel="noreferrer"
                        class="mt-2 inline-flex items-center text-sm text-primary"
                      >
                        Open repository
                        <ExternalLink class="ml-1 h-3 w-3" />
                      </a>
                    {/if}
                  </div>
                {/if}

                {#if project.git_branch}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Branch
                    </p>
                    <p class="mt-1 text-sm">{project.git_branch}</p>
                  </div>
                {/if}

                {#if project.git_commit}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Last Commit
                    </p>
                    <p class="mt-1 rounded bg-muted p-2 font-mono text-sm">
                      {project.git_commit}
                    </p>
                  </div>
                {/if}

                {#if project.has_uncommitted_changes !== undefined}
                  <div>
                    <p class="text-sm font-medium text-muted-foreground">
                      Uncommitted Changes
                    </p>
                    <Badge
                      variant={project.has_uncommitted_changes
                        ? "destructive"
                        : "default"}
                    >
                      {project.has_uncommitted_changes ? "Yes" : "No"}
                    </Badge>
                  </div>
                {/if}
              </CardContent>
            </Card>
          {/if}
        </TabsContent>

        <TabsContent value="dependencies" class="mt-6 space-y-6">
          {#if hasDependencies && project.metadata?.dependencies}
            <div class="space-y-6">
              <!-- Package Manager Info -->
              <Card>
                <CardHeader>
                  <CardTitle class="flex items-center gap-2">
                    <Package class="h-5 w-5" />
                    Package Manager
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <Badge variant="outline" class="px-3 py-1 text-base">
                    {project.metadata.dependencies.packageManager}
                  </Badge>
                </CardContent>
              </Card>

              <!-- Dependencies -->
              {#if Object.keys(project.metadata.dependencies.dependencies || {}).length > 0}
                <Card>
                  <CardHeader>
                    <CardTitle>Dependencies</CardTitle>
                    <CardDescription>
                      {Object.keys(project.metadata.dependencies.dependencies)
                        .length} production dependencies
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div class="max-h-[400px] space-y-2 overflow-y-auto">
                      {#each Object.entries(project.metadata.dependencies.dependencies) as [name, version]}
                        <div
                          class="flex items-center justify-between rounded border p-2 {project.metadata.dependencies.outdated?.includes(
                            name,
                          )
                            ? 'border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20'
                            : ''}"
                        >
                          <div class="flex items-center gap-2">
                            <Package class="h-4 w-4 text-muted-foreground" />
                            <span class="font-mono text-sm font-medium"
                              >{name}</span
                            >
                          </div>
                          <div class="flex items-center gap-2">
                            <span class="text-sm text-muted-foreground"
                              >{version}</span
                            >
                            {#if project.metadata.dependencies.outdated?.includes(name)}
                              <Badge
                                variant="outline"
                                class="border-yellow-300 bg-yellow-100 text-yellow-800 dark:border-yellow-700 dark:bg-yellow-900 dark:text-yellow-200"
                              >
                                Outdated
                              </Badge>
                            {/if}
                            {#if project.metadata.dependencies.vulnerabilities?.includes(name)}
                              <Badge variant="destructive">Vulnerable</Badge>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </CardContent>
                </Card>
              {/if}

              <!-- Dev Dependencies -->
              {#if Object.keys(project.metadata.dependencies.devDependencies || {}).length > 0}
                <Card>
                  <CardHeader>
                    <CardTitle>Dev Dependencies</CardTitle>
                    <CardDescription>
                      {Object.keys(
                        project.metadata.dependencies.devDependencies,
                      ).length} development dependencies
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div class="max-h-[400px] space-y-2 overflow-y-auto">
                      {#each Object.entries(project.metadata.dependencies.devDependencies) as [name, version]}
                        <div
                          class="flex items-center justify-between rounded border p-2 {project.metadata.dependencies.outdated?.includes(
                            name,
                          )
                            ? 'border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20'
                            : ''}"
                        >
                          <div class="flex items-center gap-2">
                            <Package class="h-4 w-4 text-muted-foreground" />
                            <span class="font-mono text-sm font-medium"
                              >{name}</span
                            >
                          </div>
                          <div class="flex items-center gap-2">
                            <span class="text-sm text-muted-foreground"
                              >{version}</span
                            >
                            {#if project.metadata.dependencies.outdated?.includes(name)}
                              <Badge
                                variant="outline"
                                class="border-yellow-300 bg-yellow-100 text-yellow-800 dark:border-yellow-700 dark:bg-yellow-900 dark:text-yellow-200"
                              >
                                Outdated
                              </Badge>
                            {/if}
                            {#if project.metadata.dependencies.vulnerabilities?.includes(name)}
                              <Badge variant="destructive">Vulnerable</Badge>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </CardContent>
                </Card>
              {/if}

              <!-- Summary -->
              {#if (project.metadata.dependencies.outdated?.length || 0) > 0 || (project.metadata.dependencies.vulnerabilities?.length || 0) > 0}
                <Card class="border-orange-200 dark:border-orange-800">
                  <CardHeader>
                    <CardTitle>Summary</CardTitle>
                  </CardHeader>
                  <CardContent class="space-y-2">
                    {#if (project.metadata.dependencies.outdated?.length || 0) > 0}
                      <div class="flex items-center gap-2">
                        <Badge
                          variant="outline"
                          class="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
                        >
                          {project.metadata.dependencies.outdated.length} outdated
                          packages
                        </Badge>
                      </div>
                    {/if}
                    {#if (project.metadata.dependencies.vulnerabilities?.length || 0) > 0}
                      <div class="flex items-center gap-2">
                        <Badge variant="destructive">
                          {project.metadata.dependencies.vulnerabilities.length} vulnerable
                          packages
                        </Badge>
                      </div>
                    {/if}
                  </CardContent>
                </Card>
              {/if}
            </div>
          {:else}
            <PageEmpty
              title="No dependency data"
              description="Dependency information has not been scanned for this project yet. Refresh metadata to update size and git info, or edit the project to assign package managers."
              icon={Package}
              actionLabel="Refresh metadata"
              onAction={handleRefreshMetadata}
            />
          {/if}
        </TabsContent>

        <TabsContent value="pipelines" class="mt-6 space-y-6">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <h2 class="text-xl font-semibold">Pipelines</h2>
              <div class="flex flex-wrap gap-2">
                {#if starterPackStatus && starterPackStatus.missingKeys.length > 0}
                  <Button
                    variant="default"
                    disabled={provisioningPipelines}
                    onclick={handleSetupDefaultPipelines}
                    class="gap-2"
                  >
                    <Sparkles class="h-4 w-4" />
                    {provisioningPipelines
                      ? "Setting up..."
                      : `Setup default pipelines (${starterPackStatus.pack.displayName})`}
                  </Button>
                {/if}
                <Button
                  variant="outline"
                  onclick={() => {
                    const projectId = getValidProjectId(projectIdParam);
                    if (projectId) goto(`/projects/${projectId}/pipelines/new`);
                  }}
                >
                  Create from Template
                </Button>
                <Button onclick={handleCreatePipeline}>Create Pipeline</Button>
              </div>
            </div>

            {#if showBuilder}
              <PipelineBuilder
                pipeline={selectedPipeline || undefined}
                projectId={getValidProjectId(projectIdParam) || ""}
                onSave={handleBuilderClose}
                onCancel={handleBuilderClose}
              />
            {:else if pipelinesLoading || provisioningPipelines}
              <PageLoading
                message={provisioningPipelines
                  ? `Setting up ${starterPackStatus?.pack.displayName ?? "default"} pipelines…`
                  : "Loading pipelines..."}
              />
            {:else if pipelines.length === 0}
              <PageEmpty
                title="No pipelines yet"
                description={starterPackStatus
                  ? `Set up Install, Dev, and Build pipelines for ${starterPackStatus.pack.displayName}`
                  : "Create a pipeline to automate install, dev, build, and deploy workflows for this project."}
                icon={Code}
                actionLabel={starterPackStatus
                  ? "Setup default pipelines"
                  : "Create pipeline"}
                onAction={starterPackStatus
                  ? handleSetupDefaultPipelines
                  : handleCreatePipeline}
              />
            {:else}
              <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {#each pipelines as pipeline (pipeline.id)}
                  <Card>
                    <CardHeader>
                      <div class="flex items-start justify-between gap-2">
                        <CardTitle class="text-base">{pipeline.name}</CardTitle>
                        {#if pipeline.category}
                          <Badge variant={categoryBadgeVariant(pipeline.category)}>
                            {pipeline.category}
                          </Badge>
                        {/if}
                      </div>
                      {#if pipeline.description}
                        <CardDescription>{pipeline.description}</CardDescription
                        >
                      {/if}
                    </CardHeader>
                    <CardContent class="space-y-2">
                      <p class="text-sm text-muted-foreground">
                        {pipeline.steps.length} step{pipeline.steps.length !== 1
                          ? "s"
                          : ""}
                      </p>
                      <div class="flex flex-wrap gap-2">
                        <Button
                          size="sm"
                          onclick={() => handleExecutePipeline(pipeline.id)}
                          disabled={!pipeline.enabled}
                        >
                          {runButtonLabel(pipeline)}
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          onclick={() => handleEditPipeline(pipeline)}
                        >
                          Edit
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onclick={() => handleDeletePipeline(pipeline.id)}
                        >
                          Delete
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                {/each}
              </div>
            {/if}

            {#if !showBuilder && !pipelinesLoading && !provisioningPipelines}
              <PipelineRunHistoryList
                executions={runHistory}
                loading={runHistoryLoading}
                title="Run history"
                description="Recent pipeline runs for this project"
                showPipeline
                onRefresh={loadRunHistory}
                onSelect={handleViewRun}
              />
            {/if}
        </TabsContent>

        <TabsContent value="terminal" class="mt-6">
          <Card class="overflow-hidden">
            <CardContent class="p-0">
              <div class="h-[600px]">
                <ProjectTerminal
                  projectId={project.id}
                  projectName={project.name}
                  projectPath={project.path}
                  settings={{
                    theme: "dark",
                    fontSize: 14,
                    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                    cursorStyle: "block",
                    scrollbackLines: 1000,
                    bellSound: false,
                    autoClose: true,
                    confirmClose: true,
                    defaultShell: navigator.userAgent.includes("Windows")
                      ? "cmd.exe"
                      : "bash",
                    workingDirectory: project.path,
                  }}
                />
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
  {/if}
</div>
