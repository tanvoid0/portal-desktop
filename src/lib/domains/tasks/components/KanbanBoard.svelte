<script lang="ts">
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import { kanbanColumns, tasks, selectedTaskIds, isMultiSelectMode, taskActions } from '../stores/taskStore';
	import { createDragDropState, handleDragStart, handleDragEnd, handleDragOver, handleDragLeave, handleDrop, getDropZoneClasses, getTaskCardClasses } from '../utils/dragDrop';
	import type { Task } from '../types';
	import TaskCard from './TaskCard.svelte';

	interface Props {
		handleTaskSelect: (task: Task) => void;
		handleTaskStatusToggle: (taskId: string) => void;
		handleTaskSelection: (taskId: string) => void;
		handleCreateSubtask: (task: Task) => void;
		getSubtaskCount: (taskId: string, allTasks: Task[]) => number;
		getTaskStatusColor: (status: string) => string;
		getStatusBadgeColor: (status: string) => string;
		getPriorityColor: (priority: string) => string;
		getTaskIcon: (task: Task) => string;
	}

	let { 
		handleTaskSelect, 
		handleTaskStatusToggle, 
		handleTaskSelection,
		handleCreateSubtask,
		getSubtaskCount,
		getTaskStatusColor,
		getStatusBadgeColor,
		getPriorityColor,
		getTaskIcon
	}: Props = $props();

	// Drag and drop state
	let dragDropState = $state(createDragDropState());

	function getTaskSubtasks(taskId: string): Task[] {
		return $tasks.filter(task => task.parentId === taskId);
	}

	async function handleTaskMove(taskId: string, newStatus: string) {
		try {
			await taskActions.updateTask(taskId, { status: newStatus as any });
		} catch (error) {
			console.error('Failed to move task:', error);
		}
	}
</script>

		<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
	{#each $kanbanColumns as column}
		<div 
			class="space-y-3 {getDropZoneClasses(column.id, dragDropState, 'min-h-[200px] p-2 rounded-lg border-2 border-dashed border-transparent')}"
			role="region"
			aria-label="Drop zone for {column.title} tasks"
			ondragover={(e) => handleDragOver(e, column.id, dragDropState)}
			ondragleave={(e) => handleDragLeave(e, dragDropState)}
			ondrop={(e) => handleDrop(e, column.id, dragDropState, handleTaskMove)}
		>
			<div class="flex items-center justify-between">
				<h3 class="text-sm font-semibold text-foreground">{column.title}</h3>
				<span class="text-xs text-muted-foreground bg-muted px-2 py-1 rounded-full">
					{column.tasks.length}
				</span>
			</div>
			
			<div class="space-y-2">
				{#each column.tasks as task}
					<div
						draggable="true"
						ondragstart={(e) => handleDragStart(e, task, dragDropState)}
						ondragend={(e) => handleDragEnd(e, dragDropState)}
						onclick={() => $isMultiSelectMode ? handleTaskSelection(task.id) : handleTaskSelect(task)}
						onkeydown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								e.preventDefault();
								$isMultiSelectMode ? handleTaskSelection(task.id) : handleTaskSelect(task);
							}
						}}
						role="button"
						tabindex="0"
						aria-label="Select task: {task.title}"
						class="cursor-pointer hover:shadow-md transition-shadow {($selectedTaskIds.has(task.id)) ? 'ring-2 ring-warning-500 bg-warning-50 dark:bg-warning-900/20' : ''} {getTaskCardClasses(task, dragDropState)}"
					>
						<Card>
							<CardContent class="p-3">
								<div class="flex items-start justify-between mb-1">
									{#if $isMultiSelectMode}
										<input
											type="checkbox"
											checked={$selectedTaskIds.has(task.id)}
											onclick={(e) => e.stopPropagation()}
											class="mt-1 h-4 w-4 text-primary focus:ring-primary border-border rounded"
										/>
									{/if}
									<h4 class="font-medium text-foreground text-sm">{task.title}</h4>
									<div class="flex items-center gap-1">
										{#if getSubtaskCount(task.id, $tasks) > 0}
											<div class="flex items-center gap-1 text-xs text-muted-foreground">
												<Icon icon="mdi:subdirectory-arrow-right" class="w-3 h-3" />
												<span>{getSubtaskCount(task.id, $tasks)}</span>
											</div>
										{/if}
										<Icon 
											icon="mdi:flag" 
											class="w-3 h-3 {getPriorityColor(task.priority)}" 
										/>
										<span class="text-xs text-muted-foreground uppercase">{task.priority}</span>
									</div>
								</div>
								
								{#if task.description}
									<p class="text-xs text-muted-foreground mb-3 line-clamp-2">{task.description}</p>
								{/if}
								
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										{#if task.type}
											<Badge variant="outline" class="text-xs">
												{task.type}
											</Badge>
										{/if}
									</div>
									
									<div class="flex items-center gap-1">
										{#if !task.parentId}
											<Button
												onclick={(e) => { e.stopPropagation(); handleCreateSubtask(task); }}
												variant="ghost"
												size="sm"
												class="h-8 w-8 p-0"
												title="Create subtask"
											>
												<Icon 
													icon="mdi:plus" 
													class="w-4 h-4 text-muted-foreground" 
												/>
											</Button>
										{/if}
										<Button
											onclick={(e) => { e.stopPropagation(); handleTaskStatusToggle(task.id); }}
											variant="ghost"
											size="sm"
											class="h-8 w-8 p-0"
										>
											<Icon 
												icon={getTaskIcon(task)} 
												class="w-4 h-4 {getTaskStatusColor(task.status)}" 
											/>
										</Button>
									</div>
								</div>
							</CardContent>
						</Card>
						
						<!-- Subtasks in Kanban View -->
						{#if getTaskSubtasks(task.id).length > 0}
							<div class="ml-4 mt-2 space-y-2">
								{#each getTaskSubtasks(task.id) as subtask}
									<div 
										onclick={(e) => { e.stopPropagation(); handleTaskSelect(subtask); }}
										onkeydown={(e) => {
											if (e.key === 'Enter' || e.key === ' ') {
												e.preventDefault();
												e.stopPropagation();
												handleTaskSelect(subtask);
											}
										}}
										role="button"
										tabindex="0"
										aria-label="Select subtask: {subtask.title}"
										class="cursor-pointer hover:shadow-md transition-shadow"
									>
										<Card class="border-l-4 border-l-primary bg-primary/5 dark:bg-primary/10">
											<CardContent class="p-3">
												<div class="flex items-start justify-between mb-2">
													<div class="flex items-center gap-2">
														<Icon icon="mdi:subdirectory-arrow-right" class="w-3 h-3 text-primary flex-shrink-0" />
														<h4 class="font-medium text-foreground text-sm">{subtask.title}</h4>
													</div>
													<div class="flex items-center gap-1">
														<Icon 
															icon="mdi:flag" 
															class="w-3 h-3 {getPriorityColor(subtask.priority)}" 
														/>
														<span class="text-xs text-muted-foreground uppercase">{subtask.priority}</span>
													</div>
												</div>
												
												{#if subtask.description}
													<p class="text-xs text-muted-foreground mb-2 line-clamp-2">{subtask.description}</p>
												{/if}
												
												<div class="flex items-center justify-between">
													<div class="flex items-center gap-2">
														<Badge class="text-xs {getStatusBadgeColor(subtask.status)}">
															{subtask.status}
														</Badge>
													</div>
													
													<Button
														onclick={(e) => { e.stopPropagation(); handleTaskStatusToggle(subtask.id); }}
														variant="ghost"
														size="sm"
														class="h-8 w-8 p-0"
													>
														<Icon 
															icon={getTaskIcon(subtask)} 
															class="w-4 h-4 {getTaskStatusColor(subtask.status)}" 
														/>
													</Button>
												</div>
											</CardContent>
										</Card>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/each}
</div>
