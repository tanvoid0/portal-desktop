<!--
	Terminal - Containerized Terminals Tab
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Container, Terminal, Plus, Play, Square } from '@lucide/svelte';
	import { containers, deploymentActions } from '$lib/domains/deployments/stores/deploymentStore';
	import { deploymentService } from '$lib/domains/deployments/services/deploymentService';
	import { ContainerStatus } from '$lib/domains/deployments/types';
	import { goto } from '$app/navigation';

	let containerList = $derived($containers);

	onMount(async () => {
		await deploymentActions.loadContainers();
	});

	function openTerminal(containerId: string) {
		goto(`/terminal?container=${containerId}`);
	}
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
		{#if containerList.length > 0}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each containerList as container}
					<Card class="hover:shadow-md transition-shadow">
						<CardHeader>
							<CardTitle class="flex items-center space-x-2">
								<Container class="h-5 w-5" />
								<span>{container.name}</span>
							</CardTitle>
							<CardDescription>{container.image || 'No image specified'}</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<span class="text-sm text-muted-foreground">Status:</span>
									<Badge variant={container.status === 'running' ? 'default' : 'secondary'}>
										{container.status}
									</Badge>
								</div>
								{#if container.ports && container.ports.length > 0}
									<div class="flex items-center justify-between">
										<span class="text-sm text-muted-foreground">Ports:</span>
										<div class="flex flex-wrap gap-1">
											{#each container.ports as port}
												<Badge variant="outline" class="text-xs">{port}</Badge>
											{/each}
										</div>
									</div>
								{/if}
								<div class="flex items-center space-x-2 pt-2">
									{#if container.status === 'running'}
										<Button size="sm" variant="outline" class="flex-1" onclick={() => openTerminal(container.id)}>
											<Terminal class="h-4 w-4 mr-1" />
											Open Terminal
										</Button>
									{:else}
										<Button size="sm" variant="outline" class="flex-1" onclick={async () => {
											await deploymentService.startContainer(container.id);
											await deploymentActions.loadContainers();
										}}>
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
					<div class="text-2xl font-bold">{containerList.length}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Running</CardTitle>
					<Play class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{containerList.filter(c => c.status === 'running').length}</div>
				</CardContent>
			</Card>
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">Stopped</CardTitle>
					<Square class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{containerList.filter(c => c.status === ContainerStatus.EXITED || c.status === ContainerStatus.DEAD).length}</div>
				</CardContent>
			</Card>
		</div>
	</div>
</div>
