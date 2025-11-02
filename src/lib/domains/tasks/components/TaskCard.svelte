<script lang="ts">
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import { currentlyTracking } from '../stores/taskStore';
	import type { Task } from '../types';

	interface Props {
		task: Task;
		onClick?: () => void;
		onStatusToggle?: () => void;
		onCreateSubtask?: () => void;
		showCheckbox?: boolean;
		isSelected?: boolean;
		onSelectionChange?: () => void;
	}

	let { 
		task, 
		onClick, 
		onStatusToggle, 
		onCreateSubtask,
		showCheckbox = false, 
		isSelected = false, 
		onSelectionChange 
	}: Props = $props();

	function getTaskStatusColor(status: string) {
		switch (status) {
			case 'completed': return 'text-green-500';
			case 'in-progress': return 'text-blue-500';
			case 'cancelled': return 'text-red-500';
			default: return 'text-gray-400';
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

	function getSubtaskCount(taskId: string): number {
		// This would need to be passed from parent or computed
		return 0;
	}

	// New helper functions for advanced features
	function isOverdue(task: Task): boolean {
		return !!task.dueDate && task.dueDate < new Date() && task.status !== 'completed' && task.status !== 'cancelled';
	}

	function isDueToday(task: Task): boolean {
		if (!task.dueDate) return false;
		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const tomorrow = new Date(today);
		tomorrow.setDate(tomorrow.getDate() + 1);
		return task.dueDate >= today && task.dueDate < tomorrow;
	}

	function isDueThisWeek(task: Task): boolean {
		if (!task.dueDate) return false;
		const today = new Date();
		const weekFromNow = new Date(today);
		weekFromNow.setDate(weekFromNow.getDate() + 7);
		return task.dueDate >= today && task.dueDate <= weekFromNow;
	}

	function isCurrentlyTracking(task: Task): boolean {
		return $currentlyTracking?.taskId === task.id;
	}

	function getTimeEstimateText(task: Task): string {
		if (!task.estimatedTime) return '';
		const hours = Math.floor(task.estimatedTime / 60);
		const minutes = task.estimatedTime % 60;
		if (hours > 0) {
			return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
		}
		return `${minutes}m`;
	}

	function getActualTimeText(task: Task): string {
		if (!task.actualTime) return '';
		const hours = Math.floor(task.actualTime / 60);
		const minutes = task.actualTime % 60;
		if (hours > 0) {
			return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
		}
		return `${minutes}m`;
	}

	function isBlocked(task: Task): boolean {
		return !!task.blockedBy && task.blockedBy.length > 0;
	}

	function isBlocking(task: Task): boolean {
		return !!task.blocks && task.blocks.length > 0;
	}
</script>

<Card 
	class="cursor-pointer hover:shadow-lg hover:scale-[1.02] transition-all duration-200 ease-out {isSelected ? 'ring-2 ring-warning-500 bg-warning-50 dark:bg-warning-900/20 shadow-lg' : 'hover:shadow-md'}"
	onclick={onClick}
>
	<CardContent class="p-4">
		<div class="flex items-start justify-between mb-2">
			{#if showCheckbox}
				<input
					type="checkbox"
					checked={isSelected}
					onclick={(e) => { e.stopPropagation(); onSelectionChange?.(); }}
					class="mt-1 h-4 w-4 text-primary focus:ring-primary border-border rounded"
				/>
			{/if}
			<div class="flex-1">
				<div class="flex items-center gap-2 mb-1">
					<h4 class="font-medium text-foreground text-sm">{task.title}</h4>
					
					<!-- Due Date Indicators -->
					{#if isOverdue(task)}
						<Badge variant="destructive" class="text-xs">
							<Icon icon="mdi:alert-circle" class="w-3 h-3 mr-1" />
							Overdue
						</Badge>
					{:else if isDueToday(task)}
						<Badge variant="secondary" class="text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300">
							<Icon icon="mdi:clock-alert" class="w-3 h-3 mr-1" />
							Due Today
						</Badge>
					{:else if isDueThisWeek(task)}
						<Badge variant="outline" class="text-xs">
							<Icon icon="mdi:calendar-week" class="w-3 h-3 mr-1" />
							This Week
						</Badge>
					{/if}
					
					<!-- Time Tracking Indicator -->
					{#if isCurrentlyTracking(task)}
						<Badge variant="secondary" class="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300">
							<Icon icon="mdi:timer" class="w-3 h-3 mr-1" />
							Tracking
						</Badge>
					{/if}
					
					<!-- Dependency Indicators -->
					{#if isBlocked(task)}
						<Badge variant="outline" class="text-xs border-red-200 text-red-600 dark:border-red-800 dark:text-red-400">
							<Icon icon="mdi:lock" class="w-3 h-3 mr-1" />
							Blocked
						</Badge>
					{/if}
					
					{#if isBlocking(task)}
						<Badge variant="outline" class="text-xs border-orange-200 text-orange-600 dark:border-orange-800 dark:text-orange-400">
							<Icon icon="mdi:lock-open" class="w-3 h-3 mr-1" />
							Blocking
						</Badge>
					{/if}
				</div>
				
				<!-- Tags -->
				{#if task.tags && task.tags.length > 0}
					<div class="flex flex-wrap gap-1 mb-2">
						{#each task.tags as tag}
							<Badge variant="outline" class="text-xs">
								<Icon icon="mdi:tag" class="w-3 h-3 mr-1" />
								{tag}
							</Badge>
						{/each}
					</div>
				{/if}
			</div>
			
			<div class="flex items-center gap-1">
				{#if getSubtaskCount(task.id) > 0}
					<div class="flex items-center gap-1 text-xs text-muted-foreground">
						<Icon icon="mdi:subdirectory-arrow-right" class="w-3 h-3" />
						<span>{getSubtaskCount(task.id)}</span>
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
				
				<!-- Time Estimates -->
				{#if task.estimatedTime || task.actualTime}
					<div class="flex items-center gap-2 text-xs text-muted-foreground">
						{#if task.estimatedTime}
							<div class="flex items-center gap-1">
								<Icon icon="mdi:clock-outline" class="w-3 h-3" />
								<span>{getTimeEstimateText(task)}</span>
							</div>
						{/if}
						{#if task.actualTime}
							<div class="flex items-center gap-1">
								<Icon icon="mdi:clock-check" class="w-3 h-3" />
								<span>{getActualTimeText(task)}</span>
							</div>
						{/if}
					</div>
				{/if}
				
				<!-- Comments and Attachments Count -->
				{#if task.comments && task.comments.length > 0}
					<div class="flex items-center gap-1 text-xs text-muted-foreground">
						<Icon icon="mdi:comment-outline" class="w-3 h-3" />
						<span>{task.comments.length}</span>
					</div>
				{/if}
				
				{#if task.attachments && task.attachments.length > 0}
					<div class="flex items-center gap-1 text-xs text-muted-foreground">
						<Icon icon="mdi:paperclip" class="w-3 h-3" />
						<span>{task.attachments.length}</span>
					</div>
				{/if}
			</div>
			
			<div class="flex items-center gap-1">
				{#if onCreateSubtask && !task.parentId}
					<Button
						onclick={(e) => { e.stopPropagation(); onCreateSubtask(); }}
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
					onclick={(e) => { e.stopPropagation(); onStatusToggle?.(); }}
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
