<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';

	interface Props {
		onCreate?: (data: any) => void;
		onCancel?: () => void;
	}

	let { onCreate, onCancel }: Props = $props();

	let name = $state('');
	let description = $state('');
	let projectId = $state('');
	let port = $state(3000);

	function handleSubmit() {
		onCreate?.({
			name,
			description,
			project_id: projectId,
			port
		});
	}
</script>

<Card class="w-full max-w-md">
	<CardHeader>
		<CardTitle>Create New Deployment</CardTitle>
		<CardDescription>Set up a new deployment for your project</CardDescription>
	</CardHeader>
	<CardContent class="space-y-4">
		<div>
			<Label for="name">Name</Label>
			<Input id="name" bind:value={name} placeholder="My Deployment" />
		</div>
		
		<div>
			<Label for="description">Description</Label>
			<Textarea id="description" bind:value={description} placeholder="Optional description" />
		</div>
		
		<div>
			<Label for="projectId">Project ID</Label>
			<Input id="projectId" bind:value={projectId} placeholder="project-id" />
		</div>
		
		<div>
			<Label for="port">Port</Label>
			<Input id="port" type="number" bind:value={port} />
		</div>
		
		<div class="flex gap-2">
			<Button onclick={handleSubmit} disabled={!name || !projectId}>
				Create Deployment
			</Button>
			<Button variant="outline" onclick={onCancel}>
				Cancel
			</Button>
		</div>
	</CardContent>
</Card>
