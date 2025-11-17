<!--
	Terminal - Containerized Terminals Tab
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Container, Terminal, Plus, Play, Square } from '@lucide/svelte';

	// Mock container data - in real implementation, this would come from a store
	let containers = $state([
		{
			id: '1',
			name: 'web-server',
			image: 'nginx:latest',
			status: 'running',
			ports: ['80:80', '443:443'],
			created: '2024-01-15T10:30:00Z'
		},
		{
			id: '2',
			name: 'database',
			image: 'postgres:15',
			status: 'stopped',
			ports: ['5432:5432'],
			created: '2024-01-14T15:45:00Z'
		},
		{
			id: '3',
			name: 'redis-cache',
			image: 'redis:7-alpine',
			status: 'running',
			ports: ['6379:6379'],
			created: '2024-01-16T09:15:00Z'
		}
	]);

	onMount(() => {
		// Initialize container data
	});
</script>

<div class="h-full w-full p-6">
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h2 class="text-2xl font-bold">Containerized Terminals</h2>
				<p class="text-muted-foreground">Manage terminals for Docker containers</p>
			</div>
			<Button>
				<Plus class="h-4 w-4 mr-2" />
				New Container Terminal
			</Button>
		</div>

		<!-- Containers List -->
		{#if containers.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each containers as container}
					<Card class="hover:shadow-md transition-shadow">
						<CardHeader>
							<CardTitle class="flex items-center space-x-2">
								<Container class="h-5 w-5" />
								<span>{container.name}</span>
							</CardTitle>
							<CardDescription>{container.image}</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Status:</span>
									<Badge variant={container.status === 'running' ? 'default' : 'secondary'}>
										{container.status}
									</Badge>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Ports:</span>
									<div class="flex flex-wrap gap-1">
										{#each container.ports as port}
											<Badge variant="outline" class="text-xs">{port}</Badge>
										{/each}
									</div>
								</div>
								<div class="flex items-center space-x-2 pt-2">
									{#if container.status === 'running'}
										<Button size="sm" variant="outline" class="flex-1">
											<Terminal class="h-4 w-4 mr-1" />
											Open Terminal
										</Button>
										<Button size="sm" variant="outline" class="text-red-600 hover:text-red-700">
											<Square class="h-4 w-4" />
										</Button>
									{:else}
										<Button size="sm" variant="outline" class="flex-1">
											<Play class="h-4 w-4 mr-1" />
											Start Container
										</Button>
									{/if}
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
						<Container class="h-8 w-8 text-muted-foreground mx-auto mb-2" />
						<p class="text-muted-foreground">No containers found</p>
						<p class="text-sm text-muted-foreground">Start a container to get started</p>
					</div>
				</CardContent>
			</Card>
		{/if}

		<!-- Container Statistics -->
		<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Total Containers</CardTitle>
					<Container class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{containers.length}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Running</CardTitle>
					<Play class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{containers.filter(c => c.status === 'running').length}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Stopped</CardTitle>
					<Square class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{containers.filter(c => c.status === 'stopped').length}</div>
				</CardContent>
			</Card>
		</div>
	</div>
</div>
