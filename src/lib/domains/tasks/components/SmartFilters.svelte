<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { taskActions, overdueTasks, dueTodayTasks, blockedTasks, unestimatedTasks } from '../stores/taskStore';
	import type { TaskFilters } from '../types';
	import Icon from '@iconify/svelte';

	interface Props {
		onFilterChange?: (filters: TaskFilters) => void;
	}

	let { onFilterChange }: Props = $props();

	// Pre-defined smart filters
	const smartFilters = [
		{
			id: 'overdue',
			name: 'Overdue Tasks',
			description: 'Tasks past their due date',
			icon: 'mdi:alert-circle',
			color: 'text-red-600',
			count: $overdueTasks.length,
			filter: { status: 'pending', dueDate: { before: new Date() } }
		},
		{
			id: 'due-today',
			name: 'Due Today',
			description: 'Tasks due today',
			icon: 'mdi:calendar-today',
			color: 'text-yellow-600',
			count: $dueTodayTasks.length,
			filter: { status: 'pending', dueDate: { today: true } }
		},
		{
			id: 'blocked',
			name: 'Blocked Tasks',
			description: 'Tasks that are blocked by dependencies',
			icon: 'mdi:lock',
			color: 'text-orange-600',
			count: $blockedTasks.length,
			filter: { status: 'pending', hasBlockers: true }
		},
		{
			id: 'unestimated',
			name: 'Unestimated',
			description: 'Tasks without time estimates',
			icon: 'mdi:clock-alert',
			color: 'text-blue-600',
			count: $unestimatedTasks.length,
			filter: { status: 'pending', estimatedTime: null }
		},
		{
			id: 'high-priority',
			name: 'High Priority',
			description: 'High priority tasks',
			icon: 'mdi:flag',
			color: 'text-red-600',
			count: 0, // This would be calculated
			filter: { status: 'pending', priority: 'high' }
		},
		{
			id: 'in-progress',
			name: 'In Progress',
			description: 'Tasks currently being worked on',
			icon: 'mdi:play-circle',
			color: 'text-green-600',
			count: 0, // This would be calculated
			filter: { status: 'in_progress' }
		}
	];

	function applyFilter(filter: any) {
		onFilterChange?.(filter.filter);
	}

	function clearFilters() {
		onFilterChange?.({});
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<CardTitle class="text-lg">Smart Filters</CardTitle>
			<Button variant="ghost" size="sm" onclick={clearFilters}>
				<Icon icon="mdi:close" class="w-4 h-4 mr-1" />
				Clear
			</Button>
		</div>
	</CardHeader>
	<CardContent class="p-4 pt-0">
		<div class="space-y-3">
			{#each smartFilters as filter}
				<Button
					variant="ghost"
					class="w-full justify-start h-auto p-3"
					onclick={() => applyFilter(filter)}
				>
					<div class="flex items-center w-full">
						<Icon icon={filter.icon} class="w-5 h-5 mr-3 {filter.color}" />
						<div class="flex-1 text-left">
							<div class="font-medium">{filter.name}</div>
							<div class="text-xs text-muted-foreground">{filter.description}</div>
						</div>
						{#if filter.count > 0}
							<Badge variant="secondary" class="ml-2">
								{filter.count}
							</Badge>
						{/if}
					</div>
				</Button>
			{/each}
		</div>
	</CardContent>
</Card>
