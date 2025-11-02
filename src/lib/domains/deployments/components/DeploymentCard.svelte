<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Button } from '@/lib/components/ui/button';
	import type { Deployment } from '../types';
	import { DeploymentStatus } from '../types';

	interface Props {
		deployment: Deployment;
		onStart?: (id: string) => void;
		onStop?: (id: string) => void;
		onDelete?: (id: string) => void;
	}

	let { deployment, onStart, onStop, onDelete }: Props = $props();
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle class="text-lg">{deployment.name}</CardTitle>
				<CardDescription>{deployment.description || 'No description'}</CardDescription>
			</div>
			<Badge variant={deployment.status === DeploymentStatus.RUNNING ? 'default' : 'secondary'}>
				{deployment.status}
			</Badge>
		</div>
	</CardHeader>
	<CardContent>
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="text-sm text-muted-foreground">Project Path:</span>
				<span class="text-sm font-mono">{deployment.projectPath}</span>
			</div>
			<div class="flex items-center justify-between">
				<span class="text-sm text-muted-foreground">Type:</span>
				<span class="text-sm">{deployment.projectType}</span>
			</div>
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
			{:else}
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
