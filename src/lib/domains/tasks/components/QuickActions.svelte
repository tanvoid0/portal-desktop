<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { taskActions, currentlyTracking, productivityMetrics } from '../stores/taskStore';
	import type { Task } from '../types';
	import Icon from '@iconify/svelte';

	interface Props {
		onQuickAdd?: () => void;
		onStartTracking?: (taskId: string) => void;
		onStopTracking?: () => void;
	}

	let { onQuickAdd, onStartTracking, onStopTracking }: Props = $props();

	// Quick action handlers
	function handleQuickAdd() {
		onQuickAdd?.();
	}

	function handleStartTracking() {
		if ($currentlyTracking) {
			onStartTracking?.($currentlyTracking.taskId);
		}
	}

	function handleStopTracking() {
		onStopTracking?.();
	}

	function handleQuickComplete() {
		// Find a quick task to complete
		// This would be implemented based on your business logic
		console.log('Quick complete task');
	}

	function handleQuickTemplate() {
		// Apply a common template
		console.log('Apply quick template');
	}
</script>

<Card class="w-full">
	<CardContent class="p-4">
		<div class="space-y-4">
			<!-- Header -->
			<div class="flex items-center justify-between">
				<h3 class="text-lg font-semibold">Quick Actions</h3>
				{#if $currentlyTracking}
					<Badge variant="secondary" class="bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300">
						<Icon icon="mdi:timer" class="w-3 h-3 mr-1" />
						Tracking
					</Badge>
				{/if}
			</div>

			<Separator />

			<!-- Quick Action Buttons -->
			<div class="grid grid-cols-2 gap-2">
				<Button 
					variant="default" 
					onclick={handleQuickAdd}
					class="h-12 flex flex-col items-center gap-1"
				>
					<Icon icon="mdi:plus" class="w-5 h-5" />
					<span class="text-xs">Quick Add</span>
				</Button>

				{#if $currentlyTracking}
					<Button 
						variant="destructive" 
						onclick={handleStopTracking}
						class="h-12 flex flex-col items-center gap-1"
					>
						<Icon icon="mdi:stop" class="w-5 h-5" />
						<span class="text-xs">Stop Timer</span>
					</Button>
				{:else}
					<Button 
						variant="secondary" 
						onclick={handleStartTracking}
						class="h-12 flex flex-col items-center gap-1"
					>
						<Icon icon="mdi:play" class="w-5 h-5" />
						<span class="text-xs">Start Timer</span>
					</Button>
				{/if}

				<Button 
					variant="outline" 
					onclick={handleQuickComplete}
					class="h-12 flex flex-col items-center gap-1"
				>
					<Icon icon="mdi:check-circle" class="w-5 h-5" />
					<span class="text-xs">Quick Complete</span>
				</Button>

				<Button 
					variant="outline" 
					onclick={handleQuickTemplate}
					class="h-12 flex flex-col items-center gap-1"
				>
					<Icon icon="mdi:template" class="w-5 h-5" />
					<span class="text-xs">Template</span>
				</Button>
			</div>

			<!-- Productivity Metrics -->
			{#if $productivityMetrics}
				<div class="space-y-2">
					<h4 class="text-sm font-medium text-muted-foreground">Today's Progress</h4>
					<div class="grid grid-cols-2 gap-2 text-xs">
						<div class="flex justify-between">
							<span>Completion Rate:</span>
							<span class="font-medium">{Math.round($productivityMetrics.completionRate * 100)}%</span>
						</div>
						<div class="flex justify-between">
							<span>Time Accuracy:</span>
							<span class="font-medium">{Math.round($productivityMetrics.timeAccuracy * 100)}%</span>
						</div>
						<div class="flex justify-between">
							<span>Velocity:</span>
							<span class="font-medium">{$productivityMetrics.velocity} tasks</span>
						</div>
						<div class="flex justify-between">
							<span>Overdue:</span>
							<span class="font-medium text-destructive">{$productivityMetrics.overdueCount}</span>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</CardContent>
</Card>
