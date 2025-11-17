<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { Input } from '@/lib/components/ui/input';
	import Icon from '@iconify/svelte';
	import { goto } from '$app/navigation';
	import { 
		tasks, 
		selectedTask, 
		selectedTaskIds, 
		isMultiSelectMode,
		taskStats,
		filteredTasks,
		kanbanColumns,
		taskActions,
		getSubtaskCount,
		isLoading,
		error
	} from '../stores/taskStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import LoadingSpinner from '@/lib/components/ui/loading-spinner.svelte';
	import type { Task, TaskStatus, TaskPriority } from '../types';
	import TaskCard from './TaskCard.svelte';
	import TaskStats from './TaskStats.svelte';
	import TaskProgress from './TaskProgress.svelte';
	import TaskFilterModal from './TaskFilterModal.svelte';
	import KanbanBoard from './KanbanBoard.svelte';
	import TaskList from './TaskList.svelte';
	import QuickActions from './QuickActions.svelte';
	import SmartFilters from './SmartFilters.svelte';
	import SavedViews from './SavedViews.svelte';
	import TimeTracker from './TimeTracker.svelte';
	import TemplateManager from './TemplateManager.svelte';

	// View state
	let currentView = $state<'kanban' | 'list'>('kanban');
	let searchQuery = $state('');
	let showSidebar = $state(true);
	let sidebarTab = $state<'actions' | 'filters' | 'views' | 'templates' | 'tracker'>('actions');

	// Filter modal state
	let showFilterModal = $state(false);
	
	// Keyboard shortcuts modal state
	let showKeyboardShortcuts = $state(false);
	
	// Delete modal state
	let showDeleteModal = $state(false);
	let taskToDelete = $state<Task | null>(null);

	// Filter state
	let statusFilters = $state<TaskStatus[]>([]);
	let priorityFilters = $state<TaskPriority[]>([]);
	let typeFilters = $state<string[]>([]);

	// Multi-select state
	let selectedTaskIdsState = $state($selectedTaskIds);
	let isMultiSelectModeState = $state($isMultiSelectMode);

	// Computed values
	let parentTasks = $derived($tasks.filter(task => !task.parentId));
	let filteredTasksWithSearch = $derived(
		parentTasks.filter(task => {
			if (searchQuery.trim()) {
				const query = searchQuery.toLowerCase();
				return task.title.toLowerCase().includes(query) ||
					(task.description && task.description.toLowerCase().includes(query));
			}
			return true;
		})
	);

	// Active filter states
	let activeStatusFilters = $derived(statusFilters);
	let activePriorityFilters = $derived(priorityFilters);
	let activeTypeFilters = $derived(typeFilters);
	let hasActiveFilters = $derived(
		searchQuery.trim() !== '' || 
		statusFilters.length > 0 || 
		priorityFilters.length > 0 || 
		typeFilters.length > 0
	);
	let activeFilterCount = $derived(
		(statusFilters.length > 0 ? 1 : 0) +
		(priorityFilters.length > 0 ? 1 : 0) +
		(typeFilters.length > 0 ? 1 : 0) +
		(searchQuery.trim() !== '' ? 1 : 0)
	);

	// Helper functions
	function getTaskStatusColor(status: string) {
		switch (status) {
			case 'completed': return 'text-green-500';
			case 'in-progress': return 'text-blue-500';
			case 'cancelled': return 'text-red-500';
			default: return 'text-gray-400';
		}
	}

	function getStatusBadgeColor(status: string) {
		switch (status) {
			case 'completed': return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300';
			case 'in-progress': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300';
			case 'cancelled': return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300';
			default: return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
		}
	}

	function getPriorityColor(priority: string) {
		switch (priority) {
			case 'high': return 'text-red-500';
			case 'medium': return 'text-yellow-500';
			case 'low': return 'text-green-500';
			default: return 'text-gray-400';
		}
	}

	function getTaskIcon(task: Task) {
		switch (task.status) {
			case 'completed': return 'mdi:check-circle';
			case 'in-progress': return 'mdi:progress-clock';
			case 'cancelled': return 'mdi:cancel';
			default: return 'mdi:circle-outline';
		}
	}

	// Event handlers
	function handleTaskSelect(task: Task) {
		taskActions.selectTask(task);
		// Add haptic feedback for better UX
		if (navigator.vibrate) {
			navigator.vibrate(50);
		}
		goto(`/tasks/${task.id}`);
	}

	function handleTaskCreate() {
		// Add smooth transition
		goto('/tasks/create');
	}


	function handleTaskEdit(task: Task) {
		goto(`/tasks/${task.id}/edit`);
	}


	// Filter functions
	function toggleStatusFilter(status: TaskStatus) {
		if (statusFilters.includes(status)) {
			statusFilters = statusFilters.filter(s => s !== status);
		} else {
			statusFilters = [...statusFilters, status];
		}
	}

	function togglePriorityFilter(priority: TaskPriority) {
		if (priorityFilters.includes(priority)) {
			priorityFilters = priorityFilters.filter(p => p !== priority);
		} else {
			priorityFilters = [...priorityFilters, priority];
		}
	}

	function toggleTypeFilter(type: string) {
		if (typeFilters.includes(type)) {
			typeFilters = typeFilters.filter(t => t !== type);
		} else {
			typeFilters = [...typeFilters, type];
		}
	}

	function clearAllFilters() {
		searchQuery = '';
		statusFilters = [];
		priorityFilters = [];
		typeFilters = [];
	}

	function handleTaskStatusToggle(taskId: string) {
		taskActions.toggleTaskStatus(taskId);
	}

	function handleMultiSelectToggle() {
		taskActions.toggleMultiSelectMode();
	}

	function handleTaskSelection(taskId: string) {
		taskActions.toggleTaskSelection(taskId);
	}

	function handleCreateSubtask(parentTask: Task) {
		// Navigate to create task page with parentId pre-filled
		goto(`/tasks/create?parentId=${parentTask.id}`);
	}

	function handleSelectAll() {
		taskActions.selectAllTasks(filteredTasksWithSearch.map(task => task.id));
	}

	function handleClearSelection() {
		taskActions.clearSelection();
	}

	async function handleBulkDelete() {
		if ($selectedTaskIds.size > 0) {
			if (confirm(`Are you sure you want to delete ${$selectedTaskIds.size} selected task${$selectedTaskIds.size === 1 ? '' : 's'}?`)) {
				try {
					await taskActions.deleteTasksBulk(Array.from($selectedTaskIds));
					toastActions.success(
						'Tasks deleted',
						`Successfully deleted ${$selectedTaskIds.size} task${$selectedTaskIds.size === 1 ? '' : 's'}`
					);
				} catch (err) {
					toastActions.error(
						'Failed to delete tasks',
						err instanceof Error ? err.message : 'An unexpected error occurred'
					);
				}
			}
		}
	}

	// Keyboard shortcuts
	function handleKeydown(event: KeyboardEvent) {
		if (event.ctrlKey || event.metaKey) {
			if (event.key === 'n') {
				event.preventDefault();
				handleTaskCreate();
			} else if (event.key === 'f') {
				event.preventDefault();
				showFilterModal = true;
			} else if (event.key === 'k') {
				event.preventDefault();
				// Focus search input
				const searchInput = document.querySelector('input[placeholder="Search tasks..."]') as HTMLInputElement;
				if (searchInput) {
					searchInput.focus();
				}
			} else if (event.key === 'h') {
				event.preventDefault();
				showKeyboardShortcuts = true;
			} else if (event.key === 'm') {
				event.preventDefault();
				handleMultiSelectToggle();
			} else if (event.key === 'a' && $isMultiSelectMode) {
				event.preventDefault();
				handleSelectAll();
			} else if (event.key === 'Delete' && $selectedTaskIds.size > 0) {
				event.preventDefault();
				handleBulkDelete();
			}
		} else if (event.key === 'Escape') {
			if (showFilterModal) {
				showFilterModal = false;
			} else if (showKeyboardShortcuts) {
				showKeyboardShortcuts = false;
			} else if ($isMultiSelectMode) {
				handleMultiSelectToggle();
			}
		}
	}

	// Initialize
	onMount(() => {
		// Tasks are automatically loaded from database on app start
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-background">
	<!-- Header -->
	<div class="border-b border-border bg-card">
		<div class="w-full px-4 py-3">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-xl font-bold text-foreground">Task Manager</h1>
					<p class="text-xs text-muted-foreground">Organize and track your work efficiently</p>
				</div>
				
				<div class="flex items-center gap-3">
					<!-- View Toggle -->
					<div class="flex bg-muted rounded-lg p-1">
						<Button
							onclick={() => currentView = 'kanban'}
							variant={currentView === 'kanban' ? 'default' : 'ghost'}
							size="sm"
							class="px-3 py-1.5 text-sm font-medium"
						>
							<Icon icon="mdi:view-column" class="w-4 h-4 mr-1.5" />
							Kanban
						</Button>
						<Button
							onclick={() => currentView = 'list'}
							variant={currentView === 'list' ? 'default' : 'ghost'}
							size="sm"
							class="px-3 py-1.5 text-sm font-medium"
						>
							<Icon icon="mdi:format-list-bulleted" class="w-4 h-4 mr-1.5" />
							List
						</Button>
					</div>
					
					<!-- Multi-select Controls -->
					{#if $isMultiSelectMode}
						<div class="flex items-center gap-3 bg-warning-50 dark:bg-warning-900/20 border border-warning-200 dark:border-warning-800 rounded-lg px-4 py-3">
							<Badge variant="secondary" class="bg-warning-100 text-warning-800 dark:bg-warning-800 dark:text-warning-100">
								<Icon icon="mdi:checkbox-multiple-marked" class="w-3 h-3 mr-1" />
								{$selectedTaskIds.size} selected
							</Badge>
							<div class="flex items-center gap-2">
								<Button
									onclick={handleSelectAll}
									variant="outline"
									size="sm"
									class="h-7 px-2 text-xs"
								>
									<Icon icon="mdi:select-all" class="w-3 h-3 mr-1" />
									Select All
								</Button>
								<Button
									onclick={handleClearSelection}
									variant="outline"
									size="sm"
									class="h-7 px-2 text-xs"
								>
									<Icon icon="mdi:close" class="w-3 h-3 mr-1" />
									Clear
								</Button>
								<Button
									onclick={handleBulkDelete}
									disabled={$selectedTaskIds.size === 0}
									variant="destructive"
									size="sm"
									class="h-7 px-2 text-xs"
								>
									<Icon icon="mdi:delete" class="w-3 h-3 mr-1" />
									Delete ({$selectedTaskIds.size})
								</Button>
								<Button
									onclick={handleMultiSelectToggle}
									variant="ghost"
									size="sm"
									class="h-7 px-2 text-xs"
								>
									<Icon icon="mdi:close" class="w-3 h-3 mr-1" />
									Cancel
								</Button>
							</div>
						</div>
					{:else}
						<Button
							onclick={handleMultiSelectToggle}
							variant="outline"
							class="flex items-center space-x-2"
						>
							<Icon icon="mdi:checkbox-multiple-marked" class="w-4 h-4" />
							<span>Multi-Select</span>
						</Button>
					{/if}
					
					<!-- Action Buttons -->
					<Button
						onclick={() => showKeyboardShortcuts = true}
						variant="ghost"
						class="flex items-center space-x-2"
						title="Keyboard Shortcuts (Ctrl+H)"
					>
						<Icon icon="mdi:keyboard" class="w-4 h-4" />
						<span>Shortcuts</span>
					</Button>
					<Button
						onclick={() => goto('/tasks/generate')}
						variant="outline"
						class="flex items-center space-x-2"
					>
						<Icon icon="lucide:sparkles" class="w-4 h-4" />
						<span>Generate Tasks with AI</span>
					</Button>
					<Button
						onclick={handleTaskCreate}
						class="flex items-center space-x-2"
					>
						<Icon icon="mdi:plus" class="w-4 h-4" />
						<span>New Task</span>
					</Button>
				</div>
			</div>
		</div>
	</div>

	<!-- Main Content -->
	<div class="w-full px-4 py-4">
		<!-- Stats and Progress Row -->
		<div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-4">
			<!-- Left: Compact Stats -->
			<div class="lg:col-span-2">
				<TaskStats />
			</div>
			<!-- Right: Progress Bar -->
			<div class="lg:col-span-1">
				<TaskProgress />
			</div>
		</div>

		<!-- Search and Filters Row -->
		<Card class="mb-4">
			<CardContent class="p-3">
				<div class="flex items-center gap-3">
					<!-- Search Input -->
					<div class="flex-1 relative">
						<Icon icon="mdi:magnify" class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
						<Input
							type="text"
							placeholder="Search tasks..."
							bind:value={searchQuery}
							class="pl-10"
						/>
					</div>
					
					<!-- Filter Button -->
					<Button
						variant="outline"
						onclick={() => showFilterModal = !showFilterModal}
						class="flex items-center gap-2"
					>
						<Icon icon="mdi:filter-variant" class="w-4 h-4" />
						Filters
						{#if hasActiveFilters}
							<Badge variant="secondary" class="ml-1 px-1.5 py-0.5 text-xs">
								{activeFilterCount}
							</Badge>
						{/if}
					</Button>
				</div>
				
				<!-- Active Filters Display -->
				{#if hasActiveFilters}
					<div class="mt-3 pt-3 border-t border-border">
						<div class="flex items-center gap-2 flex-wrap">
							<span class="text-xs font-medium text-muted-foreground">Active filters:</span>
							
							{#if searchQuery.trim() !== ''}
								<Badge variant="outline" class="text-xs">
									<Icon icon="mdi:magnify" class="w-3 h-3 mr-1" />
									"{searchQuery}"
									<Button
										onclick={() => searchQuery = ''}
										variant="ghost"
										size="sm"
										class="ml-1 h-4 w-4 p-0 hover:bg-muted"
									>
										<Icon icon="mdi:close" class="w-3 h-3" />
									</Button>
								</Badge>
							{/if}
							
							{#each activeStatusFilters as status}
								<Badge variant="outline" class="text-xs">
									<Icon icon="mdi:circle" class="w-3 h-3 mr-1" />
									{status === 'pending' ? 'To Do' : 
									 status === 'in-progress' ? 'In Progress' : 
									 status === 'completed' ? 'Completed' : 'Cancelled'}
									<Button
										onclick={() => toggleStatusFilter(status)}
										variant="ghost"
										size="sm"
										class="ml-1 h-4 w-4 p-0 hover:bg-muted"
									>
										<Icon icon="mdi:close" class="w-3 h-3" />
									</Button>
								</Badge>
							{/each}
							
							{#each activePriorityFilters as priority}
								<Badge variant="outline" class="text-xs">
									<Icon icon="mdi:flag" class="w-3 h-3 mr-1 {priority === 'high' ? 'text-red-500' : priority === 'medium' ? 'text-yellow-500' : 'text-green-500'}" />
									{priority.charAt(0).toUpperCase() + priority.slice(1)}
									<Button
										onclick={() => togglePriorityFilter(priority)}
										variant="ghost"
										size="sm"
										class="ml-1 h-4 w-4 p-0 hover:bg-muted"
									>
										<Icon icon="mdi:close" class="w-3 h-3" />
									</Button>
								</Badge>
							{/each}
							
							{#each activeTypeFilters as type}
								<Badge variant="outline" class="text-xs">
									{type}
									<Button
										onclick={() => toggleTypeFilter(type)}
										variant="ghost"
										size="sm"
										class="ml-1 h-4 w-4 p-0 hover:bg-muted"
									>
										<Icon icon="mdi:close" class="w-3 h-3" />
									</Button>
								</Badge>
							{/each}
							
							<Button 
								variant="outline" 
								size="sm" 
								onclick={clearAllFilters}
								class="ml-2 text-xs"
							>
								<Icon icon="mdi:filter-remove" class="w-3 h-3 mr-1" />
								Clear All
							</Button>
						</div>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Filter Modal -->
		{#if showFilterModal}
			<TaskFilterModal 
				bind:open={showFilterModal}
				{searchQuery}
				{statusFilters}
				{priorityFilters}
				{typeFilters}
				onSearchChange={(value) => searchQuery = value}
				onStatusFilterChange={(filters) => statusFilters = filters}
				onPriorityFilterChange={(filters) => priorityFilters = filters}
				onTypeFilterChange={(filters) => typeFilters = filters}
				onClearAll={clearAllFilters}
			/>
		{/if}

		<!-- Main Content -->
		{#if $isLoading}
			<div class="flex items-center justify-center py-12">
				<LoadingSpinner size="lg" text="Loading tasks..." />
			</div>
		{:else if $error}
			<div class="flex items-center justify-center py-12">
				<div class="text-center space-y-4">
					<Icon icon="mdi:alert-circle" class="w-12 h-12 text-destructive mx-auto" />
					<div>
						<h3 class="text-lg font-semibold text-destructive">Failed to load tasks</h3>
						<p class="text-muted-foreground">{$error}</p>
					</div>
					<Button onclick={() => taskActions.loadTasks()} variant="outline">
						<Icon icon="mdi:refresh" class="w-4 h-4 mr-2" />
						Try Again
					</Button>
				</div>
			</div>
		{:else if currentView === 'kanban'}
			<KanbanBoard 
				{handleTaskSelect}
				{handleTaskStatusToggle}
				{handleTaskSelection}
				{handleCreateSubtask}
				{getSubtaskCount}
				{getTaskStatusColor}
				{getStatusBadgeColor}
				{getPriorityColor}
				{getTaskIcon}
			/>
		{:else if currentView === 'list'}
			<TaskList 
				{handleTaskSelect}
				{handleTaskStatusToggle}
				{handleTaskSelection}
				{handleCreateSubtask}
				{getSubtaskCount}
				{getTaskStatusColor}
				{getStatusBadgeColor}
				{getPriorityColor}
				{getTaskIcon}
			/>
		{/if}
	</div>

	<!-- Keyboard Shortcuts Modal -->
	{#if showKeyboardShortcuts}
		<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
			<Card class="w-full max-w-2xl max-h-[80vh] overflow-hidden">
				<CardHeader class="pb-3">
					<div class="flex items-center justify-between">
						<CardTitle class="text-lg">Keyboard Shortcuts</CardTitle>
						<Button
							variant="ghost"
							size="sm"
							onclick={() => showKeyboardShortcuts = false}
							class="h-8 w-8 p-0"
						>
							<Icon icon="mdi:close" class="w-4 h-4" />
						</Button>
					</div>
				</CardHeader>
				
				<CardContent class="space-y-6 overflow-y-auto">
					<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
						<div class="space-y-4">
							<h3 class="font-semibold text-foreground">Navigation & Actions</h3>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm">Create new task</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+N</kbd>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm">Open filters</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+F</kbd>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm">Focus search</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+K</kbd>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm">Toggle multi-select</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+M</kbd>
								</div>
							</div>
						</div>

						<div class="space-y-4">
							<h3 class="font-semibold text-foreground">Multi-Select Mode</h3>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm">Select all tasks</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+A</kbd>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm">Delete selected</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Delete</kbd>
								</div>
							</div>

							<h3 class="font-semibold text-foreground">General</h3>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm">Show shortcuts</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Ctrl+H</kbd>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm">Close modal</span>
									<kbd class="px-2 py-1 bg-muted rounded text-xs">Esc</kbd>
								</div>
							</div>
						</div>
					</div>

					<div class="pt-4 border-t border-border">
						<div class="flex items-center justify-end">
							<Button onclick={() => showKeyboardShortcuts = false}>
								Got it
							</Button>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	{/if}

</div>
