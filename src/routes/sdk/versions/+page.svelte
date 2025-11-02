<!--
	SDK Versions Page
	Shows installed SDK versions and allows management
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CheckCircle, XCircle, Download, Trash2, Play } from 'lucide-svelte';

	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let versions = $state<any[]>([]);

	// Initialize data
	onMount(async () => {
		await loadVersions();
	});

	async function loadVersions() {
		loading = true;
		error = null;
		
		try {
			// Load SDK versions
			const availableSDKs = await invoke<any[]>('get_all_available_sdks');
			
			// Load versions for each SDK
			const versionPromises = availableSDKs.map(async (sdk: any) => {
				try {
					const versions = await invoke('fetch_available_versions', { sdkType: sdk.id });
					return {
						...sdk,
						versions: versions || []
					};
				} catch {
					return {
						...sdk,
						versions: []
					};
				}
			});
			
			const sdkWithVersions = await Promise.all(versionPromises);
			
			// Flatten versions from all SDKs
			versions = sdkWithVersions.flatMap((sdk: any) => 
				sdk.versions.map((version: any) => ({
					...version,
					sdkType: sdk.id,
					sdkName: sdk.name
				}))
			);
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load SDK versions';
			console.error('Failed to load SDK versions:', err);
		} finally {
			loading = false;
		}
	}

	async function installVersion(version: any) {
		version.downloading = true;
		version.progress = 0;
		
		try {
			await invoke('download_and_install_version', { 
				sdkType: version.sdkType, 
				version: version.version,
				use_manager: false
			});
			
			version.downloading = false;
			version.installed = true;
			
			// Reload data to get updated status
			await loadVersions();
		} catch (err) {
			version.downloading = false;
			version.error = err instanceof Error ? err.message : 'Installation failed';
		}
	}

	async function uninstallVersion(version: any) {
		try {
			await invoke('uninstall_sdk_version', { 
				sdkType: version.sdkType, 
				version: version.version 
			});
			version.installed = false;
			
			// Reload data to get updated status
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall version';
		}
	}

	async function setActiveVersion(version: any) {
		try {
			await invoke('switch_sdk_version', { 
				sdkType: version.sdkType, 
				version: version.version 
			});
			
			// Update all versions
			versions.forEach(v => v.active = false);
			version.active = true;
			
			// Reload data to get updated status
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to set active version';
		}
	}
</script>

<svelte:head>
	<title>SDK Versions - Portal Desktop</title>
</svelte:head>

<div class="space-y-6 p-6 w-full max-w-none">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<div class="flex-1">
			<h1 class="text-3xl font-bold">SDK Versions</h1>
			<p class="text-muted-foreground">
				Manage installed SDK versions and switch between them
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadVersions} disabled={loading}>
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

	<!-- Versions List -->
	<div class="space-y-4">
		{#each versions as version}
			<Card>
				<CardContent class="p-6">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="flex items-center gap-2">
								<h3 class="text-lg font-semibold">{version.sdkName}</h3>
								<Badge variant="outline">{version.version}</Badge>
								{#if version.lts}
									<Badge variant="secondary">LTS</Badge>
								{/if}
							</div>
							<div class="flex items-center gap-2">
								{#if version.installed}
									{#if version.active}
										<Badge variant="default" class="bg-green-100 text-green-800">
											<CheckCircle class="w-3 h-3 mr-1" />
											Active
										</Badge>
									{:else}
										<Badge variant="outline" class="text-green-600">
											<CheckCircle class="w-3 h-3 mr-1" />
											Installed
										</Badge>
									{/if}
								{:else}
									<Badge variant="outline" class="text-gray-500">
										<XCircle class="w-3 h-3 mr-1" />
										Not Installed
									</Badge>
								{/if}
							</div>
						</div>
						
						<div class="flex items-center gap-2">
							{#if version.installed}
								{#if !version.active}
									<Button 
										variant="outline" 
										size="sm"
										onclick={() => setActiveVersion(version)}
									>
										<Play class="w-4 h-4 mr-1" />
										Activate
									</Button>
								{/if}
								<Button 
									variant="outline" 
									size="sm"
									onclick={() => uninstallVersion(version)}
								>
									<Trash2 class="w-4 h-4 mr-1" />
									Uninstall
								</Button>
							{:else}
								<Button 
									size="sm"
									onclick={() => installVersion(version)}
									disabled={version.downloading}
								>
									{#if version.downloading}
										<Download class="w-4 h-4 mr-1" />
										Installing...
									{:else}
										<Download class="w-4 h-4 mr-1" />
										Install
									{/if}
								</Button>
							{/if}
						</div>
					</div>
					
					{#if version.description}
						<p class="text-sm text-muted-foreground mt-2">{version.description}</p>
					{/if}
					
					{#if version.release_date}
						<p class="text-xs text-muted-foreground mt-1">Released: {version.release_date}</p>
					{/if}
					
					<!-- Installation Error -->
					{#if version.error}
						<div class="mt-3 p-3 border border-red-200 bg-red-50 rounded-md">
							<p class="text-xs text-red-600">
								{version.error}
							</p>
						</div>
					{/if}
				</CardContent>
			</Card>
		{/each}
	</div>
</div>
