<!--
	Projects page - compact rows grouped by area
-->

<script module lang="ts">
  // Rescanning every project is slow, so it runs once per app session in the background.
  let modulesRefreshed = false;
</script>

<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Activity from '~icons/fa6-solid/wave-square';
  import AlertTriangle from '~icons/fa6-solid/triangle-exclamation';
  import Star from '~icons/fa6-solid/star';
  import Archive from '~icons/fa6-solid/box-archive';
  import Plus from '~icons/fa6-solid/plus';
  import ScanSearch from '~icons/fa6-solid/magnifying-glass';
  import ProjectRow from '$lib/domains/projects/components/ProjectRow.svelte';
  import ProjectCard from '$lib/domains/projects/components/ProjectCard.svelte';
  import ProjectsFilterBar from '$lib/domains/projects/components/ProjectsFilterBar.svelte';
  import ProjectGroupList from '$lib/domains/projects/components/ProjectGroupList.svelte';
  import {
    readOpenGroups,
    saveOpenGroups,
    readViewMode,
    saveViewMode,
    type ViewMode,
  } from '$lib/domains/projects/utils/viewPrefs';
  import { projectService } from '$lib/domains/projects';
  import { createProjectsQuery } from '$lib/domains/projects/queries/projectQueries';
  import { createAllModulesQuery } from '$lib/domains/projects/queries/moduleQueries';
  import { refreshAllModules } from '$lib/domains/projects/api/moduleApi';
  import { groupByProject } from '$lib/domains/projects/utils/moduleStacks';
  import { nestProjects, descendantIds } from '$lib/domains/projects/utils/nestProjects';
  import { groupByArea, STARRED_KEY } from '$lib/domains/projects/utils/groupProjects';
  import type { ProjectGroup } from '$lib/domains/projects/utils/groupProjects';
  import { queryClient } from '$lib/domains/shared/query';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import type { Project, ProjectStage } from '$lib/domains/projects/types';

  import {
    toggleProjectStar,
    setProjectStage,
    setProjectArchived,
  } from '$lib/domains/projects/api/projectApi';
  import { toast } from '$lib/utils/toast';
  import { projectIconRegistry } from '$lib/domains/projects/utils/iconRegistry';
  import { logger } from '$lib/domains/shared/services/logger';
  import { confirmAction } from '$lib/utils/confirm';
  import { breadcrumbActions } from '$lib/domains/shared/stores/breadcrumbStore';
  import { PageHeader, PageLoading, PageError, PageEmpty, PageStats } from '$lib/components/shell';
  import type { PageStat } from '$lib/components/shell';

  const log = logger.createScoped('ProjectsPage');

  const projectsQuery = createProjectsQuery();
  const modulesQuery = createAllModulesQuery();

  let searchQuery = $state('');
  let filterStack = $state<string | null>(null);
  let filterActivity = $state<string | null>(null);
  let filterStage = $state<string | null>(null);
  let starredOnly = $state(false);
  let showArchived = $state(false);
  let sortBy = $state<'recent' | 'name'>('recent');
  let registryReady = $state(false);

  const projects = $derived(projectsQuery.data ?? []);
  const modulesByProject = $derived(groupByProject(modulesQuery.data ?? []));
  const isLoading = $derived(projectsQuery.isPending);
  const pageError = $derived(projectsQuery.isError ? 'Failed to load projects' : null);

  $effect(() => {
    if (projectsQuery.isSuccess) {
      void projectIconRegistry.ensureLoaded().then(() => {
        registryReady = true;
      });
      breadcrumbActions.setProjectBreadcrumbs();
    }
  });

  $effect(() => {
    if (modulesRefreshed) return;
    modulesRefreshed = true;
    refreshAllModules()
      // The scan also fills the stack of projects that had none (and may add new
      // framework rows), so reload the icon registry and refetch both.
      .then(() => projectIconRegistry.reload())
      .then(() =>
        Promise.all([
          queryClient.invalidateQueries({ queryKey: queryKeys.projects.modules }),
          queryClient.invalidateQueries({ queryKey: queryKeys.projects.all }),
        ])
      )
      .catch((error) => log.error('Failed to refresh modules', error));
  });

  const stackNames = (project: Project): string[] =>
    [
      ...projectIconRegistry.resolveFrameworks(project),
      ...projectIconRegistry.resolveLanguages(project),
    ].map((item) => item.name);

  const lastSeen = (p: Project): number => {
    const d = p.last_activity ?? p.last_opened;
    return d ? new Date(d).getTime() : 0;
  };

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
        if (filterStack && !stackNames(project).includes(filterStack)) return false;
        if (filterActivity && project.activity_status !== filterActivity) return false;
        if (filterStage && (project.stage ?? 'none') !== filterStage) return false;
        if (starredOnly && !project.starred) return false;
        if (!showArchived && project.archived) return false;
        return true;
      })
      .sort((a, b) =>
        sortBy === 'name'
          ? a.name.localeCompare(b.name)
          : lastSeen(b) - lastSeen(a) || a.name.localeCompare(b.name)
      )
  );

  // Projects nested inside another (by path or manual grouping) sit under their
  // root row. Computed over the filtered set so a nested project still surfaces
  // as a root when its parent is filtered out.
  const groups = $derived(groupByArea(nestProjects(filteredProjects)));

  const RECENT_WINDOW_MS = 30 * 24 * 60 * 60 * 1000;

  // Groups collapse, so 184 rows aren't all in the DOM at once. Open/closed is
  // remembered per group.
  let openGroups = $state<Record<string, boolean>>(readOpenGroups());

  function setGroupOpen(key: string, open: boolean) {
    openGroups = { ...openGroups, [key]: open };
    saveOpenGroups(openGroups);
  }

  let viewMode = $state<ViewMode>(readViewMode());

  function setViewMode(mode: string) {
    if (mode !== 'list' && mode !== 'grid') return;
    viewMode = mode;
    saveViewMode(mode);
  }

  function clearFilters() {
    searchQuery = '';
    filterStack = null;
    filterActivity = null;
    filterStage = null;
    starredOnly = false;
    showArchived = false;
  }

  // Default open: groups holding something touched in the last 30 days, off the
  // same `lastSeen` the "Recently active" sort uses.
  const recentGroupKeys = $derived.by(() => {
    const cutoff = Date.now() - RECENT_WINDOW_MS;
    const touched = (p: Project) => lastSeen(p) >= cutoff;
    return new Set(
      groups
        .filter((g) => g.items.some((i) => touched(i.project) || i.children.some(touched)))
        .map((g) => g.key)
    );
  });

  // While searching or filtering, everything matching stays visible.
  const isGroupOpen = (group: ProjectGroup) =>
    isFiltered || (openGroups[group.key] ?? recentGroupKeys.has(group.key));

  // A group holding one project doesn't earn a header — the row carries the path.
  const isLoneProject = (group: ProjectGroup) =>
    group.key !== STARRED_KEY && group.items.length === 1 && group.items[0].children.length === 0;

  const stats = $derived<PageStat[]>([
    { label: 'Total', value: projects.length, icon: FolderOpen },
    {
      label: 'Active this week',
      value: projects.filter((p) => p.activity_status === 'active').length,
      icon: Activity,
    },
    {
      label: 'Need attention',
      value: projects.filter((p) => p.has_uncommitted_changes || (p.unpushed_commits ?? 0) > 0)
        .length,
      icon: AlertTriangle,
    },
    { label: 'Starred', value: projects.filter((p) => p.starred).length, icon: Star },
    {
      label: 'Archived',
      value: projects.filter((p) => p.archived).length,
      icon: Archive,
    },
  ]);

  const hasHiddenArchived = $derived(!showArchived && projects.some((p) => p.archived));

  const isFiltered = $derived(
    Boolean(
      searchQuery.trim() ||
      filterStack ||
      filterActivity ||
      filterStage ||
      starredOnly ||
      showArchived
    )
  );

  const uniqueStacks = $derived.by(() => {
    if (!registryReady) return [];
    const names = new Set<string>();
    for (const project of projects) for (const name of stackNames(project)) names.add(name);
    return [...names].sort((a, b) => a.localeCompare(b));
  });

  // Manual grouping: a project can be grouped under any project except itself
  // and its own descendants (which would make a cycle).
  function eligibleParents(project: Project): Array<{ id: string; name: string }> {
    const blocked = descendantIds(projects, project.id);
    blocked.add(project.id);
    return projects
      .filter((p) => !blocked.has(p.id))
      .map((p) => ({ id: p.id, name: p.name }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }

  async function handleToggleStar(project: Project) {
    try {
      await toggleProjectStar(Number(project.id));
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (error) {
      log.error('Failed to toggle star', error);
      toast.error('Could not update the star');
    }
  }

  const handleSetStage = async (project: Project, stage: ProjectStage | null) => {
    try {
      await setProjectStage(Number(project.id), stage);
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (error) {
      log.error('Failed to set project stage', error);
      toast.error('Could not update the stage');
    }
  };

  const handleSetArchived = async (project: Project, archived: boolean) => {
    try {
      await setProjectArchived(Number(project.id), archived);
      await queryClient.invalidateQueries({ queryKey: queryKeys.projects.all });
    } catch (error) {
      log.error('Failed to set project archived', error);
      toast.error('Could not update the archive state');
    }
  };

  const handleSetParent = async (projectId: string, parentId: number | null) => {
    try {
      await projectService.setProjectParent(projectId, parentId);
    } catch (error) {
      log.error('Failed to set project parent', error);
    }
  };

  const handleProjectDelete = async (project: Project) => {
    const confirmed = await confirmAction(
      `Are you sure you want to delete "${project.name}"?`,
      'Delete project'
    );
    if (confirmed) {
      try {
        await projectService.deleteProject(project.id);
        log.info('Project deleted', { id: project.id, name: project.name });
      } catch (error) {
        log.error('Failed to delete project', error);
      }
    }
  };
</script>

<svelte:head>
  <title>Projects - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader title="Projects" description="Manage your development projects">
    {#snippet actions()}
      <Button variant="outline" onclick={() => goto('/projects/audit')}>
        <ScanSearch class="mr-2 h-4 w-4" />
        Audit
      </Button>
      <Button onclick={() => goto('/projects/create')}>
        <Plus class="mr-2 h-4 w-4" />
        New Project
      </Button>
    {/snippet}
  </PageHeader>

  <!-- No fabricated zeros while the list is still loading. -->
  <PageStats stats={isLoading ? stats.map((s) => ({ ...s, value: '–' })) : stats} />

  <ProjectsFilterBar
    bind:searchQuery
    bind:filterStack
    bind:filterActivity
    bind:filterStage
    bind:starredOnly
    bind:showArchived
    bind:sortBy
    {viewMode}
    {uniqueStacks}
    {isFiltered}
    onClear={clearFilters}
    onViewModeChange={setViewMode}
  />

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
      filteredDescription={hasHiddenArchived
        ? 'All matching projects are archived — turn on "Show archived" to see them'
        : 'Try adjusting your search criteria'}
      isFiltered={isFiltered || hasHiddenArchived}
      icon={FolderOpen}
      actionLabel="Create Project"
      onAction={() => goto('/projects/create')}
    />
  {:else}
    {#snippet row(project: Project, nested: boolean, pathPrefix: string)}
      <ProjectRow
        {project}
        {nested}
        {pathPrefix}
        {registryReady}
        modules={modulesByProject.get(Number(project.id)) ?? []}
        eligibleParents={eligibleParents(project)}
        onDelete={handleProjectDelete}
        onSetParent={handleSetParent}
        onToggleStar={handleToggleStar}
        onSetStage={handleSetStage}
        onSetArchived={handleSetArchived}
      />
    {/snippet}

    {#snippet card(project: Project)}
      <ProjectCard
        {project}
        {registryReady}
        modules={modulesByProject.get(Number(project.id)) ?? []}
        eligibleParents={eligibleParents(project)}
        onDelete={handleProjectDelete}
        onSetParent={handleSetParent}
        onToggleStar={handleToggleStar}
        onSetStage={handleSetStage}
        onSetArchived={handleSetArchived}
      />
    {/snippet}

    <ProjectGroupList
      {groups}
      {viewMode}
      {isFiltered}
      {isGroupOpen}
      {isLoneProject}
      onGroupOpenChange={setGroupOpen}
      {row}
      {card}
    />
  {/if}
</div>
