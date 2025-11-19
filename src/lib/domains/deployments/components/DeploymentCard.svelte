<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Button } from '@/lib/components/ui/button';
	import type { Deployment } from '../types';
	import { DeploymentStatus, DeploymentType } from '../types';
	import { Container, Terminal } from '@lucide/svelte';

	interface Props {
		deployment: Deployment;
		onStart?: (id: string) => void;
		onStop?: (id: string) => void;
		onDelete?: (id: string) => void;
	}

	let { deployment, onStart, onStop, onDelete }: Props = $props();

	function getStatusColor(status: DeploymentStatus): string {
		switch (status) {
			case DeploymentStatus.RUNNING:
				return 'bg-green-100 text-green-800';
			case DeploymentStatus.STOPPED:
				return 'bg-gray-100 text-gray-800';
			case DeploymentStatus.BUILDING:
			case DeploymentStatus.CREATING:
				return 'bg-yellow-100 text-yellow-800';
			case DeploymentStatus.FAILED:
				return 'bg-red-100 text-red-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				{#if deployment.type === DeploymentType.DOCKER}
					<Container class="h-5 w-5 text-muted-foreground" />
				{:else}
					<Terminal class="h-5 w-5 text-muted-foreground" />
				{/if}
				<div>
					<CardTitle class="text-lg">{deployment.name}</CardTitle>
					<CardDescription>{deployment.description || 'No description'}</CardDescription>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Badge variant="outline">
					{deployment.type === DeploymentType.DOCKER ? 'Docker' : 'CLI'}
				</Badge>
				<Badge class={getStatusColor(deployment.status)}>
					{deployment.status}
				</Badge>
			</div>
		</div>
	</CardHeader>
	<CardContent>
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="text-sm text-muted-foreground">Project Path:</span>
				<span class="text-sm font-mono truncate max-w-[200px]" title={deployment.projectPath}>
					{deployment.projectPath}
				</span>
			</div>
			<div class="flex items-center justify-between">
				<span class="text-sm text-muted-foreground">Project Type:</span>
				<span class="text-sm">{deployment.projectType}</span>
			</div>
			{#if deployment.type === DeploymentType.DOCKER && deployment.dockerImageName}
				<div class="flex items-center justify-between">
					<span class="text-sm text-muted-foreground">Image:</span>
					<span class="text-sm font-mono truncate max-w-[200px]" title={deployment.dockerImageName}>
						{deployment.dockerImageName}
					</span>
				</div>
				{#if deployment.container?.ports && deployment.container.ports.length > 0}
					<div class="flex items-center justify-between">
						<span class="text-sm text-muted-foreground">Port:</span>
						<span class="text-sm">{deployment.container.ports[0].hostPort}:{deployment.container.ports[0].containerPort}</span>
					</div>
				{/if}
			{:else if deployment.type === DeploymentType.CLI && deployment.command}
				<div class="flex items-center justify-between">
					<span class="text-sm text-muted-foreground">Command:</span>
					<span class="text-sm font-mono truncate max-w-[200px]" title={deployment.command}>
						{deployment.command}
					</span>
				</div>
				{#if deployment.processId}
					<div class="flex items-center justify-between">
						<span class="text-sm text-muted-foreground">PID:</span>
						<span class="text-sm">{deployment.processId}</span>
					</div>
				{/if}
			{/if}
			<div class="flex items-center justify-between">
				<span class="text-sm text-muted-foreground">Created:</span>
				<span class="text-sm">{new Date(deployment.createdAt).toLocaleDateString()}</span>
			</div>
		</div>
		
		<div class="flex gap-2 mt-4">
			{#if deployment.status === DeploymentStatus.RUNNING}
				<Button variant="outline" size="sm" onclick={() => onStop?.(deployment.id)}>
					Stop
				</Button>
			{:else if deployment.status !== DeploymentStatus.BUILDING && deployment.status !== DeploymentStatus.CREATING}
				<Button variant="default" size="sm" onclick={() => onStart?.(deployment.id)}>
					Start
				</Button>
			{/if}
			<Button variant="destructive" size="sm" onclick={() => onDelete?.(deployment.id)}>
				Delete
			</Button>
		</div>
	</CardContent>
</Card>
