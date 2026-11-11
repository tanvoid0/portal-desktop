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
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Activity from '~icons/fa6-solid/wave-square';
  import AlertTriangle from '~icons/fa6-solid/triangle-exclamation';
  import Star from '~icons/fa6-solid/star';
  import Archive from '~icons/fa6-solid/box-archive';
  import Plus from '~icons/fa6-solid/plus';
  import ScanSearch from '~icons/fa6-solid/magnifying-glass';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import ListIcon from '~icons/fa6-solid/list';
  import GridIcon from '~icons/fa6-solid/table-cells-large';
  import ProjectRow from '$lib/domains/projects/components/ProjectRow.svelte';
  import ProjectCard from '$lib/domains/projects/components/ProjectCard.svelte';
  import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import Select from '$lib/components/ui/select.svelte';
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
  import { PROJECT_STAGES } from '$lib/domains/projects/types';
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
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    PageStats,
    PageFilters,
  } from '$lib/components/shell';
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

  // Groups collapse, so 184 rows aren't all in the DOM at once. Open/closed is
  // remembered per group; localStorage can throw or hold junk, in which case the
  // defaults below apply and the page still renders.
  const OPEN_GROUPS_KEY = 'portal.projects.openGroups';
  const RECENT_WINDOW_MS = 30 * 24 * 60 * 60 * 1000;

  function readOpenGroups(): Record<string, boolean> {
    if (typeof localStorage === 'undefined') return {};
    try {
      const parsed: unknown = JSON.parse(localStorage.getItem(OPEN_GROUPS_KEY) ?? 'null');
      return parsed && typeof parsed === 'object' ? (parsed as Record<string, boolean>) : {};
    } catch {
      return {};
    }
  }

  let openGroups = $state<Record<string, boolean>>(readOpenGroups());

  function setGroupOpen(key: string, open: boolean) {
    openGroups = { ...openGroups, [key]: open };
    try {
      localStorage?.setItem(OPEN_GROUPS_KEY, JSON.stringify(openGroups));
    } catch {
      // Quota or private mode — the choice still holds for this session.
    }
  }

  const VIEW_MODE_KEY = 'portal.projects.viewMode';
  type ViewMode = 'list' | 'grid';

  function readViewMode(): ViewMode {
    try {
      return localStorage?.getItem(VIEW_MODE_KEY) === 'grid' ? 'grid' : 'list';
    } catch {
      return 'list';
    }
  }

  let viewMode = $state<ViewMode>(readViewMode());

  function setViewMode(mode: string) {
    if (mode !== 'list' && mode !== 'grid') return;
    viewMode = mode;
    try {
      localStorage?.setItem(VIEW_MODE_KEY, mode);
    } catch {
      // Quota or private mode — the choice still holds for this session.
    }
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

  <PageFilters bind:searchQuery searchPlaceholder="Search projects...">
    {#snippet filters()}
      <Select
        value={filterStack ?? ''}
        placeholder="All stacks"
        options={[
          { value: '', label: 'All stacks' },
          ...uniqueStacks.map((n) => ({ value: n, label: n })),
        ]}
        onSelect={(value) => (filterStack = value || null)}
        class="w-[170px]"
      />

      <Select
        value={filterActivity ?? ''}
        placeholder="Any activity"
        options={[
          { value: '', label: 'Any activity' },
          { value: 'active', label: 'Active' },
          { value: 'recent', label: 'Recent' },
          { value: 'dormant', label: 'Dormant' },
          { value: 'stale', label: 'Stale' },
        ]}
        onSelect={(value) => (filterActivity = value || null)}
        class="w-[150px]"
      />

      <Select
        value={filterStage ?? ''}
        placeholder="Any stage"
        options={[
          { value: '', label: 'Any stage' },
          ...PROJECT_STAGES.map((s) => ({ value: s.value, label: s.label })),
          { value: 'none', label: 'Not set' },
        ]}
        onSelect={(value) => (filterStage = value || null)}
        class="w-[150px]"
      />

      <Select
        value={sortBy}
        options={[
          { value: 'recent', label: 'Recently active' },
          { value: 'name', label: 'Name' },
        ]}
        onSelect={(value) => (sortBy = value as typeof sortBy)}
        class="w-[170px]"
      />

      <div class="flex items-center gap-2">
        <Switch id="starred-only" bind:checked={starredOnly} />
        <Label for="starred-only" class="text-sm">Starred only</Label>
      </div>

      <div class="flex items-center gap-2">
        <Switch id="show-archived" bind:checked={showArchived} />
        <Label for="show-archived" class="text-sm">Show archived</Label>
      </div>
    {/snippet}

    {#snippet actions()}
      {#if isFiltered}
        <Button
          variant="ghost"
          size="sm"
          onclick={() => {
            searchQuery = '';
            filterStack = null;
            filterActivity = null;
            filterStage = null;
            starredOnly = false;
            showArchived = false;
          }}
          class="text-muted-foreground"
        >
          Clear filters
        </Button>
      {/if}

      <ToggleGroup type="single" value={viewMode} onValueChange={setViewMode} variant="outline">
        <ToggleGroupItem value="list" aria-label="List view">
          <ListIcon class="h-3.5 w-3.5" />
        </ToggleGroupItem>
        <ToggleGroupItem value="grid" aria-label="Grid view">
          <GridIcon class="h-3.5 w-3.5" />
        </ToggleGroupItem>
      </ToggleGroup>
    {/snippet}
  </PageFilters>

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

    {#snippet cardGrid(items: ProjectGroup['items'])}
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
        {#each items as item (item.project.id)}
          {@render card(item.project)}
          {#each item.children as child (child.id)}
            {@render card(child)}
          {/each}
        {/each}
      </div>
    {/snippet}

    <div class="space-y-6">
      {#each groups as group (group.key)}
        {#if isLoneProject(group)}
          {#if viewMode === 'grid'}
            {@render cardGrid(group.items)}
          {:else}
            {@render row(group.items[0].project, false, group.title)}
          {/if}
        {:else}
          <Collapsible
            bind:open={() => isGroupOpen(group), (open) => setGroupOpen(group.key, open)}
          >
            <section>
              <h2 class="mb-1 border-b px-3 pb-1">
                <!-- While a filter is on, every match stays expanded, so the toggle is inert. -->
                <CollapsibleTrigger
                  disabled={isFiltered}
                  class="group flex w-full items-baseline gap-2 rounded text-sm font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <ChevronRight
                    class="h-3 w-3 shrink-0 text-muted-foreground transition-transform group-data-[state=open]:rotate-90"
                  />
                  <span class="truncate">{group.title}</span>
                  <span class="text-xs font-normal text-muted-foreground">{group.items.length}</span
                  >
                </CollapsibleTrigger>
              </h2>
              <CollapsibleContent>
                {#if viewMode === 'grid'}
                  {@render cardGrid(group.items)}
                {:else}
                  {#each group.items as item (item.project.id)}
                    {@render row(item.project, false, '')}
                    {#each item.children as child (child.id)}
                      {@render row(child, true, '')}
                    {/each}
                  {/each}
                {/if}
              </CollapsibleContent>
            </section>
          </Collapsible>
        {/if}
      {/each}
    </div>
  {/if}
</div>
