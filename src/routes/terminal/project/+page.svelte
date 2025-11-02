<!--
	Terminal - Project Terminals Tab
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { projectStore } from '$lib/domains/projects';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { FolderOpen, Terminal, Plus } from 'lucide-svelte';

	// Reactive stores
	let projects = $derived($projectStore.projects);

	onMount(() => {
		// Initialize project store if needed
	});
</script>

<div class="h-full w-full p-6">
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h2 class="text-2xl font-bold">Project Terminals</h2>
				<p class="text-muted-foreground">Manage terminals for your projects</p>
			</div>
			<Button>
				<Plus class="h-4 w-4 mr-2" />
				New Project Terminal
			</Button>
		</div>

		<!-- Projects List -->
		{#if projects.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each projects as project}
					<Card class="hover:shadow-md transition-shadow">
						<CardHeader>
							<CardTitle class="flex items-center space-x-2">
								<FolderOpen class="h-5 w-5" />
								<span>{project.name}</span>
							</CardTitle>
							<CardDescription>{project.description || 'No description'}</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Path:</span>
									<code class="text-xs bg-muted px-2 py-1 rounded">
										{project.path}
									</code>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Status:</span>
									<Badge variant="default">
										Active
									</Badge>
								</div>
								<div class="flex items-center space-x-2 pt-2">
									<Button size="sm" variant="outline" class="flex-1">
										<Terminal class="h-4 w-4 mr-1" />
										Open Terminal
									</Button>
									<Button size="sm" variant="outline">
										<Plus class="h-4 w-4" />
									</Button>
								</div>
							</div>
						</CardContent>
					</Card>
				{/each}
			</div>
		{:else}
			<Card>
				<CardContent class="flex items-center justify-center h-32">
					<div class="text-center">
						<FolderOpen class="h-8 w-8 text-muted-foreground mx-auto mb-2" />
						<p class="text-muted-foreground">No projects found</p>
						<p class="text-sm text-muted-foreground">Create a project to get started</p>
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
</div>
