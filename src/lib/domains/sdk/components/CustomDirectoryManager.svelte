<!--
	Custom Directory Manager - FlyEnv-style directory management
	Allows users to add and manage custom SDK installation directories
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '$lib/components/ui/dialog';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		FolderPlus, 
		FolderOpen, 
		Trash2, 
		RefreshCw, 
		AlertCircle, 
		CheckCircle, 
		Plus,
		X,
		Search,
		Settings
	} from 'lucide-svelte';

	interface CustomDirectory {
		id: string;
		path: string;
		sdk_type: string;
		name: string;
		description?: string;
		is_valid: boolean;
		installations: SDKInstallation[];
	}

	interface SDKInstallation {
		sdk_type: string;
		version: string;
		path: string;
		is_active: boolean;
	}

	let { sdkType = 'all' }: { sdkType?: string } = $props();

	// State
	let directories = $state<CustomDirectory[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let showAddDialog = $state(false);
	let showEditDialog = $state(false);
	let selectedDirectory: CustomDirectory | null = $state(null);
	let searchTerm = $state('');

	// Form state
	let newDirectoryPath = $state('');
	let newDirectoryName = $state('');
	let newDirectoryDescription = $state('');
	let newDirectorySdkType = $state('nodejs');

	// Initialize
	onMount(() => {
		loadDirectories();
	});

	async function loadDirectories() {
		loading = true;
		error = null;
		
		try {
			const result = await invoke('get_custom_directories', { sdkType });
			directories = Array.isArray(result) ? result : [];
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load custom directories';
			console.error('Failed to load custom directories:', err);
		} finally {
			loading = false;
		}
	}

	async function addDirectory() {
		if (!newDirectoryPath.trim() || !newDirectoryName.trim()) {
			error = 'Path and name are required';
			return;
		}

		loading = true;
		error = null;

		try {
			await invoke('add_custom_sdk_directory', {
				path: newDirectoryPath,
				sdkType: newDirectorySdkType,
				name: newDirectoryName,
				description: newDirectoryDescription || null
			});

			// Reset form
			newDirectoryPath = '';
			newDirectoryName = '';
			newDirectoryDescription = '';
			newDirectorySdkType = 'nodejs';
			showAddDialog = false;

			// Reload directories
			await loadDirectories();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to add custom directory';
			console.error('Failed to add custom directory:', err);
		} finally {
			loading = false;
		}
	}

	async function removeDirectory(directoryId: string) {
		if (!confirm('Are you sure you want to remove this custom directory?')) {
			return;
		}

		loading = true;
		error = null;

		try {
			await invoke('remove_custom_sdk_directory', { directoryId });
			await loadDirectories();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to remove custom directory';
			console.error('Failed to remove custom directory:', err);
		} finally {
			loading = false;
		}
	}

	async function rescanDirectory(directoryId: string) {
		loading = true;
		error = null;

		try {
			await invoke('rescan_custom_directory', { directoryId });
			await loadDirectories();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to rescan directory';
			console.error('Failed to rescan directory:', err);
		} finally {
			loading = false;
		}
	}

	function openEditDialog(directory: CustomDirectory) {
		selectedDirectory = directory;
		showEditDialog = true;
	}

	function getSdkTypeColor(sdkType: string) {
		switch (sdkType.toLowerCase()) {
			case 'nodejs': return 'bg-green-100 text-green-800';
			case 'python': return 'bg-blue-100 text-blue-800';
			case 'java': return 'bg-orange-100 text-orange-800';
			case 'rust': return 'bg-red-100 text-red-800';
			case 'go': return 'bg-cyan-100 text-cyan-800';
			case 'php': return 'bg-purple-100 text-purple-800';
			case 'ruby': return 'bg-pink-100 text-pink-800';
			default: return 'bg-gray-100 text-gray-800';
		}
	}

	// Filter directories based on search term
	let filteredDirectories = $derived(() => {
		if (!searchTerm.trim()) return directories;
		
		const term = searchTerm.toLowerCase();
		return directories.filter(dir => 
			dir.name.toLowerCase().includes(term) ||
			dir.path.toLowerCase().includes(term) ||
			dir.sdk_type.toLowerCase().includes(term) ||
			(dir.description && dir.description.toLowerCase().includes(term))
		);
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold">Custom Directories</h2>
			<p class="text-muted-foreground">Manage custom SDK installation directories</p>
		</div>
		<Button onclick={() => showAddDialog = true}>
			<Plus class="w-4 h-4 mr-2" />
			Add Directory
		</Button>
	</div>

	<!-- Search -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
			<Input 
				placeholder="Search directories..." 
				bind:value={searchTerm}
				class="pl-10"
			/>
		</div>
		<Button variant="outline" onclick={loadDirectories} disabled={loading}>
			<RefreshCw class="w-4 h-4 mr-2" />
			Refresh
		</Button>
	</div>

	<!-- Error Alert -->
	{#if error}
		<Alert variant="destructive">
			<AlertCircle class="w-4 h-4" />
			<AlertDescription>{error}</AlertDescription>
		</Alert>
	{/if}

	<!-- Loading State -->
	{#if loading && directories.length === 0}
		<div class="flex items-center justify-center py-8">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
			<span class="ml-2">Loading directories...</span>
		</div>
	{:else if filteredDirectories().length === 0}
		<div class="text-center py-8 text-muted-foreground">
			<FolderOpen class="w-12 h-12 mx-auto mb-4 opacity-50" />
			<p>No custom directories found</p>
			{#if searchTerm}
				<p class="text-sm">Try adjusting your search terms</p>
			{:else}
				<p class="text-sm">Add a custom directory to get started</p>
			{/if}
		</div>
	{:else}
		<!-- Directories Grid -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each filteredDirectories() as directory}
				<Card class="hover:shadow-md transition-shadow">
					<CardHeader class="pb-3">
						<div class="flex items-start justify-between">
							<div class="flex-1 min-w-0">
								<CardTitle class="text-lg truncate">{directory.name}</CardTitle>
								<div class="flex items-center gap-2 mt-1">
									<Badge variant="outline" class={getSdkTypeColor(directory.sdk_type)}>
										{directory.sdk_type}
									</Badge>
									{#if directory.is_valid}
										<CheckCircle class="w-4 h-4 text-green-500" />
									{:else}
										<AlertCircle class="w-4 h-4 text-red-500" />
									{/if}
								</div>
							</div>
							<div class="flex items-center gap-1">
								<Button 
									variant="ghost" 
									size="sm" 
									onclick={() => openEditDialog(directory)}
									title="Edit directory"
								>
									<Settings class="w-4 h-4" />
								</Button>
								<Button 
									variant="ghost" 
									size="sm" 
									onclick={() => rescanDirectory(directory.id)}
									title="Rescan directory"
								>
									<RefreshCw class="w-4 h-4" />
								</Button>
								<Button 
									variant="ghost" 
									size="sm" 
									onclick={() => removeDirectory(directory.id)}
									title="Remove directory"
									class="text-red-500 hover:text-red-700"
								>
									<Trash2 class="w-4 h-4" />
								</Button>
							</div>
						</div>
					</CardHeader>
					<CardContent class="pt-0">
						<div class="space-y-3">
							<div>
								<Label class="text-xs text-muted-foreground">Path</Label>
								<p class="text-sm font-mono bg-muted p-2 rounded mt-1 break-all">
									{directory.path}
								</p>
							</div>
							
							{#if directory.description}
								<div>
									<Label class="text-xs text-muted-foreground">Description</Label>
									<p class="text-sm mt-1">{directory.description}</p>
								</div>
							{/if}

							<div>
								<Label class="text-xs text-muted-foreground">Installations</Label>
								<div class="flex flex-wrap gap-1 mt-1">
									{#if directory.installations.length > 0}
										{#each directory.installations as installation}
											<Badge variant="secondary" class="text-xs">
												{installation.version}
												{#if installation.is_active}
													<span class="ml-1 text-green-500">●</span>
												{/if}
											</Badge>
										{/each}
									{:else}
										<span class="text-xs text-muted-foreground">No installations found</span>
									{/if}
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>

<!-- Add Directory Dialog -->
<Dialog bind:open={showAddDialog}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Add Custom Directory</DialogTitle>
		</DialogHeader>
		
		<div class="space-y-4">
			<div>
				<Label for="directory-path">Directory Path</Label>
				<Input 
					id="directory-path"
					placeholder="/path/to/sdk/installations"
					bind:value={newDirectoryPath}
				/>
			</div>
			
			<div>
				<Label for="directory-name">Name</Label>
				<Input 
					id="directory-name"
					placeholder="My Custom SDKs"
					bind:value={newDirectoryName}
				/>
			</div>
			
			<div>
				<Label for="directory-sdk-type">SDK Type</Label>
				<select 
					id="directory-sdk-type"
					bind:value={newDirectorySdkType}
					class="w-full p-2 border rounded-md"
				>
					<option value="nodejs">Node.js</option>
					<option value="python">Python</option>
					<option value="java">Java</option>
					<option value="rust">Rust</option>
					<option value="go">Go</option>
					<option value="php">PHP</option>
					<option value="ruby">Ruby</option>
				</select>
			</div>
			
			<div>
				<Label for="directory-description">Description (Optional)</Label>
				<Input 
					id="directory-description"
					placeholder="Description of this directory"
					bind:value={newDirectoryDescription}
				/>
			</div>
		</div>
		
		<div class="flex justify-end gap-2 mt-6">
			<Button variant="outline" onclick={() => showAddDialog = false}>
				Cancel
			</Button>
			<Button onclick={addDirectory} disabled={loading}>
				{loading ? 'Adding...' : 'Add Directory'}
			</Button>
		</div>
	</DialogContent>
</Dialog>

<!-- Edit Directory Dialog -->
<Dialog bind:open={showEditDialog}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Edit Directory</DialogTitle>
		</DialogHeader>
		
		{#if selectedDirectory}
			<div class="space-y-4">
				<div>
					<Label>Directory Path</Label>
					<p class="text-sm font-mono bg-muted p-2 rounded mt-1">
						{selectedDirectory.path}
					</p>
				</div>
				
				<div>
					<Label>SDK Type</Label>
					<Badge variant="outline" class={getSdkTypeColor(selectedDirectory.sdk_type)}>
						{selectedDirectory.sdk_type}
					</Badge>
				</div>
				
				<div>
					<Label>Installations</Label>
					<div class="flex flex-wrap gap-1 mt-1">
						{#if selectedDirectory.installations.length > 0}
							{#each selectedDirectory.installations as installation}
								<Badge variant="secondary" class="text-xs">
									{installation.version}
									{#if installation.is_active}
										<span class="ml-1 text-green-500">●</span>
									{/if}
								</Badge>
							{/each}
						{:else}
							<span class="text-xs text-muted-foreground">No installations found</span>
						{/if}
					</div>
				</div>
			</div>
		{/if}
		
		<div class="flex justify-end gap-2 mt-6">
			<Button variant="outline" onclick={() => showEditDialog = false}>
				Close
			</Button>
			<Button 
				variant="destructive" 
				onclick={() => {
					if (selectedDirectory) {
						removeDirectory(selectedDirectory.id);
						showEditDialog = false;
					}
				}}
			>
				Remove Directory
			</Button>
		</div>
	</DialogContent>
</Dialog>
