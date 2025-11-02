<script lang="ts">
	import { Button } from '@/lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/lib/components/ui/dialog';
	import { taskActions, savedViews } from '../stores/taskStore';
	import type { TaskFilters, SavedView } from '../types';
	import Icon from '@iconify/svelte';

	interface Props {
		onViewChange?: (filters: TaskFilters) => void;
	}

	let { onViewChange }: Props = $props();

	let showSaveDialog = $state(false);
	let newViewName = $state('');
	let newViewDescription = $state('');

	function loadView(view: SavedView) {
		onViewChange?.(view.filters);
	}

	function deleteView(viewId: string) {
		taskActions.deleteSavedView(viewId);
	}

	function saveCurrentView() {
		if (!newViewName.trim()) return;
		
		// This would save the current filter state
		// For now, we'll use a placeholder
		const newView: SavedView = {
			id: crypto.randomUUID(),
			name: newViewName.trim(),
			description: newViewDescription.trim() || undefined,
			filters: {}, // Current filters would be passed here
			isDefault: false,
			createdAt: new Date(),
			updatedAt: new Date()
		};
		
		taskActions.saveView(newView.name, newView.description || '', newView.filters, newView.isDefault);
		newViewName = '';
		newViewDescription = '';
		showSaveDialog = false;
	}

	function setAsDefault(viewId: string) {
		taskActions.setDefaultView(viewId);
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<CardTitle class="text-lg">Saved Views</CardTitle>
			<Dialog bind:open={showSaveDialog}>
				<DialogTrigger>
					<Button variant="outline" size="sm">
						<Icon icon="mdi:plus" class="w-4 h-4 mr-1" />
						Save View
					</Button>
				</DialogTrigger>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Save Current View</DialogTitle>
						<DialogDescription>
							Save your current filter settings as a reusable view.
						</DialogDescription>
					</DialogHeader>
					<div class="space-y-4">
						<div class="space-y-2">
							<Label for="viewName">View Name</Label>
							<Input
								id="viewName"
								bind:value={newViewName}
								placeholder="e.g., My Daily Tasks"
							/>
						</div>
						<div class="space-y-2">
							<Label for="viewDescription">Description (optional)</Label>
							<Input
								id="viewDescription"
								bind:value={newViewDescription}
								placeholder="Brief description of this view"
							/>
						</div>
					</div>
					<DialogFooter>
						<Button variant="outline" onclick={() => showSaveDialog = false}>
							Cancel
						</Button>
						<Button onclick={saveCurrentView} disabled={!newViewName.trim()}>
							Save View
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	</CardHeader>
	<CardContent class="p-4 pt-0">
		{#if $savedViews.length === 0}
			<div class="text-center py-8 text-muted-foreground">
				<Icon icon="mdi:view-list" class="w-12 h-12 mx-auto mb-2" />
				<div class="text-sm">No saved views yet</div>
				<div class="text-xs">Create your first view to get started</div>
			</div>
		{:else}
			<div class="space-y-2">
				{#each $savedViews as view}
					<div class="flex items-center justify-between p-2 rounded-lg hover:bg-muted/50 transition-colors">
						<div class="flex items-center gap-3">
							<Button
								variant="ghost"
								class="justify-start flex-1 h-auto p-0"
								onclick={() => loadView(view)}
							>
								<div class="flex items-center gap-2">
									<Icon icon="mdi:view-list" class="w-4 h-4" />
									<div class="text-left">
										<div class="font-medium">{view.name}</div>
										{#if view.description}
											<div class="text-xs text-muted-foreground">{view.description}</div>
										{/if}
									</div>
								</div>
							</Button>
							{#if view.isDefault}
								<Badge variant="secondary" class="text-xs">Default</Badge>
							{/if}
						</div>
						
						<div class="flex items-center gap-1">
							{#if !view.isDefault}
								<Button
									variant="ghost"
									size="sm"
									onclick={() => setAsDefault(view.id)}
									title="Set as default"
								>
									<Icon icon="mdi:star-outline" class="w-4 h-4" />
								</Button>
							{/if}
							<Button
								variant="ghost"
								size="sm"
								onclick={() => deleteView(view.id)}
								title="Delete view"
								class="text-destructive hover:text-destructive"
							>
								<Icon icon="mdi:delete" class="w-4 h-4" />
							</Button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
