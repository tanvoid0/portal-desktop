<!--
	SDK Manager Detail Page
	Shows details for a specific SDK manager
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CheckCircle, XCircle, Download, ExternalLink } from 'lucide-svelte';

	// Get manager name from URL
	let managerName = $derived($page.params.name);
	
	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let manager = $state<any>(null);
	let isInstalled = $state(false);
	let versions = $state<any[]>([]);

	// Initialize data
	onMount(async () => {
		await loadManagerDetails();
	});

	async function loadManagerDetails() {
		loading = true;
		error = null;
		
		try {
			// Load manager details from backend
			const sdkDetails = await invoke('get_sdk_details', { sdkType: managerName });
			
			if (!sdkDetails) {
				error = `Manager '${managerName}' not found`;
				return;
			}
			
			manager = sdkDetails;
			
			// Check installation status
			try {
				isInstalled = await invoke<boolean>('check_manager_installed', { managerName: manager.manager_type || manager.id });
			} catch {
				isInstalled = false;
			}
			
			// Load versions if installed
			if (isInstalled) {
				try {
					versions = await invoke('fetch_available_versions', { sdkType: managerName });
				} catch {
					versions = [];
				}
			}
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load manager details';
			console.error('Failed to load manager details:', err);
		} finally {
			loading = false;
		}
	}

	async function installManager() {
		manager.installing = true;
		manager.installProgress = 0;
		
		try {
			// Execute installation command
			await invoke('execute_command', { 
				command: manager.install_command,
				workingDirectory: '/tmp'
			});
			
			manager.installing = false;
			isInstalled = true;
			
			// Reload data to get updated status
			await loadManagerDetails();
		} catch (err) {
			manager.installing = false;
			manager.installError = err instanceof Error ? err.message : 'Installation failed';
		}
	}

	async function uninstallManager() {
		try {
			// TODO: Implement uninstallation logic
			await invoke('uninstall_manager', { managerName: manager.id });
			isInstalled = false;
			
			// Reload data to get updated status
			await loadManagerDetails();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall manager';
		}
	}
</script>

<svelte:head>
	<title>{manager?.name || 'SDK Manager'} - Portal Desktop</title>
</svelte:head>

{#if loading}
	<div class="flex items-center justify-center p-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		<span class="ml-2">Loading...</span>
	</div>
{:else if error}
	<div class="p-4 border border-red-200 bg-red-50 rounded-md">
		<p class="text-sm text-red-600">{error}</p>
	</div>
{:else if manager}
	<div class="space-y-6 p-6 w-full max-w-none">
		<!-- Header -->
		<div class="flex items-center gap-4">
			<div class="flex-1">
				<h1 class="text-3xl font-bold">{manager.name}</h1>
				<p class="text-muted-foreground">{manager.description}</p>
			</div>
			<div class="flex items-center gap-2">
				{#if isInstalled}
					<Badge variant="default" class="bg-green-100 text-green-800">
						<CheckCircle class="w-3 h-3 mr-1" />
						Installed
					</Badge>
				{:else}
					<Badge variant="outline" class="text-gray-500">
						<XCircle class="w-3 h-3 mr-1" />
						Not Installed
					</Badge>
				{/if}
			</div>
		</div>

		<!-- Manager Details -->
		<div class="flex flex-col gap-6 lg:flex-row">
			<Card>
				<CardHeader>
					<CardTitle>Manager Information</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<label class="text-sm font-medium">Category</label>
						<p class="text-sm text-muted-foreground">{manager.category}</p>
					</div>
					<div>
						<label class="text-sm font-medium">Website</label>
						<a 
							href={manager.website} 
							target="_blank" 
							class="text-sm text-blue-600 hover:underline flex items-center gap-1"
						>
							{manager.website}
							<ExternalLink class="w-3 h-3" />
						</a>
					</div>
					<div>
						<label class="text-sm font-medium">Installation Command</label>
						<code class="text-xs bg-gray-100 p-2 rounded block mt-1">{manager.installCommand}</code>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Features</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex flex-wrap gap-2">
						{#each manager.features as feature}
							<Badge variant="secondary" class="text-xs">{feature}</Badge>
						{/each}
					</div>
				</CardContent>
			</Card>
		</div>

		<!-- Actions -->
		<Card>
			<CardHeader>
				<CardTitle>Actions</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex items-center gap-4">
					{#if isInstalled}
						<Button 
							variant="outline" 
							onclick={uninstallManager}
						>
							<XCircle class="w-4 h-4 mr-2" />
							Uninstall Manager
						</Button>
					{:else}
						<Button 
							onclick={installManager}
							disabled={manager.installing}
						>
							{#if manager.installing}
								<Download class="w-4 h-4 mr-2" />
								Installing...
							{:else}
								<Download class="w-4 h-4 mr-2" />
								Install Manager
							{/if}
						</Button>
					{/if}
					
					<Button 
						variant="outline"
						onclick={() => window.open(manager.website, '_blank')}
					>
						<ExternalLink class="w-4 h-4 mr-2" />
						Visit Website
					</Button>
				</div>
				
				<!-- Installation Error -->
				{#if manager.installError}
					<div class="mt-4 p-3 border border-red-200 bg-red-50 rounded-md">
						<p class="text-sm text-red-600">
							{manager.installError}
						</p>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Versions (if installed) -->
		{#if isInstalled && versions.length > 0}
			<Card>
				<CardHeader>
					<CardTitle>Available Versions</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="space-y-2">
						{#each versions.slice(0, 10) as version}
							<div class="flex items-center justify-between p-2 border rounded">
								<span class="text-sm">{version.version}</span>
								{#if version.lts}
									<Badge variant="secondary" class="text-xs">LTS</Badge>
								{/if}
							</div>
						{/each}
						{#if versions.length > 10}
							<p class="text-sm text-muted-foreground">... and {versions.length - 10} more versions</p>
						{/if}
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
{/if}
