<!--
	Service Config Editor - FlyEnv-style configuration editor
	Allows editing service configuration files with syntax highlighting
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import Select from '$lib/components/ui/select.svelte';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { 
		Settings, 
		Save, 
		RotateCcw, 
		Download, 
		Upload,
		AlertCircle,
		CheckCircle,
		FileText,
		Code
	} from 'lucide-svelte';

	interface ConfigFile {
		name: string;
		path: string;
		content: string;
		format: 'json' | 'yaml' | 'toml' | 'ini' | 'xml' | 'text';
		lastModified: string;
	}

	interface EnvironmentVariable {
		name: string;
		value: string;
		scope: 'global' | 'session' | 'project';
	}

	interface Props {
		serviceId: string;
		serviceName: string;
		isOpen: boolean;
		onClose: () => void;
	}

	let { serviceId, serviceName, isOpen, onClose }: Props = $props();

	// State
	let configFiles = $state<ConfigFile[]>([]);
	let environmentVars = $state<EnvironmentVariable[]>([]);
	let activeTab = $state('files');
	let selectedFile = $state<ConfigFile | null>(null);
	let editedContent = $state('');
	let hasChanges = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);

	// Initialize
	onMount(async () => {
		if (isOpen) {
			await loadConfig();
		}
	});

	// Watch for dialog open/close
	$effect(() => {
		if (isOpen) {
			loadConfig();
		}
	});

	async function loadConfig() {
		loading = true;
		error = null;
		
		try {
			// Mock config files - in real implementation, this would load from the service
			configFiles = [
				{
					name: 'nginx.conf',
					path: '/etc/nginx/nginx.conf',
					content: `server {
    listen 80;
    server_name localhost;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}`,
					format: 'text',
					lastModified: new Date().toISOString()
				},
				{
					name: 'config.json',
					path: '/path/to/service/config.json',
					content: `{
    "port": 3000,
    "host": "localhost",
    "database": {
        "host": "localhost",
        "port": 5432,
        "name": "myapp"
    },
    "logging": {
        "level": "info",
        "file": "/var/log/service.log"
    }
}`,
					format: 'json',
					lastModified: new Date().toISOString()
				}
			];

			// Mock environment variables
			environmentVars = [
				{ name: 'NODE_ENV', value: 'development', scope: 'project' },
				{ name: 'PORT', value: '3000', scope: 'session' },
				{ name: 'DATABASE_URL', value: 'postgres://localhost:5432/myapp', scope: 'project' }
			];

		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load configuration';
			console.error('Failed to load service config:', err);
		} finally {
			loading = false;
		}
	}

	function selectFile(file: ConfigFile) {
		selectedFile = file;
		editedContent = file.content;
		hasChanges = false;
	}

	function handleContentChange() {
		hasChanges = editedContent !== selectedFile?.content;
	}

	async function saveConfig() {
		if (!selectedFile) return;

		loading = true;
		error = null;
		success = null;

		try {
			await invoke('update_service_config', {
				serviceId,
				config: {
					[selectedFile.name]: editedContent
				}
			});

			// Update the file content
			selectedFile.content = editedContent;
			selectedFile.lastModified = new Date().toISOString();
			hasChanges = false;
			success = 'Configuration saved successfully';

			// Clear success message after 3 seconds
			setTimeout(() => {
				success = null;
			}, 3000);

		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save configuration';
		} finally {
			loading = false;
		}
	}

	function resetChanges() {
		if (selectedFile) {
			editedContent = selectedFile.content;
			hasChanges = false;
		}
	}

	async function downloadConfig() {
		if (!selectedFile) return;

		try {
			const blob = new Blob([editedContent], { type: 'text/plain' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = selectedFile.name;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
		} catch (err) {
			error = 'Failed to download configuration file';
		}
	}

	function addEnvironmentVar() {
		environmentVars = [...environmentVars, { name: '', value: '', scope: 'project' }];
	}

	function removeEnvironmentVar(index: number) {
		environmentVars = environmentVars.filter((_, i) => i !== index);
	}

	function updateEnvironmentVar(index: number, field: keyof EnvironmentVariable, value: string) {
		environmentVars[index] = { ...environmentVars[index], [field]: value };
	}

	function getFileIcon(format: string) {
		switch (format) {
			case 'json': return '{}';
			case 'yaml': return 'Y';
			case 'toml': return 'T';
			case 'ini': return 'I';
			case 'xml': return 'X';
			default: return 'T';
		}
	}

	function getFileLanguage(format: string) {
		switch (format) {
			case 'json': return 'json';
			case 'yaml': return 'yaml';
			case 'toml': return 'toml';
			case 'ini': return 'ini';
			case 'xml': return 'xml';
			default: return 'text';
		}
	}
</script>

<Dialog bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
	<DialogContent class="max-w-6xl h-[90vh] flex flex-col">
		<DialogHeader>
			<DialogTitle class="flex items-center gap-2">
				<Settings class="w-5 h-5" />
				Service Configuration: {serviceName}
			</DialogTitle>
		</DialogHeader>

		<!-- Error/Success Messages -->
		{#if error}
			<div class="p-3 border border-red-200 bg-red-50 rounded-md">
				<div class="flex items-center gap-2">
					<AlertCircle class="w-4 h-4 text-red-600" />
					<span class="text-sm text-red-600">{error}</span>
				</div>
			</div>
		{/if}

		{#if success}
			<div class="p-3 border border-green-200 bg-green-50 rounded-md">
				<div class="flex items-center gap-2">
					<CheckCircle class="w-4 h-4 text-green-600" />
					<span class="text-sm text-green-600">{success}</span>
				</div>
			</div>
		{/if}

		<!-- Main Content -->
		<Tabs bind:value={activeTab} class="flex-1 flex flex-col">
			<TabsList class="grid w-full grid-cols-2">
				<TabsTrigger value="files">
					<FileText class="w-4 h-4 mr-2" />
					Configuration Files
				</TabsTrigger>
				<TabsTrigger value="environment">
					<Code class="w-4 h-4 mr-2" />
					Environment Variables
				</TabsTrigger>
			</TabsList>

			<!-- Configuration Files Tab -->
			<TabsContent value="files" class="flex-1 flex gap-4">
				<!-- File List -->
				<div class="w-1/3 space-y-2">
					<h3 class="font-medium mb-2">Configuration Files</h3>
					<div class="space-y-1">
						{#each configFiles as file}
							<Button
								variant={selectedFile?.name === file.name ? 'default' : 'ghost'}
								class="w-full justify-start"
								onclick={() => selectFile(file)}
							>
								<span class="mr-2">{getFileIcon(file.format)}</span>
								{file.name}
							</Button>
						{/each}
					</div>
				</div>

				<!-- File Editor -->
				<div class="flex-1 flex flex-col">
					{#if selectedFile}
						<div class="flex items-center justify-between mb-4">
							<div>
								<h3 class="font-medium">{selectedFile.name}</h3>
								<p class="text-sm text-muted-foreground">{selectedFile.path}</p>
							</div>
							<div class="flex items-center gap-2">
								<Badge variant="outline">{getFileLanguage(selectedFile.format)}</Badge>
								<Button variant="outline" size="sm" onclick={downloadConfig}>
									<Download class="w-4 h-4 mr-2" />
									Download
								</Button>
							</div>
						</div>

						<Textarea
							bind:value={editedContent}
							oninput={handleContentChange}
							class="flex-1 font-mono text-sm"
							placeholder="Configuration content..."
						/>

						<div class="flex items-center justify-between mt-4">
							<div class="flex items-center gap-2">
								{#if hasChanges}
									<Badge variant="secondary">Unsaved changes</Badge>
								{/if}
							</div>
							<div class="flex items-center gap-2">
								<Button variant="outline" onclick={resetChanges} disabled={!hasChanges}>
									<RotateCcw class="w-4 h-4 mr-2" />
									Reset
								</Button>
								<Button onclick={saveConfig} disabled={!hasChanges || loading}>
									<Save class="w-4 h-4 mr-2" />
									Save
								</Button>
							</div>
						</div>
					{:else}
						<div class="flex-1 flex items-center justify-center text-muted-foreground">
							<div class="text-center">
								<FileText class="w-12 h-12 mx-auto mb-4 opacity-50" />
								<p>Select a configuration file to edit</p>
							</div>
						</div>
					{/if}
				</div>
			</TabsContent>

			<!-- Environment Variables Tab -->
			<TabsContent value="environment" class="flex-1">
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<h3 class="font-medium">Environment Variables</h3>
						<Button variant="outline" onclick={addEnvironmentVar}>
							Add Variable
						</Button>
					</div>

					<div class="space-y-3">
						{#each environmentVars as envVar, index}
							<div class="flex items-center gap-3 p-3 border rounded-lg">
								<div class="flex-1">
									<Label for="env-name-{index}" class="text-xs">Name</Label>
									<Input
										id="env-name-{index}"
										bind:value={envVar.name}
										oninput={(e) => updateEnvironmentVar(index, 'name', (e.target as HTMLInputElement).value)}
										placeholder="VARIABLE_NAME"
									/>
								</div>
								<div class="flex-1">
									<Label for="env-value-{index}" class="text-xs">Value</Label>
									<Input
										id="env-value-{index}"
										bind:value={envVar.value}
										oninput={(e) => updateEnvironmentVar(index, 'value', (e.target as HTMLInputElement)?.value || '')}
										placeholder="variable_value"
									/>
								</div>
								<div class="w-32">
									<Label for="env-scope-{index}" class="text-xs">Scope</Label>
									<Select
										options={[
											{ value: 'project', label: 'Project' },
											{ value: 'session', label: 'Session' },
											{ value: 'global', label: 'Global' }
										]}
										defaultValue={envVar.scope}
										onSelect={(value: string) => updateEnvironmentVar(index, 'scope', value)}
									/>
								</div>
								<Button
									variant="ghost"
									size="sm"
									onclick={() => removeEnvironmentVar(index)}
								>
									Remove
								</Button>
							</div>
						{/each}
					</div>
				</div>
			</TabsContent>
		</Tabs>
	</DialogContent>
</Dialog>
