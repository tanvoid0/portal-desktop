<!--
  Project Details Page
  Run-first: how to run it, what it's built with, and its git state.
-->

<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Terminal from '~icons/fa6-solid/terminal';
  import Info from '~icons/fa6-solid/circle-info';
  import Layers from '~icons/fa6-solid/layer-group';
  import Workflow from '~icons/fa6-solid/diagram-project';
  import FileKey from '~icons/fa6-solid/key';
  import BoxArchive from '~icons/fa6-solid/box-archive';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { ProjectTerminal } from '$lib/domains/terminal';
  import ProjectSummary from '$lib/domains/projects/components/ProjectSummary.svelte';
  import ProjectModules from '$lib/domains/projects/components/ProjectModules.svelte';
  import ProjectCommands from '$lib/domains/projects/components/ProjectCommands.svelte';
  import ProjectTechStack from '$lib/domains/projects/components/ProjectTechStack.svelte';
  import ProjectGitCard from '$lib/domains/projects/components/ProjectGitCard.svelte';
  import ProjectEnv from '$lib/domains/projects/components/ProjectEnv.svelte';
  import EnvBackupSection from '$lib/domains/projects/components/EnvBackupSection.svelte';
  import ProjectActionsPanel from '$lib/domains/actions/components/ProjectActionsPanel.svelte';
  import { PageHeader, PageLoading, PageError, ResponsivePanel } from '$lib/components/shell';
  import { Badge } from '$lib/components/ui/badge';
  import ProjectHeaderActions from '$lib/domains/projects/components/ProjectHeaderActions.svelte';
  import ProjectAgentSessionsCard from '$lib/domains/projects/components/ProjectAgentSessionsCard.svelte';
  import { toGitHubUrl, timeAgo } from '$lib/domains/projects/utils/detailFormat';
  import {
    ProjectDetailState,
    type ProjectTab,
  } from '$lib/domains/projects/state/projectDetail.svelte';

  const d = new ProjectDetailState();
</script>

<svelte:head>
  <title
    >{d.project ? `${d.project.name} - Project Details` : 'Project Details'} - Portal Desktop</title
  >
</svelte:head>

<div class="space-y-6">
  {#if d.loading}
    <PageLoading message="Loading project details..." />
  {:else if d.error}
    <PageError title="Failed to load project" message={d.error} onRetry={d.reloadProject} />
  {:else if d.project}
    {@const currentProject = d.project}
    {@const githubUrl = toGitHubUrl(currentProject.git_repository)}
    <PageHeader title={currentProject.name} badge={currentProject.status}>
      {#snippet subtitle()}
        {#if d.isMoving}
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
        <ProjectHeaderActions
          starred={!!currentProject.starred}
          {githubUrl}
          isDeploying={d.isDeploying}
          refreshing={d.refreshing}
          railDocked={d.displayTab === 'overview'}
          onShowDetails={() => (d.factsOpen = true)}
          onOpenEditor={d.handleOpenInEditor}
          onOpenFolder={d.handleOpenInExplorer}
          onTerminal={d.switchToTerminal}
          onEdit={d.handleEdit}
          onDeploy={d.handleDeploy}
          onToggleStar={d.handleToggleStar}
          onRedetect={d.handleRedetect}
        />
      {/snippet}
    </PageHeader>

    <div class="flex items-start gap-6">
      <div class="min-w-0 flex-1">
        <Tabs
          value={d.displayTab}
          onValueChange={(v) => d.setActiveTab(v as ProjectTab)}
          class="w-full"
        >
          <TabsList>
            <TabsTrigger value="overview" class="gap-2">
              <FolderOpen class="h-4 w-4" />
              Overview
            </TabsTrigger>
            {#if d.hasModules}
              <TabsTrigger value="apps" class="gap-2">
                <Layers class="h-4 w-4" />
                Apps ({d.modules.length})
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
            <TabsTrigger value="backup" class="gap-2">
              <BoxArchive class="h-4 w-4" />
              Backup
            </TabsTrigger>
            <TabsTrigger value="terminal" class="gap-2">
              <Terminal class="h-4 w-4" />
              Terminal
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" class="mt-6 space-y-6">
            <ProjectCommands
              project={currentProject}
              runningCommand={d.runningCommand}
              refreshing={d.refreshing}
              onRun={d.handleRunCommand}
              onRedetect={d.handleRedetect}
              onSaved={d.reloadProject}
            />
            {#if currentProject.last_opened}
              <p class="text-xs text-muted-foreground">
                Last opened {timeAgo(currentProject.last_opened)}
              </p>
            {/if}
          </TabsContent>

          {#if d.hasModules}
            <TabsContent value="apps" class="mt-6">
              <ProjectModules
                modules={d.modules}
                rescanning={d.rescanningModules}
                onRescan={d.handleRescanModules}
              />
            </TabsContent>
          {/if}

          <TabsContent value="actions" class="mt-6 space-y-6">
            <ProjectActionsPanel project={currentProject} enabled={d.activeTab === 'actions'} />
          </TabsContent>

          <TabsContent value="env" class="mt-6">
            <ProjectEnv
              projectId={Number(currentProject.id)}
              projectPath={currentProject.path}
              enabled={d.activeTab === 'env'}
            />
          </TabsContent>

          <TabsContent value="backup" class="mt-6">
            <EnvBackupSection
              projectId={Number(currentProject.id)}
              projectName={currentProject.name}
              title="Env file backup"
              description="Back up this project's .env files into the vault, encrypted, and restore them on this or another machine. They are never sent anywhere unencrypted."
            />
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

      <!-- Docked on Overview; on work tabs it is a Sheet behind the header's Details button. -->
      <ResponsivePanel
        bind:open={d.factsOpen}
        side="right"
        desktopClass="w-80"
        title="Details"
        collapsed={d.displayTab !== 'overview'}
      >
        <div class="space-y-4 overflow-y-auto p-4">
          {#if d.about}
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center gap-2">
                  <Info class="h-5 w-5" />
                  About
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p class="text-sm text-muted-foreground">{d.about}</p>
              </CardContent>
            </Card>
          {/if}
          <ProjectTechStack
            project={currentProject}
            modules={d.modules}
            refreshing={d.refreshing}
            onRedetect={d.handleRedetect}
          />
          <ProjectGitCard
            project={currentProject}
            {githubUrl}
            refreshing={d.refreshing}
            onRedetect={d.handleRedetect}
            onTerminal={d.switchToTerminal}
          />
          <ProjectAgentSessionsCard project={currentProject} sessions={d.agentSessionsForProject} />
        </div>
      </ResponsivePanel>
    </div>
  {/if}
</div>
