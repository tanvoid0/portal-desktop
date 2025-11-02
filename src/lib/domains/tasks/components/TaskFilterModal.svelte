<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Input } from '@/lib/components/ui/input';
	import { Badge } from '@/lib/components/ui/badge';
	import { Checkbox } from '@/lib/components/ui/checkbox';
	import Select from '@/lib/components/ui/select.svelte';
	import Icon from '@iconify/svelte';
	import type { TaskStatus, TaskPriority } from '../types';

	interface Props {
		open: boolean;
		searchQuery: string;
		statusFilters: TaskStatus[];
		priorityFilters: TaskPriority[];
		typeFilters: string[];
		onSearchChange: (value: string) => void;
		onStatusFilterChange: (filters: TaskStatus[]) => void;
		onPriorityFilterChange: (filters: TaskPriority[]) => void;
		onTypeFilterChange: (filters: string[]) => void;
		onClearAll: () => void;
	}

	let { 
		open = $bindable(false),
		searchQuery,
		statusFilters,
		priorityFilters,
		typeFilters,
		onSearchChange,
		onStatusFilterChange,
		onPriorityFilterChange,
		onTypeFilterChange,
		onClearAll
	}: Props = $props();

	let localSearchQuery = $state(searchQuery);
	let localStatusFilters = $state([...statusFilters]);
	let localPriorityFilters = $state([...priorityFilters]);
	let localTypeFilters = $state([...typeFilters]);

	// Available filter options
	const statusOptions: { value: TaskStatus; label: string; icon: string }[] = [
		{ value: 'pending', label: 'To Do', icon: 'mdi:clock-outline' },
		{ value: 'in-progress', label: 'In Progress', icon: 'mdi:progress-clock' },
		{ value: 'completed', label: 'Completed', icon: 'mdi:check-circle' },
		{ value: 'cancelled', label: 'Cancelled', icon: 'mdi:cancel' }
	];

	const priorityOptions: { value: TaskPriority; label: string; icon: string; color: string }[] = [
		{ value: 'low', label: 'Low', icon: 'mdi:flag', color: 'text-green-500' },
		{ value: 'medium', label: 'Medium', icon: 'mdi:flag', color: 'text-yellow-500' },
		{ value: 'high', label: 'High', icon: 'mdi:flag', color: 'text-red-500' }
	];

	const typeOptions: { value: string; label: string }[] = [
		{ value: 'Story', label: 'Story' },
		{ value: 'Bug', label: 'Bug' },
		{ value: 'Feature', label: 'Feature' },
		{ value: 'Note', label: 'Note' },
		{ value: 'Task', label: 'Task' },
		{ value: 'Epic', label: 'Epic' }
	];

	function toggleStatusFilter(status: TaskStatus) {
		if (localStatusFilters.includes(status)) {
			localStatusFilters = localStatusFilters.filter(s => s !== status);
		} else {
			localStatusFilters = [...localStatusFilters, status];
		}
	}

	function togglePriorityFilter(priority: TaskPriority) {
		if (localPriorityFilters.includes(priority)) {
			localPriorityFilters = localPriorityFilters.filter(p => p !== priority);
		} else {
			localPriorityFilters = [...localPriorityFilters, priority];
		}
	}

	function toggleTypeFilter(type: string) {
		if (localTypeFilters.includes(type)) {
			localTypeFilters = localTypeFilters.filter(t => t !== type);
		} else {
			localTypeFilters = [...localTypeFilters, type];
		}
	}

	// Remove the old group filter function

	function applyFilters() {
		onSearchChange(localSearchQuery);
		onStatusFilterChange(localStatusFilters);
		onPriorityFilterChange(localPriorityFilters);
		onTypeFilterChange(localTypeFilters);
		open = false;
	}

	function clearAllFilters() {
		localSearchQuery = '';
		localStatusFilters = [];
		localPriorityFilters = [];
		localTypeFilters = [];
		onClearAll();
		open = false;
	}

	function handleClose() {
		// Reset local state to current props
		localSearchQuery = searchQuery;
		localStatusFilters = [...statusFilters];
		localPriorityFilters = [...priorityFilters];
		localTypeFilters = [...typeFilters];
		open = false;
	}
</script>

{#if open}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<Card class="w-full max-w-2xl max-h-[80vh] overflow-hidden">
			<CardHeader class="pb-3">
				<div class="flex items-center justify-between">
					<CardTitle class="text-lg">Advanced Filters</CardTitle>
					<Button
						variant="ghost"
						size="sm"
						onclick={handleClose}
						class="h-8 w-8 p-0"
					>
						<Icon icon="mdi:close" class="w-4 h-4" />
					</Button>
				</div>
			</CardHeader>
			
			<CardContent class="space-y-6 overflow-y-auto">
				<!-- Search -->
				<div>
					<label class="block text-sm font-medium text-foreground mb-2">Search</label>
					<div class="relative">
						<Icon icon="mdi:magnify" class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
						<Input
							type="text"
							placeholder="Search tasks..."
							bind:value={localSearchQuery}
							class="pl-10"
						/>
					</div>
				</div>

				<!-- Status Filters -->
				<div>
					<label class="block text-sm font-medium text-foreground mb-3">Status</label>
					<div class="grid grid-cols-2 gap-2">
						{#each statusOptions as option}
							<label class="flex items-center space-x-2 cursor-pointer p-2 rounded-md hover:bg-muted/50">
								<Checkbox
									checked={localStatusFilters.includes(option.value)}
									onCheckedChange={() => toggleStatusFilter(option.value)}
								/>
								<Icon icon={option.icon} class="w-4 h-4 text-muted-foreground" />
								<span class="text-sm">{option.label}</span>
							</label>
						{/each}
					</div>
				</div>

				<!-- Priority Filters -->
				<div>
					<label class="block text-sm font-medium text-foreground mb-3">Priority</label>
					<div class="grid grid-cols-3 gap-2">
						{#each priorityOptions as option}
							<label class="flex items-center space-x-2 cursor-pointer p-2 rounded-md hover:bg-muted/50">
								<Checkbox
									checked={localPriorityFilters.includes(option.value)}
									onCheckedChange={() => togglePriorityFilter(option.value)}
								/>
								<Icon icon={option.icon} class="w-4 h-4 {option.color}" />
								<span class="text-sm">{option.label}</span>
							</label>
						{/each}
					</div>
				</div>

				<!-- Type Filters -->
				<div>
					<label class="block text-sm font-medium text-foreground mb-3">Types</label>
					<div class="grid grid-cols-2 gap-2">
						{#each typeOptions as typeOption}
							<label class="flex items-center space-x-2 cursor-pointer p-2 rounded-md hover:bg-muted/50">
								<Checkbox
									checked={localTypeFilters.includes(typeOption.value)}
									onCheckedChange={() => toggleTypeFilter(typeOption.value)}
								/>
								<span class="text-sm">{typeOption.label}</span>
							</label>
						{/each}
					</div>
				</div>

				<!-- Action Buttons -->
				<div class="flex items-center justify-between pt-4 border-t border-border">
					<Button
						variant="outline"
						onclick={clearAllFilters}
						class="text-red-600 hover:text-red-700"
					>
						<Icon icon="mdi:filter-remove" class="w-4 h-4 mr-2" />
						Clear All
					</Button>
					
					<div class="flex items-center gap-2">
						<Button variant="outline" onclick={handleClose}>
							Cancel
						</Button>
						<Button onclick={applyFilters}>
							Apply Filters
						</Button>
					</div>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}
