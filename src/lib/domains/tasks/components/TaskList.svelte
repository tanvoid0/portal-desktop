<script lang="ts">
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import { filteredTasks, tasks, selectedTaskIds, isMultiSelectMode } from '../stores/taskStore';
	import type { Task } from '../types';

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

	function getTaskSubtasks(taskId: string): Task[] {
		return $tasks.filter(task => task.parentId === taskId);
	}
</script>

<Card>
	<CardContent class="p-0">
		<div class="divide-y divide-border">
			{#each $filteredTasks as task}
				<div
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
					class="p-3 hover:bg-muted/50 cursor-pointer transition-colors {$selectedTaskIds.has(task.id) ? 'bg-warning-50 dark:bg-warning-900/20' : ''}"
				>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							{#if $isMultiSelectMode}
								<input
									type="checkbox"
									checked={$selectedTaskIds.has(task.id)}
									onclick={(e) => e.stopPropagation()}
									class="h-4 w-4 text-primary focus:ring-primary border-border rounded"
								/>
							{/if}
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
										class="w-5 h-5 {getTaskStatusColor(task.status)}" 
									/>
								</Button>
							</div>
							
							<div>
								<div class="flex items-center gap-2">
									<h4 class="font-medium text-foreground">{task.title}</h4>
									{#if getSubtaskCount(task.id, $tasks) > 0}
										<div class="flex items-center gap-1 text-xs text-muted-foreground">
											<Icon icon="mdi:subdirectory-arrow-right" class="w-3 h-3" />
											<span>{getSubtaskCount(task.id, $tasks)}</span>
										</div>
									{/if}
								</div>
								{#if task.description}
									<p class="text-sm text-muted-foreground mt-1">{task.description}</p>
								{/if}
							</div>
						</div>
						
						<div class="flex items-center gap-3">
							{#if task.type}
								<Badge variant="outline" class="text-xs">
									{task.type}
								</Badge>
							{/if}
							
							<div class="flex items-center gap-1">
								<Icon 
									icon="mdi:flag" 
									class="w-4 h-4 {getPriorityColor(task.priority)}" 
								/>
								<Badge variant="outline" class="text-xs uppercase">
									{task.priority}
								</Badge>
							</div>
							
							<Badge class="text-sm {getStatusBadgeColor(task.status)}">
								{task.status}
							</Badge>
						</div>
					</div>
					
					<!-- Subtasks in List View -->
					{#if getTaskSubtasks(task.id).length > 0}
						<div class="ml-6 space-y-1">
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
									class="p-3 hover:bg-muted/50 cursor-pointer transition-colors border-l-4 border-l-primary bg-primary/5 dark:bg-primary/10"
								>
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-3">
											<Button
												onclick={(e) => { e.stopPropagation(); handleTaskStatusToggle(subtask.id); }}
												variant="ghost"
												size="sm"
												class="h-8 w-8 p-0"
											>
												<Icon 
													icon={getTaskIcon(subtask)} 
													class="w-5 h-5 {getTaskStatusColor(subtask.status)}" 
												/>
											</Button>
											
											<div>
												<div class="flex items-center gap-2">
													<Icon icon="mdi:subdirectory-arrow-right" class="w-3 h-3 text-primary flex-shrink-0" />
													<h4 class="font-medium text-foreground">{subtask.title}</h4>
												</div>
												{#if subtask.description}
													<p class="text-sm text-muted-foreground mt-1">{subtask.description}</p>
												{/if}
											</div>
										</div>
										
										<div class="flex items-center gap-3">
											{#if subtask.type}
												<Badge variant="outline" class="text-xs">
													{subtask.type}
												</Badge>
											{/if}
											
											<div class="flex items-center gap-1">
												<Icon 
													icon="mdi:flag" 
													class="w-4 h-4 {getPriorityColor(subtask.priority)}" 
												/>
												<Badge variant="outline" class="text-xs uppercase">
													{subtask.priority}
												</Badge>
											</div>
											
											<Badge class="text-sm {getStatusBadgeColor(subtask.status)}">
												{subtask.status}
											</Badge>
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</CardContent>
</Card>
