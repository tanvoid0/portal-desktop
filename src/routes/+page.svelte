<script lang="ts">
  import { goto } from '$app/navigation';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { createDashboardOverviewQuery } from '$lib/domains/dashboard/queries/dashboardQueries';
  import { PageHeader, PageLoading, PageStats } from '$lib/components/shell';

  const dashboardQuery = createDashboardOverviewQuery();
  const overview = $derived(dashboardQuery.data);

  // Main navigation items
  const mainNavItems = $derived([
    {
      title: 'Projects',
      description: 'Manage your projects',
      url: '/projects',
      icon: 'folder',
      badge: overview?.project_stats.total_projects ?? 0,
      color: 'bg-primary/10 text-primary',
    },
    {
      title: 'Tasks',
      description: 'View and manage tasks',
      url: '/tasks',
      icon: 'check-square',
      badge: overview?.task_stats.total ?? 0,
      color: 'bg-status-success-bg text-status-success',
    },
    {
      title: 'Terminal',
      description: 'Integrated terminal',
      url: '/terminal',
      icon: 'terminal',
      color: 'bg-muted text-foreground',
    },
    {
      title: 'Cloud',
      description: 'Cloud resources',
      url: '/cloud',
      icon: 'cloud',
      color: 'bg-status-info-bg text-status-info',
    },
  ]);

  // Icon component helper
  function getIcon(iconName: string) {
    const icons: Record<string, string> = {
      'folder-plus':
        'M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      'check-square': 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
      terminal:
        'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
      code: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
      blocks:
        'M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z',
      cloud:
        'M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z',
      lock: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
      folder: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
      settings:
        'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
      container: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4',
    };
    return icons[iconName] || icons['folder'];
  }
</script>

<svelte:head>
  <title>Overview - Portal Desktop</title>
</svelte:head>

<div class="space-y-4">
  <PageHeader title="Overview" description="What's happening with your workspace">
    {#snippet actions()}
      <Button size="sm" onclick={() => goto('/projects/create')}>New Project</Button>
      <Button size="sm" variant="outline" onclick={() => goto('/tasks/create')}>New Task</Button>
    {/snippet}
  </PageHeader>

  {#if dashboardQuery.isPending && !overview}
    <PageLoading message="Loading dashboard..." />
  {:else}
    <PageStats
      stats={[
        { label: 'Tasks', value: overview?.task_stats.total ?? 0 },
        { label: 'Completed', value: overview?.task_stats.completed ?? 0 },
        { label: 'Projects', value: overview?.project_stats.total_projects ?? 0 },
        { label: 'Running services', value: overview?.running_services_count ?? 0 },
        { label: 'Completion', value: `${overview?.task_stats.completion_percentage ?? 0}%` },
      ]}
    />

    <!-- Main Navigation -->
    <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
      <Card
        class="cursor-pointer transition-shadow hover:shadow-sm"
        onclick={() => goto('/projects')}
      >
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle class="flex items-center gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                <svg
                  class="h-5 w-5 text-primary"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                  />
                </svg>
              </div>
              Projects
            </CardTitle>
            {#if (overview?.project_stats.total_projects ?? 0) > 0}
              <Badge variant="secondary">
                {overview?.project_stats.total_projects ?? 0}
              </Badge>
            {/if}
          </div>
          <CardDescription>Manage your projects and workspaces</CardDescription>
        </CardHeader>
        <CardContent>
          <p class="text-sm text-muted-foreground">
            {#if (overview?.project_stats.total_projects ?? 0) === 0}
              No projects yet. Create your first project to get started.
            {:else if (overview?.project_stats.recent_projects.length ?? 0) > 0}
              Recent: {overview?.project_stats.recent_projects[0]?.name}
            {:else}
              {overview?.project_stats.total_projects ?? 0} project
              {(overview?.project_stats.total_projects ?? 0) !== 1 ? 's' : ''} available
            {/if}
          </p>
        </CardContent>
      </Card>

      <Card class="cursor-pointer transition-shadow hover:shadow-sm" onclick={() => goto('/tasks')}>
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle class="flex items-center gap-3">
              <div
                class="flex h-10 w-10 items-center justify-center rounded-lg bg-status-success-bg"
              >
                <svg
                  class="h-5 w-5 text-status-success"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </div>
              Tasks
            </CardTitle>
            {#if (overview?.task_stats.total ?? 0) > 0}
              <Badge variant="secondary">{overview?.task_stats.total ?? 0}</Badge>
            {/if}
          </div>
          <CardDescription>View and manage your tasks</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="space-y-2">
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">Total</span>
              <span class="font-medium">{overview?.task_stats.total ?? 0}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground">Completed</span>
              <span class="font-medium text-status-success">
                {overview?.task_stats.completed ?? 0}
              </span>
            </div>
            {#if (overview?.task_stats.total ?? 0) > 0}
              <div class="mt-2 h-2 w-full rounded-full bg-secondary">
                <div
                  class="h-2 rounded-full bg-status-success transition-all"
                  style="width: {((overview?.task_stats.completed ?? 0) /
                    (overview?.task_stats.total ?? 1)) *
                    100}%"
                ></div>
              </div>
            {/if}
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- Additional Quick Links -->
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
      {#each mainNavItems.slice(2) as item}
        <Card
          class="cursor-pointer transition-shadow hover:shadow-sm"
          onclick={() => goto(item.url)}
        >
          <CardContent class="p-4">
            <div class="flex items-center gap-4">
              <div class="h-12 w-12 rounded-lg {item.color} flex items-center justify-center">
                <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d={getIcon(item.icon)}
                  />
                </svg>
              </div>
              <div class="flex-1">
                <h3 class="mb-1 font-semibold">{item.title}</h3>
                <p class="text-sm text-muted-foreground">{item.description}</p>
              </div>
              {#if item.badge}
                <Badge variant="secondary">{item.badge}</Badge>
              {/if}
            </div>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>
