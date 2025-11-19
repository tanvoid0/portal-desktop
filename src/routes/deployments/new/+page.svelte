<!--
	Create New Deployment Page
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import Select from '$lib/components/ui/select.svelte';
	import { DeploymentType, ProjectType, type DeploymentCreateRequest } from '$lib/domains/deployments/types';
	import { deploymentActions } from '$lib/domains/deployments/stores/deploymentStore';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import { logger } from '$lib/domains/shared';
	import { ArrowLeft, Loader2 } from '@lucide/svelte';

	let deploymentType = $state<DeploymentType>(DeploymentType.DOCKER);
	let name = $state('');
	let description = $state('');
	let projectPath = $state('');
	let projectType = $state<ProjectType>(ProjectType.NODE);
	let sdkVersion = $state('latest');
	
	// Docker-specific fields
	let dockerImageName = $state('');
	let dockerfilePath = $state('');
	let exposedPort = $state<number | undefined>(3000);
	
	// CLI-specific fields
	let command = $state('');
	let workingDirectory = $state('');
	
	// Environment variables
	let envVars = $state<Record<string, string>>({});
	let isSubmitting = $state(false);

	async function handleSubmit() {
		if (!name || !projectPath) {
			toast.error('Name and project path are required');
			return;
		}

		// Validate type-specific fields
		if (deploymentType === DeploymentType.CLI && !command) {
			toast.error('Command is required for CLI deployments');
			return;
		}

		isSubmitting = true;

		try {
			const request: DeploymentCreateRequest = {
				name,
				description: description || undefined,
				type: deploymentType,
				projectPath,
				projectType,
				environment: {
					variables: envVars,
					secrets: [],
					volumes: [],
					ports: [],
					networks: []
				},
				// Docker-specific
				dockerImageName: deploymentType === DeploymentType.DOCKER ? (dockerImageName || undefined) : undefined,
				dockerfilePath: deploymentType === DeploymentType.DOCKER ? (dockerfilePath || undefined) : undefined,
				exposedPort: deploymentType === DeploymentType.DOCKER ? exposedPort : undefined,
				// CLI-specific
				command: deploymentType === DeploymentType.CLI ? command : undefined,
				workingDirectory: deploymentType === DeploymentType.CLI ? (workingDirectory || projectPath) : undefined,
			};

			await deploymentActions.createDeployment(request);
			toast.success('Deployment created successfully');
			goto('/deployments');
		} catch (error) {
			logger.error('Failed to create deployment', { context: 'CreateDeploymentPage', error });
			toast.error(error instanceof Error ? error.message : 'Failed to create deployment');
		} finally {
			isSubmitting = false;
		}
	}

	function addEnvVar() {
		envVars = { ...envVars, '': '' };
	}

	function removeEnvVar(key: string) {
		const newEnvVars = { ...envVars };
		delete newEnvVars[key];
		envVars = newEnvVars;
	}

	function updateEnvVar(oldKey: string, newKey: string, value: string) {
		const newEnvVars = { ...envVars };
		if (oldKey !== newKey) {
			delete newEnvVars[oldKey];
		}
		newEnvVars[newKey] = value;
		envVars = newEnvVars;
	}
</script>

<svelte:head>
	<title>Create Deployment - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6 max-w-4xl">
	<!-- Header -->
	<div class="mb-6">
		<Button variant="ghost" onclick={() => goto('/deployments')} class="mb-4">
			<ArrowLeft class="h-4 w-4 mr-2" />
			Back to Deployments
		</Button>
		<h1 class="text-3xl font-bold tracking-tight">Create New Deployment</h1>
		<p class="text-muted-foreground mt-2">
			Set up a new deployment for your project
		</p>
	</div>

	<Card>
		<CardHeader>
			<CardTitle>Deployment Configuration</CardTitle>
			<CardDescription>Configure your deployment settings</CardDescription>
		</CardHeader>
		<CardContent>
			<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-6">
				<!-- Deployment Type -->
				<div>
					<Label for="deploymentType">Deployment Type *</Label>
					<Select
						options={[
							{ value: DeploymentType.DOCKER, label: 'Docker' },
							{ value: DeploymentType.CLI, label: 'CLI Command' }
						]}
						defaultValue={deploymentType}
						onSelect={(value) => {
							deploymentType = value as DeploymentType;
						}}
					/>
				</div>

				<!-- Basic Fields -->
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<div>
						<Label for="name">Name *</Label>
						<Input id="name" bind:value={name} placeholder="My Deployment" required />
					</div>

					<div>
						<Label for="projectType">Project Type *</Label>
						<Select
							options={Object.values(ProjectType)}
							defaultValue={projectType}
							onSelect={(value) => {
								projectType = value as ProjectType;
							}}
						/>
					</div>
				</div>
				
				<div>
					<Label for="description">Description</Label>
					<Textarea id="description" bind:value={description} placeholder="Optional description" rows={2} />
				</div>

				<div>
					<Label for="projectPath">Project Path *</Label>
					<Input id="projectPath" bind:value={projectPath} placeholder="/path/to/project" required />
				</div>

				<div>
					<Label for="sdkVersion">SDK Version</Label>
					<Input id="sdkVersion" bind:value={sdkVersion} placeholder="latest" />
				</div>

				<!-- Docker-specific fields -->
				{#if deploymentType === DeploymentType.DOCKER}
					<div class="border-t pt-6 space-y-4">
						<h3 class="text-lg font-semibold">Docker Configuration</h3>
						
						<div>
							<Label for="dockerImageName">Docker Image Name</Label>
							<Input 
								id="dockerImageName" 
								bind:value={dockerImageName} 
								placeholder="my-app:latest (auto-generated if empty)"
							/>
							<p class="text-xs text-muted-foreground mt-1">
								Leave empty to auto-generate based on deployment name
							</p>
						</div>

						<div>
							<Label for="dockerfilePath">Dockerfile Path</Label>
							<Input 
								id="dockerfilePath" 
								bind:value={dockerfilePath} 
								placeholder="{projectPath}/Dockerfile (auto-generated if empty)"
							/>
							<p class="text-xs text-muted-foreground mt-1">
								Leave empty to auto-generate Dockerfile
							</p>
						</div>

						<div>
							<Label for="exposedPort">Exposed Port</Label>
							<Input 
								id="exposedPort" 
								type="number" 
								bind:value={exposedPort} 
								placeholder="3000"
							/>
						</div>
					</div>
				{/if}

				<!-- CLI-specific fields -->
				{#if deploymentType === DeploymentType.CLI}
					<div class="border-t pt-6 space-y-4">
						<h3 class="text-lg font-semibold">CLI Configuration</h3>
						
						<div>
							<Label for="command">Command *</Label>
							<Input 
								id="command" 
								bind:value={command} 
								placeholder="npm start"
								required
							/>
							<p class="text-xs text-muted-foreground mt-1">
								Enter the command to run (e.g., "npm start", "python app.py", "cargo run")
							</p>
						</div>

						<div>
							<Label for="workingDirectory">Working Directory</Label>
							<Input 
								id="workingDirectory" 
								bind:value={workingDirectory} 
								placeholder="{projectPath} (defaults to project path)"
							/>
						</div>
					</div>
				{/if}

				<!-- Environment Variables -->
				<div class="border-t pt-6">
					<div class="flex items-center justify-between mb-4">
						<div>
							<h3 class="text-lg font-semibold">Environment Variables</h3>
							<p class="text-sm text-muted-foreground">Optional environment variables for your deployment</p>
						</div>
						<Button type="button" variant="outline" size="sm" onclick={addEnvVar}>
							Add Variable
						</Button>
					</div>
					<div class="space-y-2">
						{#each Object.entries(envVars) as [key, value]}
							<div class="flex gap-2">
								<Input 
									placeholder="Variable name"
									value={key}
									oninput={(e) => {
										const input = e.target as HTMLInputElement;
										updateEnvVar(key, input.value, value);
									}}
									class="flex-1"
								/>
								<Input 
									placeholder="Value"
									value={value}
									oninput={(e) => {
										const input = e.target as HTMLInputElement;
										updateEnvVar(key, key, input.value);
									}}
									class="flex-1"
								/>
								<Button 
									type="button"
									variant="outline" 
									size="sm"
									onclick={() => removeEnvVar(key)}
								>
									Remove
								</Button>
							</div>
						{:else}
							<p class="text-sm text-muted-foreground text-center py-4">
								No environment variables added
							</p>
						{/each}
					</div>
				</div>
				
				<!-- Actions -->
				<div class="flex gap-2 pt-4 border-t">
					<Button 
						type="submit" 
						disabled={!name || !projectPath || (deploymentType === DeploymentType.CLI && !command) || isSubmitting}
						class="flex-1"
					>
						{#if isSubmitting}
							<Loader2 class="h-4 w-4 mr-2 animate-spin" />
							Creating...
						{:else}
							Create Deployment
						{/if}
					</Button>
					<Button type="button" variant="outline" onclick={() => goto('/deployments')} disabled={isSubmitting}>
						Cancel
					</Button>
				</div>
			</form>
		</CardContent>
	</Card>
</div>

