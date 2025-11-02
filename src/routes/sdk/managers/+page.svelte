<!--
	SDK Managers Page
	Shows all SDK managers and their installation status
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CheckCircle, XCircle, Download, ExternalLink } from 'lucide-svelte';

	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let managers = $state<any[]>([]);
	let installationStatus = $state<Record<string, boolean>>({});

	// Initialize data
	onMount(async () => {
		await loadManagers();
	});

	async function loadManagers() {
		loading = true;
		error = null;
		
		try {
			// Load SDK managers from backend - get all language SDKs
			const sdkTypes = ['nodejs', 'python', 'java', 'rust', 'go', 'php', 'ruby'];
			const managerPromises = sdkTypes.map(async (sdkType) => {
				const sdkDetails = await invoke('get_sdk_details', { sdkType });
				return sdkDetails;
			});
			
			const sdkDetailsList = await Promise.all(managerPromises);
			managers = sdkDetailsList.filter(Boolean);
			
			// Check installation status for each manager
			for (const manager of managers) {
				try {
					const isInstalled = await invoke<boolean>('check_manager_installed', { managerName: manager.manager_type || manager.id });
					installationStatus[manager.id] = isInstalled;
				} catch {
					installationStatus[manager.id] = false;
				}
			}
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load SDK managers';
			console.error('Failed to load SDK managers:', err);
		} finally {
			loading = false;
		}
	}

	async function installManager(manager: any) {
		manager.installing = true;
		manager.installProgress = 0;
		
		try {
			// Execute installation command
			await invoke('execute_command', { 
				command: manager.install_command,
				workingDirectory: '/tmp'
			});
			
			manager.installing = false;
			installationStatus[manager.id] = true;
			
			// Reload data to get updated status
			await loadManagers();
		} catch (err) {
			manager.installing = false;
			manager.installError = err instanceof Error ? err.message : 'Installation failed';
		}
	}

	async function uninstallManager(manager: any) {
		try {
			// TODO: Implement uninstallation logic
			await invoke('uninstall_manager', { managerName: manager.id });
			installationStatus[manager.id] = false;
			
			// Reload data to get updated status
			await loadManagers();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall manager';
		}
	}
</script>

<svelte:head>
	<title>SDK Managers - Portal Desktop</title>
</svelte:head>

<div class="space-y-6 p-6 w-full max-w-none">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<div class="flex-1">
			<h1 class="text-3xl font-bold">SDK Managers</h1>
			<p class="text-muted-foreground">
				Install and manage SDK version managers for different programming languages
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadManagers} disabled={loading}>
				Refresh
			</Button>
		</div>
	</div>

	<!-- Error Alert -->
	{#if error}
		<div class="p-4 border border-red-200 bg-red-50 rounded-md">
			<p class="text-sm text-red-600">{error}</p>
		</div>
	{/if}

	<!-- Managers Grid -->
	<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
		{#each managers as manager}
			<Card class="relative">
				<CardHeader>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-bold">
								{manager.icon === 'nodejs' ? 'N' : manager.icon === 'rust' ? 'R' : manager.icon === 'python' ? 'P' : manager.icon === 'java' ? 'J' : manager.icon === 'ruby' ? 'R' : 'P'}
							</div>
							<div>
								<CardTitle class="text-lg">{manager.name}</CardTitle>
								<Badge variant="outline" class="text-xs">{manager.category}</Badge>
							</div>
						</div>
						<div class="flex items-center gap-2">
							{#if installationStatus[manager.id]}
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
				</CardHeader>
				<CardContent>
					<p class="text-sm text-muted-foreground mb-4">{manager.description}</p>
					
					<!-- Features -->
					<div class="mb-4">
						<h4 class="text-sm font-medium mb-2">Features:</h4>
						<div class="flex flex-wrap gap-1">
							{#each manager.features as feature}
								<Badge variant="secondary" class="text-xs">{feature}</Badge>
							{/each}
						</div>
					</div>
					
					<!-- Actions -->
					<div class="flex items-center gap-2">
						{#if installationStatus[manager.id]}
							<Button 
								variant="outline" 
								size="sm"
								onclick={() => uninstallManager(manager)}
							>
								<XCircle class="w-4 h-4 mr-1" />
								Uninstall
							</Button>
						{:else}
							<Button 
								size="sm"
								onclick={() => installManager(manager)}
								disabled={manager.installing}
							>
								{#if manager.installing}
									<Download class="w-4 h-4 mr-1" />
									Installing...
								{:else}
									<Download class="w-4 h-4 mr-1" />
									Install
								{/if}
							</Button>
						{/if}
						
						<Button 
							variant="ghost" 
							size="sm"
							onclick={() => window.open(manager.website, '_blank')}
						>
							<ExternalLink class="w-4 h-4 mr-1" />
							Website
						</Button>
					</div>
					
					<!-- Installation Error -->
					{#if manager.installError}
						<div class="mt-3 p-3 border border-red-200 bg-red-50 rounded-md">
							<p class="text-xs text-red-600">
								{manager.installError}
							</p>
						</div>
					{/if}
				</CardContent>
			</Card>
		{/each}
	</div>
</div>
