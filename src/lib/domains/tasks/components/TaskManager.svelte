<script lang="ts">
  import FaTableColumns from '~icons/fa6-solid/table-columns';
  import FaList from '~icons/fa6-solid/list';
  import FaSquareCheck from '~icons/fa6-solid/square-check';
  import FaXmark from '~icons/fa6-solid/xmark';
  import FaKeyboard from '~icons/fa6-solid/keyboard';
  import FaPlus from '~icons/fa6-solid/plus';
  import FaMagnifyingGlass from '~icons/fa6-solid/magnifying-glass';
  import FaFilter from '~icons/fa6-solid/filter';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { goto } from '$app/navigation';
  import { isLoading, error } from '../stores/taskStore';
  import { taskUi } from '../state/taskUi.svelte';
  import { TaskManagerState } from '../state/taskManager.svelte';
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
    ShellSidebarLayout,
    ResponsivePanel,
  } from '$lib/components/shell';
  import TaskDetail from './TaskDetail.svelte';
  import TaskFilterModal from './TaskFilterModal.svelte';
  import KanbanBoard from './KanbanBoard.svelte';
  import TaskList from './TaskList.svelte';
  import ArchiveView from './ArchiveView.svelte';
  import TaskShortcutsDialog from './TaskShortcutsDialog.svelte';
  import TaskProjectNav from './TaskProjectNav.svelte';
  import TaskActiveFilters from './TaskActiveFilters.svelte';
  import TaskMultiSelectBar from './TaskMultiSelectBar.svelte';

  const m = new TaskManagerState();
  const { navigate, closeTask, openTask } = m;
</script>

<svelte:window onkeydowncapture={m.noteDialogState} onkeydown={m.handleKeydown} />

<div class="flex h-full min-h-0 w-full flex-col overflow-hidden bg-background">
  <div class="shrink-0 px-4 py-4 md:px-6">
    <PageHeader title="Tasks" description="Organize and track your work efficiently">
      {#snippet actions()}
        {#if !taskUi.isMultiSelectMode}
          <Button onclick={m.handleMultiSelectToggle} variant="outline" size="sm">
            <FaSquareCheck class="mr-1.5 h-4 w-4" />
            Multi-Select
          </Button>
        {/if}
        <Button
          onclick={() => (m.showKeyboardShortcuts = true)}
          variant="ghost"
          size="sm"
          title="Keyboard Shortcuts (Ctrl+H)"
        >
          <FaKeyboard class="mr-1.5 h-4 w-4" />
          Shortcuts
        </Button>
        <Button onclick={m.handleTaskCreate} size="sm">
          <FaPlus class="mr-1.5 h-4 w-4" />
          New Task
        </Button>
      {/snippet}
    </PageHeader>
  </div>

  <ShellSidebarLayout mobileTriggerLabel="Projects menu">
    {#snippet sidebar()}
      <TaskProjectNav
        projects={m.sidebarProjects}
        selected={m.selectedProject}
        archiveActive={m.currentView === 'archive'}
        openTotal={taskUi.projectCounts.openTotal}
        noProjectOpen={m.noProjectOpen}
        showNoProject={m.hasNoProjectTasks}
        archivedCount={taskUi.projectCounts.archived}
        onSelect={(project) =>
          navigate(project, m.currentView === 'archive' ? 'board' : m.currentView)}
        onArchive={() => navigate(m.selectedProject, 'archive')}
      />
    {/snippet}

    <!--
      Bounded flex column, not a scroller: the toolbar stays put and the content
      region below owns the scrolling. Board view scrolls inside KanbanBoard
      (columns scroll vertically, the row scrolls horizontally) so its horizontal
      scrollbar sits at the viewport edge instead of at the foot of a tall page.
    -->
    <div class="flex min-h-0 min-w-0 flex-1 overflow-hidden">
      <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-hidden p-4 md:p-6">
        <!-- One toolbar row. The per-column counts live in the column headers. -->
        <div class="flex flex-wrap items-center gap-2">
          <div class="relative min-w-[200px] flex-1 sm:max-w-sm">
            <FaMagnifyingGlass
              class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              type="text"
              placeholder="Search tasks..."
              bind:value={m.searchQuery}
              class="h-9 pl-9"
            />
          </div>

          <Button
            variant="outline"
            size="sm"
            onclick={() => (m.showFilterModal = !m.showFilterModal)}
          >
            <FaFilter class="mr-1.5 h-4 w-4" />
            Filters
            {#if m.hasActiveFilters}
              <Badge variant="secondary" class="ml-1.5 px-1.5 py-0.5 text-xs">
                {m.activeFilterCount}
              </Badge>
            {/if}
          </Button>

          <div class="flex rounded-lg bg-muted p-0.5">
            <Button
              onclick={() => navigate(m.selectedProject, 'board')}
              variant={m.currentView === 'board' ? 'default' : 'ghost'}
              size="sm"
              class="h-8 px-3"
            >
              <FaTableColumns class="mr-1.5 h-4 w-4" />
              Kanban
            </Button>
            <Button
              onclick={() => navigate(m.selectedProject, 'list')}
              variant={m.currentView === 'list' ? 'default' : 'ghost'}
              size="sm"
              class="h-8 px-3"
            >
              <FaList class="mr-1.5 h-4 w-4" />
              List
            </Button>
          </div>

          {#if m.hasActiveFilters}
            <TaskActiveFilters
              searchQuery={m.searchQuery}
              statusFilters={m.statusFilters}
              priorityFilters={m.priorityFilters}
              typeFilters={m.typeFilters}
              onClearSearch={() => (m.searchQuery = '')}
              onToggleStatus={m.toggleStatusFilter}
              onTogglePriority={m.togglePriorityFilter}
              onToggleType={m.toggleTypeFilter}
              onClearAll={m.clearAllFilters}
            />
          {/if}
        </div>

        <!-- Multi-select controls (only while active) -->
        {#if taskUi.isMultiSelectMode}
          <TaskMultiSelectBar
            count={taskUi.selectedTaskIds.size}
            onSelectAll={m.handleSelectAll}
            onClear={m.handleClearSelection}
            onDelete={m.handleBulkDelete}
            onCancel={m.handleMultiSelectToggle}
          />
        {/if}

        <!-- Filter Modal -->
        {#if m.showFilterModal}
          <TaskFilterModal
            bind:open={m.showFilterModal}
            searchQuery={m.searchQuery}
            statusFilters={m.statusFilters}
            priorityFilters={m.priorityFilters}
            typeFilters={m.typeFilters}
            onSearchChange={(value) => (m.searchQuery = value)}
            onStatusFilterChange={(filters) => (m.statusFilters = filters)}
            onPriorityFilterChange={(filters) => (m.priorityFilters = filters)}
            onTypeFilterChange={(filters) => (m.typeFilters = filters)}
            onClearAll={m.clearAllFilters}
          />
        {/if}

        <!-- Main Content -->
        <div
          class="min-h-0 flex-1 {m.currentView === 'board' ? 'overflow-hidden' : 'overflow-y-auto'}"
        >
          {#if $isLoading}
            <PageLoading message="Loading tasks..." />
          {:else if $error}
            <PageError
              title="Failed to load tasks"
              message={$error}
              onRetry={() => m.tasksQuery.refetch()}
            />
          {:else if taskUi.parentTasks.length === 0 && !m.hasActiveFilters}
            <PageEmpty
              title="No tasks yet"
              description="Create your first task to start organizing your work."
              actionLabel="Create Task"
              onAction={() => goto('/tasks/create')}
            />
          {:else if m.currentView === 'archive'}
            {#if taskUi.archivedTasks.length === 0 && m.hasActiveFilters}
              <!-- archivedTasks is already search/filter-filtered, so an empty list
                 under active filters means "no match", not "nothing archived". -->
              <PageEmpty
                title="No matching archived tasks"
                description="No archived tasks match the current project, search or filters."
                actionLabel="Clear filters"
                onAction={m.clearAllFilters}
              />
            {:else if taskUi.archivedTasks.length === 0}
              <PageEmpty
                title="Archive is empty"
                description="Tasks completed more than 7 days ago and cancelled tasks show up here."
              />
            {:else}
              <ArchiveView
                tasks={taskUi.archivedTasks}
                onSelect={m.handleTaskSelect}
                projectLabel={m.projectLabel}
                onProjectClick={m.handleProjectClick}
              />
            {/if}
          {:else if !m.hasAnyVisible}
            <PageEmpty
              title="No matching tasks"
              description="No tasks match the current project, search or filters."
              actionLabel="Clear filters"
              onAction={m.clearAllFilters}
            />
          {:else if m.currentView === 'board'}
            <KanbanBoard
              handleTaskSelect={m.handleTaskSelect}
              handleTaskStatusToggle={m.handleTaskStatusToggle}
              handleTaskSelection={m.handleTaskSelection}
              handleCreateSubtask={m.handleCreateSubtask}
              onSetStatus={m.handleSetStatus}
              onCopyReference={m.handleCopyReference}
              onLaunchCoder={m.handleLaunchCoder}
              onLaunchAgents={m.handleLaunchAgents}
              onEditTask={m.handleEditTask}
              onDeleteTask={m.handleDeleteTask}
              projectLabel={m.projectLabel}
              onProjectClick={m.handleProjectClick}
              onShowArchive={() => navigate(m.selectedProject, 'archive')}
            />
          {:else}
            <TaskList
              handleTaskSelect={m.handleTaskSelect}
              handleTaskStatusToggle={m.handleTaskStatusToggle}
              handleTaskSelection={m.handleTaskSelection}
              handleCreateSubtask={m.handleCreateSubtask}
              onSetStatus={m.handleSetStatus}
              onCopyReference={m.handleCopyReference}
              onLaunchCoder={m.handleLaunchCoder}
              onLaunchAgents={m.handleLaunchAgents}
              onEditTask={m.handleEditTask}
              onDeleteTask={m.handleDeleteTask}
              projectLabel={m.projectLabel}
              onProjectClick={m.handleProjectClick}
            />
          {/if}
        </div>
      </div>

      <!--
        Task detail opens beside the board instead of navigating away, so scroll
        position, the project filter and the search text all survive. The URL is
        the record of what is open, which is what makes reload and Back work.
      -->
      {#if m.selectedTaskId}
        <ResponsivePanel
          bind:open={() => true, (open) => !open && closeTask()}
          side="right"
          desktopClass="w-[20rem] xl:w-[26rem]"
        >
          {#snippet header()}
            <div class="flex items-center justify-between gap-2 px-4 py-3">
              <h2 class="text-sm font-semibold">Task</h2>
              <Button variant="ghost" size="sm" onclick={closeTask} aria-label="Close task details">
                <FaXmark class="h-4 w-4" />
              </Button>
            </div>
          {/snippet}
          <div class="min-h-0 flex-1 overflow-y-auto p-4">
            <TaskDetail taskId={m.selectedTaskId} onDeleted={closeTask} onOpenTask={openTask} />
          </div>
        </ResponsivePanel>
      {/if}
    </div>
  </ShellSidebarLayout>

  <TaskShortcutsDialog bind:open={m.showKeyboardShortcuts} />
</div>
