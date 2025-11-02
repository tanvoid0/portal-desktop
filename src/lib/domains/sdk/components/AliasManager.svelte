<!--
	Alias Manager - FlyEnv-style alias management
	Allows users to create and manage aliases for SDK versions
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		Plus, 
		Trash2, 
		Edit, 
		RefreshCw, 
		AlertCircle, 
		CheckCircle,
		Search,
		Tag,
		X
	} from 'lucide-svelte';

	interface Alias {
		name: string;
		target_version: string;
		sdk_type: string;
		created_at: string;
		updated_at: string;
	}

	let { sdkType }: { sdkType: string } = $props();

	// State
	let aliases = $state<Alias[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let showAddDialog = $state(false);
	let showEditDialog = $state(false);
	let selectedAlias: Alias | null = $state(null);
	let searchTerm = $state('');

	// Form state
	let newAliasName = $state('');
	let newAliasTarget = $state('');
	let editAliasName = $state('');
	let editAliasTarget = $state('');

	// Initialize
	onMount(() => {
		loadAliases();
	});

	async function loadAliases() {
		loading = true;
		error = null;
		
		try {
			const result = await invoke('list_aliases', { sdkType });
			aliases = Array.isArray(result) ? result : [];
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load aliases';
			console.error('Failed to load aliases:', err);
		} finally {
			loading = false;
		}
	}

	async function createAlias() {
		if (!newAliasName.trim() || !newAliasTarget.trim()) {
			error = 'Alias name and target version are required';
			return;
		}

		loading = true;
		error = null;

		try {
			await invoke('create_alias', {
				sdkType,
				aliasName: newAliasName,
				targetVersion: newAliasTarget
			});

			// Reset form
			newAliasName = '';
			newAliasTarget = '';
			showAddDialog = false;

			// Reload aliases
			await loadAliases();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create alias';
			console.error('Failed to create alias:', err);
		} finally {
			loading = false;
		}
	}

	async function removeAlias(aliasName: string) {
		if (!confirm(`Are you sure you want to remove the alias "${aliasName}"?`)) {
			return;
		}

		loading = true;
		error = null;

		try {
			await invoke('remove_alias', { aliasName });
			await loadAliases();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to remove alias';
			console.error('Failed to remove alias:', err);
		} finally {
			loading = false;
		}
	}

	function openEditDialog(alias: Alias) {
		selectedAlias = alias;
		editAliasName = alias.name;
		editAliasTarget = alias.target_version;
		showEditDialog = true;
	}

	function closeEditDialog() {
		selectedAlias = null;
		editAliasName = '';
		editAliasTarget = '';
		showEditDialog = false;
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

	function formatDate(timestamp: string) {
		return new Date(timestamp).toLocaleDateString();
	}

	// Filter aliases based on search term
	let filteredAliases = $derived(() => {
		if (!searchTerm.trim()) return aliases;
		
		const term = searchTerm.toLowerCase();
		return aliases.filter(alias => 
			alias.name.toLowerCase().includes(term) ||
			alias.target_version.toLowerCase().includes(term)
		);
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-bold">Aliases</h2>
			<p class="text-muted-foreground">Manage version aliases for {sdkType}</p>
		</div>
		<Button onclick={() => showAddDialog = true}>
			<Plus class="w-4 h-4 mr-2" />
			Add Alias
		</Button>
	</div>

	<!-- Search -->
	<div class="flex items-center gap-4">
		<div class="flex-1 relative">
			<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
			<Input 
				placeholder="Search aliases..." 
				bind:value={searchTerm}
				class="pl-10"
			/>
		</div>
		<Button variant="outline" onclick={loadAliases} disabled={loading}>
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
	{#if loading && aliases.length === 0}
		<div class="flex items-center justify-center py-8">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
			<span class="ml-2">Loading aliases...</span>
		</div>
	{:else if filteredAliases().length === 0}
		<div class="text-center py-8 text-muted-foreground">
			<Tag class="w-12 h-12 mx-auto mb-4 opacity-50" />
			<p>No aliases found</p>
			{#if searchTerm}
				<p class="text-sm">Try adjusting your search terms</p>
			{:else}
				<p class="text-sm">Create an alias to get started</p>
			{/if}
		</div>
	{:else}
		<!-- Aliases List -->
		<div class="space-y-3">
			{#each filteredAliases() as alias}
				<Card class="hover:shadow-md transition-shadow">
					<CardContent class="p-4">
						<div class="flex items-center justify-between">
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-3">
									<div class="flex items-center gap-2">
										<Tag class="w-4 h-4 text-muted-foreground" />
										<code class="text-sm font-mono font-medium">{alias.name}</code>
									</div>
									<span class="text-muted-foreground">→</span>
									<Badge variant="outline" class="text-sm">
										{alias.target_version}
									</Badge>
								</div>
								
								<div class="flex items-center gap-2 mt-2">
									<Badge variant="outline" class={getSdkTypeColor(alias.sdk_type)}>
										{alias.sdk_type}
									</Badge>
									<span class="text-xs text-muted-foreground">
										Updated {formatDate(alias.updated_at)}
									</span>
								</div>
							</div>
							
							<div class="flex items-center gap-1">
								<Button 
									variant="ghost" 
									size="sm" 
									onclick={() => openEditDialog(alias)}
									title="Edit alias"
								>
									<Edit class="w-4 h-4" />
								</Button>
								<Button 
									variant="ghost" 
									size="sm" 
									onclick={() => removeAlias(alias.name)}
									title="Remove alias"
									class="text-red-500 hover:text-red-700"
								>
									<Trash2 class="w-4 h-4" />
								</Button>
							</div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>

<!-- Add Alias Dialog -->
<Dialog bind:open={showAddDialog}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Add Alias</DialogTitle>
		</DialogHeader>
		
		<div class="space-y-4">
			<div>
				<Label for="alias-name">Alias Name</Label>
				<Input 
					id="alias-name"
					placeholder="stable, latest, lts"
					bind:value={newAliasName}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					This will be the shortcut name for the version
				</p>
			</div>
			
			<div>
				<Label for="alias-target">Target Version</Label>
				<Input 
					id="alias-target"
					placeholder="18.17.0, 3.11.0, 1.70.0"
					bind:value={newAliasTarget}
				/>
				<p class="text-xs text-muted-foreground mt-1">
					The actual version this alias points to
				</p>
			</div>
		</div>
		
		<div class="flex justify-end gap-2 mt-6">
			<Button variant="outline" onclick={() => showAddDialog = false}>
				Cancel
			</Button>
			<Button onclick={createAlias} disabled={loading}>
				{loading ? 'Creating...' : 'Create Alias'}
			</Button>
		</div>
	</DialogContent>
</Dialog>

<!-- Edit Alias Dialog -->
<Dialog bind:open={showEditDialog}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>Edit Alias</DialogTitle>
		</DialogHeader>
		
		{#if selectedAlias}
			<div class="space-y-4">
				<div>
					<Label>Current Alias</Label>
					<div class="flex items-center gap-2 p-2 bg-muted rounded">
						<Tag class="w-4 h-4 text-muted-foreground" />
						<code class="text-sm font-mono">{selectedAlias.name}</code>
						<span class="text-muted-foreground">→</span>
						<Badge variant="outline">{selectedAlias.target_version}</Badge>
					</div>
				</div>
				
				<div>
					<Label for="edit-alias-target">New Target Version</Label>
					<Input 
						id="edit-alias-target"
						placeholder="Enter new target version"
						bind:value={editAliasTarget}
					/>
				</div>
			</div>
		{/if}
		
		<div class="flex justify-end gap-2 mt-6">
			<Button variant="outline" onclick={closeEditDialog}>
				Cancel
			</Button>
			<Button 
				variant="destructive" 
				onclick={() => {
					if (selectedAlias) {
						removeAlias(selectedAlias.name);
						closeEditDialog();
					}
				}}
			>
				Remove Alias
			</Button>
		</div>
	</DialogContent>
</Dialog>
