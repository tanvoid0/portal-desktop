<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';
	import Select from '@/lib/components/ui/select.svelte';
	import { DeploymentType, ProjectType, type DeploymentCreateRequest } from '../types';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';

	interface Props {
		onCreate?: (data: DeploymentCreateRequest) => void;
		onCancel?: () => void;
	}

	let { onCreate, onCancel }: Props = $props();

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

	function handleSubmit() {
		if (!name || !projectPath) {
			return;
		}

		// Validate type-specific fields
		if (deploymentType === DeploymentType.CLI && !command) {
			return;
		}

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

		onCreate?.(request);
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

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
	<Card class="w-full max-w-2xl max-h-[90vh] overflow-y-auto">
		<CardHeader>
			<CardTitle>Create New Deployment</CardTitle>
			<CardDescription>Set up a new deployment for your project</CardDescription>
		</CardHeader>

		<div class="space-y-4">
			<!-- Deployment Type -->
			<div>
				<Label for="deploymentType">Deployment Type</Label>
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
			<div>
				<Label for="name">Name *</Label>
				<Input id="name" bind:value={name} placeholder="My Deployment" />
			</div>
			
			<div>
				<Label for="description">Description</Label>
				<Textarea id="description" bind:value={description} placeholder="Optional description" />
			</div>

			<div>
				<Label for="projectPath">Project Path *</Label>
				<Input id="projectPath" bind:value={projectPath} placeholder="/path/to/project" />
			</div>

			<div>
				<Label for="projectType">Project Type</Label>
				<Select
					options={Object.values(ProjectType)}
					defaultValue={projectType}
					onSelect={(value) => {
						projectType = value as ProjectType;
					}}
				/>
			</div>

			<div>
				<Label for="sdkVersion">SDK Version</Label>
				<Input id="sdkVersion" bind:value={sdkVersion} placeholder="latest" />
			</div>

			<!-- Docker-specific fields -->
			{#if deploymentType === DeploymentType.DOCKER}
				<div>
					<Label for="dockerImageName">Docker Image Name</Label>
					<Input 
						id="dockerImageName" 
						bind:value={dockerImageName} 
						placeholder="my-app:latest (auto-generated if empty)"
					/>
				</div>

				<div>
					<Label for="dockerfilePath">Dockerfile Path</Label>
					<Input 
						id="dockerfilePath" 
						bind:value={dockerfilePath} 
						placeholder="{projectPath}/Dockerfile (auto-generated if empty)"
					/>
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
			{/if}

			<!-- CLI-specific fields -->
			{#if deploymentType === DeploymentType.CLI}
				<div>
					<Label for="command">Command *</Label>
					<Input 
						id="command" 
						bind:value={command} 
						placeholder="npm start"
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
			{/if}

			<!-- Environment Variables -->
			<div>
				<Label>Environment Variables</Label>
				<div class="space-y-2 mt-2">
					{#each Object.entries(envVars) as [key, value]}
						<div class="flex gap-2">
							<Input 
								placeholder="Variable name"
								value={key}
								oninput={(e) => {
									const input = e.target as HTMLInputElement;
									updateEnvVar(key, input.value, value);
								}}
							/>
							<Input 
								placeholder="Value"
								value={value}
								oninput={(e) => {
									const input = e.target as HTMLInputElement;
									updateEnvVar(key, key, input.value);
								}}
							/>
							<Button 
								variant="outline" 
								size="sm"
								onclick={() => removeEnvVar(key)}
							>
								Remove
							</Button>
						</div>
					{/each}
					<Button variant="outline" size="sm" onclick={addEnvVar}>
						Add Environment Variable
					</Button>
				</div>
			</div>
			
			<div class="flex gap-2 pt-4">
				<Button 
					onclick={handleSubmit} 
					disabled={!name || !projectPath || (deploymentType === DeploymentType.CLI && !command)}
				>
					Create Deployment
				</Button>
				<Button variant="outline" onclick={onCancel}>
					Cancel
				</Button>
			</div>
		</div>
	</Card>
</div>
