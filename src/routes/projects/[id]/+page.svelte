<!--
  Project Details Page
  Run-first: how to run it, what it's built with, and its git state.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Star from '~icons/fa6-solid/star';
  import Terminal from '~icons/fa6-solid/terminal';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import Info from '~icons/fa6-solid/circle-info';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import Layers from '~icons/fa6-solid/layer-group';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Workflow from '~icons/fa6-solid/diagram-project';
  import Rocket from '~icons/fa6-solid/rocket';
  import Code from '~icons/fa6-solid/code';
  import FileKey from '~icons/fa6-solid/key';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { ProjectTerminal } from '$lib/domains/terminal';
  import { projectService, createProjectQuery, createProjectsQuery } from '$lib/domains/projects';
  import { createProjectModulesQuery } from '$lib/domains/projects/queries/moduleQueries';
  import { refreshProjectModules } from '$lib/domains/projects/api/moduleApi';
  import { openProjectInIde, toggleProjectStar } from '$lib/domains/projects/api/projectApi';
  import ProjectSummary from '$lib/domains/projects/components/ProjectSummary.svelte';
  import ProjectModules from '$lib/domains/projects/components/ProjectModules.svelte';
  import ProjectCommands from '$lib/domains/projects/components/ProjectCommands.svelte';
  import ProjectTechStack from '$lib/domains/projects/components/ProjectTechStack.svelte';
  import ProjectGitCard from '$lib/domains/projects/components/ProjectGitCard.svelte';
  import ProjectEnv from '$lib/domains/projects/components/ProjectEnv.svelte';
  import { queryClient, queryKeys } from '$lib/domains/shared/query';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import { logger } from '$lib/domains/shared/services/logger';
  import type { Project } from '$lib/domains/projects/types';
  import { readableDescription } from '$lib/domains/projects/utils/display';
  import ProjectActionsPanel from '$lib/domains/actions/components/ProjectActionsPanel.svelte';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import {
    getProjectCommand,
    commandKindLabel,
    type ProjectCommandKind,
  } from '$lib/domains/projects/utils/projectCommandRunner';
  import { PageHeader, PageLoading, PageError, ResponsivePanel } from '$lib/components/shell';
  import { toast } from '$lib/utils/toast';
  import { deploymentActions } from '$lib/domains/deployments/stores/deploymentStore';
  import {
    loadDeployConfig,
    buildCreateRequest,
  } from '$lib/domains/deployments/utils/deployConfig';
  import { actions } from '$lib/domains/actions';
  import { Badge } from '$lib/components/ui/badge';
  import { createProjectMovesQuery } from '$lib/domains/projects/queries/projectMoveQueries';
  import { findActiveMove } from '$lib/domains/projects/api/projectMoveApi';
  import { createQuery } from '@tanstack/svelte-query';
  import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
  import { groupSessionsByProject } from '$lib/domains/agents/utils/groupSessions';
  import { openAgentsSessionById } from '$lib/domains/agents/utils/launch';
  import { launchInAgents } from '$lib/domains/projects/utils/launch';
  import Bolt from '~icons/fa6-solid/bolt';

  const log = logger.createScoped('ProjectDetailsPage');

  const projectIdParam = $derived(page.params.id);
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
        log.warn('Project ID was a name, redirecting to numeric ID', {
          name: projectIdParam,
          id: projectByName.id,
        });
        goto(`/projects/${projectByName.id}`, { replaceState: true });
        return String(projectByName.id);
      }
    }

    return null;
  });

  const PROJECT_TABS = ['overview', 'apps', 'actions', 'env', 'terminal'] as const;
  type ProjectTab = (typeof PROJECT_TABS)[number];

  const activeTab = $derived(
    resolveUrlTab(
      (() => {
        // Legacy tab aliases
        const params = new URLSearchParams(page.url.search);
        const tab = params.get('tab');
        if (tab === 'pipelines' || tab === 'ci') {
          params.set('tab', 'actions');
        }
        return params;
      })(),
      PROJECT_TABS,
      'overview'
    )
  );

  function setActiveTab(tab: ProjectTab) {
    if (tab === activeTab) return;
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  }

  const projectQuery = createProjectQuery(() => resolvedProjectId);

  // State
  let project = $state<Project | null>(null);
  let error = $state('');
  let refreshing = $state(false);
  let runningCommand = $state<ProjectCommandKind | null>(null);
  let isDeploying = $state(false);
  let rescanningModules = $state(false);
  // Below md the facts rail is a Sheet, opened from the header's "Details" button.
  let factsOpen = $state(false);

  // The one line on this page a human typed — shown in the rail, where it can wrap.
  const about = $derived(readableDescription(project?.description));

  // isLoading (not isPending): a disabled query — unresolved/invalid id — is pending forever.
  const loading = $derived(
    projectQuery.isLoading || (resolvedProjectId === null && projectsQuery.isPending)
  );

  const modulesQuery = createProjectModulesQuery(() =>
    resolvedProjectId ? Number(resolvedProjectId) : null
  );
  const movesQuery = createProjectMovesQuery();
  const isMoving = $derived(
    !!findActiveMove(movesQuery.data, resolvedProjectId ? Number(resolvedProjectId) : -1)
  );
  const modules = $derived(modulesQuery.data ?? []);
  const hasModules = $derived(modules.length > 0);

  // Agent sessions for this project's workspace — reuses the same cwd→project
  // matching the Agents sidebar groups by, scoped to just this one project.
  const agentSessions = createQuery(agentSessionsQuery);
  const agentSessionsForProject = $derived.by(() => {
    if (!project) return [];
    const [group] = groupSessionsByProject(agentSessions.data ?? [], [project]);
    return group?.projectId === project.id ? group.sessions.slice(0, 5) : [];
  });
  // An 'apps' deep link shows Overview until (and unless) the project has modules.
  const displayTab = $derived(activeTab === 'apps' && !hasModules ? 'overview' : activeTab);

  async function handleRescanModules() {
    if (!resolvedProjectId || rescanningModules) return;
    rescanningModules = true;
    try {
      await refreshProjectModules(Number(resolvedProjectId));
      // The scan can also fill an empty stack, so refetch the project too.
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.projects.modules }),
        queryClient.invalidateQueries({ queryKey: queryKeys.projects.all }),
      ]);
    } catch (err) {
      log.error('Failed to rescan modules', err);
      toast.error('Failed to rescan modules');
    } finally {
      rescanningModules = false;
    }
  }

  $effect(() => {
    if (projectQuery.data) {
      project = projectQuery.data;
      breadcrumbActions.setProjectDetailsBreadcrumbs(projectQuery.data.name);
    }
  });

  $effect(() => {
    if (projectQuery.isError) {
      error = 'Failed to load project details. Please try again.';
    } else if (projectQuery.isSuccess && projectQuery.data === null && resolvedProjectId) {
      error = 'Project not found';
    } else if (
      resolvedProjectId === null &&
      projectIdParam &&
      projectsQuery.isFetched &&
      !projectsQuery.isPending
    ) {
      error = 'Invalid project ID';
      setTimeout(() => goto('/projects'), 2000);
    } else if (projectQuery.isSuccess && projectQuery.data) {
      error = '';
    }
  });

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

  function toGitHubUrl(remote?: string): string | null {
    if (!remote) return null;
    const trimmed = remote.trim().replace(/\.git$/, '');
    if (trimmed.startsWith('https://github.com/')) {
      return trimmed;
    }
    if (trimmed.startsWith('git@github.com:')) {
      return `https://github.com/${trimmed.slice('git@github.com:'.length)}`;
    }
    return null;
  }

  const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
    ['year', 31536000],
    ['month', 2592000],
    ['day', 86400],
    ['hour', 3600],
    ['minute', 60],
  ];

  /** "3 days ago" style text for the footer line. */
  function timeAgo(date: Date | string): string {
    const seconds = (new Date(date).getTime() - Date.now()) / 1000;
    const fmt = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
    for (const [unit, size] of RELATIVE_UNITS) {
      if (Math.abs(seconds) >= size) return fmt.format(Math.round(seconds / size), unit);
    }
    return 'just now';
  }

  function handleEdit() {
    if (project) {
      goto(`/projects/edit/${project.id}`);
    }
  }

  // Deploy this project. First time (no remembered config): open the New
  // Deployment form prefilled from the project, so the user picks a .env file
  // and settings once. After that: one click builds + runs with the remembered
  // config — no form.
  async function handleDeploy() {
    if (!project || isDeploying) return;

    const saved = loadDeployConfig(project.path);
    if (!saved) {
      const params = new URLSearchParams({
        projectPath: project.path,
        name: project.name,
      });
      goto(`/deployments/new?${params.toString()}`);
      return;
    }

    isDeploying = true;
    toast.info('Building & starting deployment…');
    try {
      const request = buildCreateRequest(project.name, project.path, saved);
      await deploymentActions.deployAndStart(request);
      toast.success('Deployment built and started');
      goto('/deployments');
    } catch (error) {
      logger.error('One-click deploy failed', { context: 'ProjectDetailsPage', error });
      const raw =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : 'Deploy failed';
      toast.error(raw.length > 300 ? `${raw.slice(0, 300)}…` : raw);
    } finally {
      isDeploying = false;
    }
  }

  async function handleOpenInEditor() {
    if (!project) return;
    try {
      const editor = await openProjectInIde(Number(project.id));
      toast.success(`Opened in ${editor}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(message, 'Set a default editor in Settings');
    }
  }

  async function handleOpenInExplorer() {
    if (!project) return;

    try {
      await projectService.openProjectInExplorer(project.path);
    } catch (err) {
      log.error('Failed to open project in explorer', err);
    }
  }

  function switchToTerminal() {
    setActiveTab('terminal');
  }

  // Re-detect stack, commands and git info from the project folder
  async function handleRedetect() {
    const projectId = getValidProjectId(projectIdParam);
    if (!projectId || !project) return;

    try {
      refreshing = true;
      await projectService.refreshProjectMetadata(projectId);
      // Detection may add framework rows; the list caches the stack too.
      await projectIconRegistry.reload();
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
      toast.success('Stack and commands re-detected');
    } catch (err) {
      log.error('Failed to re-detect project', err);
      toast.error('Could not re-detect the project');
    } finally {
      refreshing = false;
    }
  }

  async function handleToggleStar() {
    if (!project) return;
    try {
      await toggleProjectStar(Number(project.id));
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (err) {
      log.error('Failed to toggle star', err);
      toast.error('Could not update the star');
    }
  }

  async function handleRunCommand(kind: ProjectCommandKind) {
    if (!project || runningCommand) return;

    const actionId = kind === 'build' ? 'build' : kind === 'start' ? 'dev' : 'test';

    runningCommand = kind;
    try {
      const result = await actions.forProject(project).run(actionId);
      if (result.success) {
        toast.success(`${commandKindLabel(kind)} completed`);
      } else {
        const err = result.steps.find((s) => s.error)?.error ?? `${commandKindLabel(kind)} failed`;
        toast.error(err);
      }
    } catch (err) {
      // Fallback when the actions catalog cannot run it: hand the command to
      // the terminal rather than a second, unscoped shell path.
      const command = getProjectCommand(project, kind);
      if (command) {
        setActiveTab('terminal');
        toast.info(`Run in terminal: ${command}`);
      } else {
        log.error(`Failed to run ${kind} command`, err);
        toast.error(`Failed to run ${commandKindLabel(kind).toLowerCase()} command`);
      }
    } finally {
      runningCommand = null;
    }
  }

  onMount(() => {
    void projectIconRegistry.ensureLoaded();
    const tabParam = page.url.searchParams.get('tab');
    // Only rewrite legacy aliases — avoid goto when already on a valid tab
    if (tabParam === 'pipelines' || tabParam === 'ci') {
      setActiveTab('actions');
    }
  });
</script>

<svelte:head>
  <title>{project ? `${project.name} - Project Details` : 'Project Details'} - Portal Desktop</title
  >
</svelte:head>

<div class="space-y-6">
  {#if loading}
    <PageLoading message="Loading project details..." />
  {:else if error}
    <PageError title="Failed to load project" message={error} onRetry={reloadProject} />
  {:else if project}
    {@const currentProject = project}
    {@const githubUrl = toGitHubUrl(currentProject.git_repository)}
    <PageHeader title={currentProject.name} badge={currentProject.status}>
      {#snippet subtitle()}
        {#if isMoving}
          <Badge
            variant="outline"
            class="mr-2 shrink-0 font-normal text-amber-600 dark:text-amber-400"
          >
            Moving…
          </Badge>
        {/if}
        <ProjectSummary project={currentProject} />
      {/snippet}
      {#snippet actions()}
        <Button
          variant="outline"
          size="icon"
          onclick={() => (factsOpen = true)}
          class="md:hidden"
          aria-label="Project details"
        >
          <Info class="h-4 w-4" />
        </Button>
        <Button onclick={handleOpenInEditor} class="gap-2">
          <Code class="h-4 w-4" />
          Open in editor
        </Button>
        <Button variant="outline" onclick={handleOpenInExplorer} class="gap-2">
          <FolderOpen class="h-4 w-4" />
          Open folder
        </Button>
        <Button variant="outline" onclick={switchToTerminal} class="gap-2">
          <Terminal class="h-4 w-4" />
          Terminal
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger
            class={buttonVariants({ variant: 'outline' })}
            aria-label="More actions"
          >
            <Ellipsis class="h-4 w-4" />
            More
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={handleEdit}>
              <Edit class="h-4 w-4" />
              Edit
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={handleDeploy} disabled={isDeploying}>
              {#if isDeploying}
                <Loader2 class="h-4 w-4 animate-spin" />
                Deploying…
              {:else}
                <Rocket class="h-4 w-4" />
                Deploy
              {/if}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={handleToggleStar}>
              <Star class="h-4 w-4 {currentProject.starred ? 'fill-current' : ''}" />
              {currentProject.starred ? 'Unstar' : 'Star'}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={handleRedetect} disabled={refreshing}>
              <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
              Re-detect stack
            </DropdownMenuItem>
            {#if githubUrl}
              <DropdownMenuItem>
                {#snippet child({ props })}
                  <a {...props} href={githubUrl} target="_blank" rel="noreferrer">
                    <ExternalLink class="h-4 w-4" />
                    GitHub
                  </a>
                {/snippet}
              </DropdownMenuItem>
            {/if}
          </DropdownMenuContent>
        </DropdownMenu>
      {/snippet}
    </PageHeader>

    <div class="flex items-start gap-6">
      <div class="min-w-0 flex-1">
        <Tabs
          value={displayTab}
          onValueChange={(v) => setActiveTab(v as ProjectTab)}
          class="w-full"
        >
          <TabsList>
            <TabsTrigger value="overview" class="gap-2">
              <FolderOpen class="h-4 w-4" />
              Overview
            </TabsTrigger>
            {#if hasModules}
              <TabsTrigger value="apps" class="gap-2">
                <Layers class="h-4 w-4" />
                Apps ({modules.length})
              </TabsTrigger>
            {/if}
            <TabsTrigger value="actions" class="gap-2">
              <Workflow class="h-4 w-4" />
              Actions
            </TabsTrigger>
            <TabsTrigger value="env" class="gap-2">
              <FileKey class="h-4 w-4" />
              Env
            </TabsTrigger>
            <TabsTrigger value="terminal" class="gap-2">
              <Terminal class="h-4 w-4" />
              Terminal
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" class="mt-6 space-y-6">
            <ProjectCommands
              project={currentProject}
              {runningCommand}
              {refreshing}
              onRun={handleRunCommand}
              onRedetect={handleRedetect}
              onSaved={reloadProject}
            />
            {#if currentProject.last_opened}
              <p class="text-xs text-muted-foreground">
                Last opened {timeAgo(currentProject.last_opened)}
              </p>
            {/if}
          </TabsContent>

          {#if hasModules}
            <TabsContent value="apps" class="mt-6">
              <ProjectModules
                {modules}
                rescanning={rescanningModules}
                onRescan={handleRescanModules}
              />
            </TabsContent>
          {/if}

          <TabsContent value="actions" class="mt-6 space-y-6">
            <ProjectActionsPanel project={currentProject} enabled={activeTab === 'actions'} />
          </TabsContent>

          <TabsContent value="env" class="mt-6">
            <ProjectEnv projectId={Number(currentProject.id)} enabled={activeTab === 'env'} />
          </TabsContent>

          <TabsContent value="terminal" class="mt-6">
            <Card class="overflow-hidden">
              <CardContent class="p-0">
                <div class="h-[600px]">
                  <ProjectTerminal
                    projectId={currentProject.id}
                    projectName={currentProject.name}
                    projectPath={currentProject.path}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      <!-- Reference facts stay on screen while you work in Actions or Terminal. -->
      <ResponsivePanel bind:open={factsOpen} side="right" desktopClass="w-80" title="Details">
        <div class="space-y-4 overflow-y-auto p-4">
          {#if about}
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <Info class="h-5 w-5" />
                  About
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p class="text-sm text-muted-foreground">{about}</p>
              </CardContent>
            </Card>
          {/if}
          <ProjectTechStack
            project={currentProject}
            {modules}
            {refreshing}
            onRedetect={handleRedetect}
          />
          <ProjectGitCard
            project={currentProject}
            {githubUrl}
            {refreshing}
            onRedetect={handleRedetect}
          />
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center gap-2">
                <Bolt class="h-5 w-5" />
                Agent sessions
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-1.5">
              {#if agentSessionsForProject.length === 0}
                <p class="text-sm text-muted-foreground">No runs yet.</p>
              {:else}
                {#each agentSessionsForProject as session (session.id)}
                  <Button
                    variant="link"
                    size="sm"
                    class="h-auto w-full justify-start truncate p-0 text-left font-normal"
                    onclick={() => openAgentsSessionById(session.id)}
                  >
                    {session.title || session.backend_id}
                  </Button>
                {/each}
              {/if}
              <Button
                variant="outline"
                size="sm"
                class="mt-2 gap-2"
                onclick={() => launchInAgents(currentProject)}
              >
                <Bolt class="h-3.5 w-3.5" />
                New run
              </Button>
            </CardContent>
          </Card>
        </div>
      </ResponsivePanel>
    </div>
  {/if}
</div>
