/**
 * State, queries and handlers for the project details route. Construct it
 * during component init — it creates queries and registers $effects/onMount.
 */
import { onMount } from 'svelte';
import { page } from '$app/state';
import { goto } from '$app/navigation';
import { buildTabUrl, resolveUrlTab } from '$lib/utils/url-tabs';
import { projectService, createProjectQuery, createProjectsQuery } from '$lib/domains/projects';
import { createProjectModulesQuery } from '$lib/domains/projects/queries/moduleQueries';
import { refreshProjectModules } from '$lib/domains/projects/api/moduleApi';
import {
  openProjectInIde,
  setProjectPushOk,
  toggleProjectStar,
} from '$lib/domains/projects/api/projectApi';
import { queryClient, queryKeys } from '$lib/domains/shared/query';
import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
import { logger } from '$lib/domains/shared/services/logger';
import type { Project } from '$lib/domains/projects/types';
import { readableDescription } from '$lib/domains/projects/utils/display';
import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
import {
  getProjectCommand,
  commandKindLabel,
  type ProjectCommandKind,
} from '$lib/domains/projects/utils/projectCommandRunner';
import { toast } from '$lib/utils/toast';
import { deploymentActions } from '$lib/domains/deployments/stores/deploymentStore';
import { loadDeployConfig, buildCreateRequest } from '$lib/domains/deployments/utils/deployConfig';
import { actions } from '$lib/domains/actions';
import { createProjectMovesQuery } from '$lib/domains/projects/queries/projectMoveQueries';
import { findActiveMove } from '$lib/domains/projects/api/projectMoveApi';
import { createQuery } from '@tanstack/svelte-query';
import { agentSessionsQuery } from '$lib/domains/agents/queries/agentsQueries';
import { groupSessionsByProject } from '$lib/domains/agents/utils/groupSessions';

const log = logger.createScoped('ProjectDetailsPage');

const PROJECT_TABS = ['overview', 'apps', 'actions', 'env', 'backup', 'terminal'] as const;
export type ProjectTab = (typeof PROJECT_TABS)[number];

export class ProjectDetailState {
  projectIdParam = $derived(page.params.id);
  projectsQuery = createProjectsQuery();

  resolvedProjectId = $derived.by((): string | null => {
    const param = this.projectIdParam;
    if (!param) return null;

    const numericId = parseInt(param, 10);
    if (!isNaN(numericId)) {
      return String(numericId);
    }

    const list = this.projectsQuery.data;
    if (list) {
      const projectByName = list.find((p) => p.name === param);
      if (projectByName) {
        log.warn('Project ID was a name, redirecting to numeric ID', {
          name: param,
          id: projectByName.id,
        });
        goto(`/projects/${projectByName.id}`, { replaceState: true });
        return String(projectByName.id);
      }
    }

    return null;
  });

  activeTab = $derived(
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

  projectQuery = createProjectQuery(() => this.resolvedProjectId);

  // State
  project = $state<Project | null>(null);
  error = $state('');
  refreshing = $state(false);
  runningCommand = $state<ProjectCommandKind | null>(null);
  isDeploying = $state(false);
  rescanningModules = $state(false);
  // Below md the facts rail is a Sheet, opened from the header's "Details" button.
  factsOpen = $state(false);

  // The one line on this page a human typed — shown in the rail, where it can wrap.
  about = $derived(readableDescription(this.project?.description));

  // isLoading (not isPending): a disabled query — unresolved/invalid id — is pending forever.
  loading = $derived(
    this.projectQuery.isLoading || (this.resolvedProjectId === null && this.projectsQuery.isPending)
  );

  modulesQuery = createProjectModulesQuery(() =>
    this.resolvedProjectId ? Number(this.resolvedProjectId) : null
  );
  movesQuery = createProjectMovesQuery();
  isMoving = $derived(
    !!findActiveMove(
      this.movesQuery.data,
      this.resolvedProjectId ? Number(this.resolvedProjectId) : -1
    )
  );
  modules = $derived(this.modulesQuery.data ?? []);
  hasModules = $derived(this.modules.length > 0);

  // Agent sessions for this project's workspace — reuses the same cwd→project
  // matching the Agents sidebar groups by, scoped to just this one project.
  agentSessions = createQuery(agentSessionsQuery);
  agentSessionsForProject = $derived.by(() => {
    if (!this.project) return [];
    const [group] = groupSessionsByProject(this.agentSessions.data ?? [], [this.project]);
    return group?.projectId === this.project.id ? group.sessions.slice(0, 5) : [];
  });
  // An 'apps' deep link shows Overview until (and unless) the project has modules.
  displayTab = $derived(
    this.activeTab === 'apps' && !this.hasModules ? 'overview' : this.activeTab
  );

  constructor() {
    $effect(() => {
      const q = this.projectQuery;
      if (q.data) {
        this.project = q.data;
        breadcrumbActions.setProjectDetailsBreadcrumbs(q.data.name);
      }
    });

    $effect(() => {
      const q = this.projectQuery;
      if (q.isError) {
        this.error = 'Failed to load project details. Please try again.';
      } else if (q.isSuccess && q.data === null && this.resolvedProjectId) {
        this.error = 'Project not found';
      } else if (
        this.resolvedProjectId === null &&
        this.projectIdParam &&
        this.projectsQuery.isFetched &&
        !this.projectsQuery.isPending
      ) {
        this.error = 'Invalid project ID';
        setTimeout(() => goto('/projects'), 2000);
      } else if (q.isSuccess && q.data) {
        this.error = '';
      }
    });

    onMount(() => {
      void projectIconRegistry.ensureLoaded();
      const tabParam = page.url.searchParams.get('tab');
      // Only rewrite legacy aliases — avoid goto when already on a valid tab
      if (tabParam === 'pipelines' || tabParam === 'ci') {
        this.setActiveTab('actions');
      }
    });
  }

  setActiveTab = (tab: ProjectTab) => {
    if (tab === this.activeTab) return;
    goto(buildTabUrl(page.url.pathname, page.url.searchParams, tab), {
      replaceState: true,
      reset: false,
    });
  };

  handleRescanModules = async () => {
    if (!this.resolvedProjectId || this.rescanningModules) return;
    this.rescanningModules = true;
    try {
      await refreshProjectModules(Number(this.resolvedProjectId));
      // The scan can also fill an empty stack, so refetch the project too.
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.projects.modules }),
        queryClient.invalidateQueries({ queryKey: queryKeys.projects.all }),
      ]);
    } catch (err) {
      log.error('Failed to rescan modules', err);
      toast.error('Failed to rescan modules');
    } finally {
      this.rescanningModules = false;
    }
  };

  // Validate and normalize project ID for action handlers
  getValidProjectId(idParam: string | undefined): string | null {
    if (!idParam) return null;
    if (this.resolvedProjectId) return this.resolvedProjectId;

    const numericId = parseInt(idParam, 10);
    if (!isNaN(numericId)) {
      return String(numericId);
    }

    return null;
  }

  reloadProject = async () => {
    await this.projectQuery.refetch();
  };

  handleEdit = () => {
    if (this.project) {
      goto(`/projects/edit/${this.project.id}`);
    }
  };

  // Deploy this project. First time (no remembered config): open the New
  // Deployment form prefilled from the project, so the user picks a .env file
  // and settings once. After that: one click builds + runs with the remembered
  // config — no form.
  handleDeploy = async () => {
    const project = this.project;
    if (!project || this.isDeploying) return;

    const saved = loadDeployConfig(project.path);
    if (!saved) {
      const params = new URLSearchParams({
        projectPath: project.path,
        name: project.name,
      });
      goto(`/deployments/new?${params.toString()}`);
      return;
    }

    this.isDeploying = true;
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
      this.isDeploying = false;
    }
  };

  handleOpenInEditor = async () => {
    if (!this.project) return;
    try {
      const editor = await openProjectInIde(Number(this.project.id));
      toast.success(`Opened in ${editor}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(message, 'Set a default editor in Settings');
    }
  };

  handleOpenInExplorer = async () => {
    if (!this.project) return;

    try {
      await projectService.openProjectInExplorer(this.project.path);
    } catch (err) {
      log.error('Failed to open project in explorer', err);
    }
  };

  switchToTerminal = () => {
    this.setActiveTab('terminal');
  };

  // Re-detect stack, commands and git info from the project folder
  handleRedetect = async () => {
    const projectId = this.getValidProjectId(this.projectIdParam);
    if (!projectId || !this.project) return;

    try {
      this.refreshing = true;
      await projectService.refreshProjectMetadata(projectId);
      // Detection may add framework rows; the list caches the stack too.
      await projectIconRegistry.reload();
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
      toast.success('Stack and commands re-detected');
    } catch (err) {
      log.error('Failed to re-detect project', err);
      toast.error('Could not re-detect the project');
    } finally {
      this.refreshing = false;
    }
  };

  handleSetPushOk = async (pushOk: boolean) => {
    if (!this.project) return;
    try {
      await setProjectPushOk(Number(this.project.id), pushOk);
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (err) {
      log.error('Failed to set push permission', err);
      toast.error('Could not update push permission');
    }
  };

  handleToggleStar = async () => {
    if (!this.project) return;
    try {
      await toggleProjectStar(Number(this.project.id));
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (err) {
      log.error('Failed to toggle star', err);
      toast.error('Could not update the star');
    }
  };

  handleRunCommand = async (kind: ProjectCommandKind) => {
    const project = this.project;
    if (!project || this.runningCommand) return;

    const actionId = kind === 'build' ? 'build' : kind === 'start' ? 'dev' : 'test';

    this.runningCommand = kind;
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
        this.setActiveTab('terminal');
        toast.info(`Run in terminal: ${command}`);
      } else {
        log.error(`Failed to run ${kind} command`, err);
        toast.error(`Failed to run ${commandKindLabel(kind).toLowerCase()} command`);
      }
    } finally {
      this.runningCommand = null;
    }
  };
}
