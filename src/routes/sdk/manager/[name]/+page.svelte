<!--
	SDK Manager Detail Page
	Shows details for a specific SDK manager with version management
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CheckCircle, XCircle, Download, ExternalLink, RefreshCw, Play, Trash2 } from '@lucide/svelte';
	import Devicon from '$lib/components/ui/devicon.svelte';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Input } from '$lib/components/ui/input';

	interface SDKManager {
		id: string;
		name: string;
		display_name: string;
		installed: boolean;
		version: string | null;
		supports_installation: boolean;
		supports_version_switching: boolean;
		install_command: string | null;
		website: string | null;
	}

	// Get manager name from URL
	let managerName = $derived($page.params.name);
	
	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let manager = $state<SDKManager | null>(null);
	let installedVersions = $state<string[]>([]);
	let availableVersions = $state<string[]>([]);
	let currentVersion = $state<string | null>(null);
	let activeTab = $state<'installed' | 'available'>('installed');
	let loadingVersions = $state(false);
	let installingVersion = $state<string | null>(null);
	let switchingVersion = $state<string | null>(null);

	// Load manager details
	$effect(() => {
		if (managerName) {
			loadManagerDetails();
		}
	});

	async function loadManagerDetails() {
		loading = true;
		error = null;
		
		try {
			// Get manager from all SDK managers
			const allManagers = await invoke<SDKManager[]>('get_all_sdk_managers');
			const foundManager = allManagers.find(m => m.id === managerName || m.name === managerName);
			
			if (!foundManager) {
				error = `Manager '${managerName}' not found`;
				return;
			}
			
			manager = foundManager;
			
			// Load versions if installed
			if (manager.installed) {
				await loadVersions();
			}
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load manager details';
			console.error('Failed to load manager details:', err);
		} finally {
			loading = false;
		}
	}

	async function loadVersions() {
		if (!manager || !manager.installed) return;
		
		loadingVersions = true;
		try {
			const [installed, available, current] = await Promise.all([
				invoke<string[]>('get_manager_installed_versions', { managerName: manager.id }).catch(() => []),
				invoke<string[]>('get_manager_available_versions', { managerName: manager.id }).catch(() => []),
				invoke<string | null>('get_manager_current_version', { managerName: manager.id }).catch(() => null),
			]);
			
			installedVersions = installed;
			availableVersions = available;
			currentVersion = current;
		} catch (err) {
			console.error('Failed to load versions:', err);
		} finally {
			loadingVersions = false;
		}
	}

	async function installVersion(version: string) {
		if (!manager) return;
		
		installingVersion = version;
		try {
			await invoke('install_version_via_manager', {
				managerName: manager.id,
				version: version,
			});
			
			// Reload versions
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to install version';
			console.error('Failed to install version:', err);
		} finally {
			installingVersion = null;
		}
	}

	async function switchVersion(version: string) {
		if (!manager) return;
		
		switchingVersion = version;
		try {
			await invoke('switch_version_via_manager', {
				managerName: manager.id,
				version: version,
			});
			
			// Reload versions to get updated current version
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to switch version';
			console.error('Failed to switch version:', err);
		} finally {
			switchingVersion = null;
		}
	}

	async function uninstallVersion(version: string) {
		if (!manager) return;
		
		if (!confirm(`Are you sure you want to uninstall version ${version}?`)) {
			return;
		}
		
		try {
			await invoke('uninstall_version_via_manager', {
				managerName: manager.id,
				version: version,
			});
			
			// Reload versions
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall version';
			console.error('Failed to uninstall version:', err);
		}
	}

	function getManagerIcon(managerId: string): string {
		const iconMap: Record<string, string> = {
			'nvm': 'devicon-nodejs-plain',
			'pyenv': 'devicon-python-plain',
			'rustup': 'devicon-rust-plain',
			'sdkman': 'devicon-sdkman-plain',
			'goenv': 'devicon-go-plain',
			'rbenv': 'devicon-ruby-plain',
			'phpenv': 'devicon-php-plain',
			'fnm': 'devicon-nodejs-plain'
		};
		return iconMap[managerId.toLowerCase()] || 'devicon-devicon-plain';
	}
</script>

<svelte:head>
	<title>{manager?.display_name || 'SDK Manager'} - Portal Desktop</title>
</svelte:head>

{#if loading}
	<div class="flex items-center justify-center p-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		<span class="ml-2">Loading...</span>
	</div>
{:else if error}
	<div class="p-4 border border-red-200 bg-red-50 rounded-md m-6">
		<p class="text-sm text-red-600">{error}</p>
	</div>
{:else if manager}
	<div class="space-y-6 p-6 w-full max-w-none">
		<!-- Header -->
		<div class="flex items-center gap-4">
			<div class="flex items-center gap-3 flex-1">
				<Devicon icon={getManagerIcon(manager.id)} class="h-12 w-12" />
				<div>
					<h1 class="text-3xl font-bold">{manager.display_name}</h1>
					<p class="text-muted-foreground">
						{manager.id} - {manager.supports_installation ? 'Supports installation' : 'Read-only'}
					</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if manager.installed}
					<Badge variant="default" class="bg-green-100 text-green-800">
						<CheckCircle class="w-3 h-3 mr-1" />
						Installed
						{#if manager.version}
							<span class="ml-1">({manager.version})</span>
						{/if}
					</Badge>
				{:else}
					<Badge variant="outline" class="text-gray-500">
						<XCircle class="w-3 h-3 mr-1" />
						Not Installed
					</Badge>
				{/if}
			</div>
		</div>

		{#if !manager.installed}
			<Card>
				<CardContent class="pt-6">
					<p class="text-muted-foreground mb-4">
						This SDK manager is not installed. Install it to manage SDK versions.
					</p>
					{#if manager.install_command}
						<div class="space-y-2">
							<p class="text-sm font-medium">Installation Command:</p>
							<code class="text-xs bg-muted p-2 rounded block font-mono break-all">
								{manager.install_command}
							</code>
						</div>
					{/if}
					{#if manager?.website}
						<div class="mt-4">
							<Button variant="outline" onclick={() => manager?.website && window.open(manager.website, '_blank')}>
								<ExternalLink class="w-4 h-4 mr-2" />
								Visit Website
							</Button>
						</div>
					{/if}
				</CardContent>
			</Card>
		{:else}
			<!-- Current Version -->
			{#if currentVersion}
				<Card>
					<CardHeader>
						<CardTitle>Current Active Version</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex items-center justify-between">
							<span class="text-2xl font-bold">{currentVersion}</span>
							<Badge variant="default" class="bg-blue-100 text-blue-800">
								Active
							</Badge>
						</div>
					</CardContent>
				</Card>
			{/if}

			<!-- Version Management Tabs -->
			<Tabs bind:value={activeTab}>
				<TabsList>
					<TabsTrigger value="installed">
						Installed ({installedVersions.length})
					</TabsTrigger>
					<TabsTrigger value="available">
						Available ({availableVersions.length})
					</TabsTrigger>
				</TabsList>

				<TabsContent value="installed" class="space-y-4">
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<CardTitle>Installed Versions</CardTitle>
								<Button variant="outline" size="sm" onclick={loadVersions} disabled={loadingVersions}>
									<RefreshCw class="w-4 h-4 mr-2 {loadingVersions ? 'animate-spin' : ''}" />
									Refresh
								</Button>
							</div>
						</CardHeader>
						<CardContent>
							{#if loadingVersions}
								<div class="flex items-center justify-center py-8">
									<RefreshCw class="h-6 w-6 animate-spin text-muted-foreground" />
								</div>
							{:else if installedVersions.length === 0}
								<p class="text-muted-foreground text-center py-8">No versions installed</p>
							{:else}
								<div class="space-y-2">
									{#each installedVersions as version}
										<div class="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50">
											<div class="flex items-center gap-3">
												<span class="font-mono text-sm">{version}</span>
												{#if currentVersion === version}
													<Badge variant="default" class="bg-blue-100 text-blue-800 text-xs">
														Current
													</Badge>
												{/if}
											</div>
											<div class="flex items-center gap-2">
												{#if currentVersion !== version}
													<Button
														variant="outline"
														size="sm"
														onclick={() => switchVersion(version)}
														disabled={switchingVersion === version}
													>
														{#if switchingVersion === version}
															<RefreshCw class="w-4 h-4 mr-1 animate-spin" />
														{:else}
															<Play class="w-4 h-4 mr-1" />
														{/if}
														Switch
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
											</div>
										</div>
									{/each}
								</div>
							{/if}
						</CardContent>
					</Card>
				</TabsContent>

				<TabsContent value="available" class="space-y-4">
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<CardTitle>Available Versions</CardTitle>
								<Button variant="outline" size="sm" onclick={loadVersions} disabled={loadingVersions}>
									<RefreshCw class="w-4 h-4 mr-2 {loadingVersions ? 'animate-spin' : ''}" />
									Refresh
								</Button>
							</div>
						</CardHeader>
						<CardContent>
							{#if loadingVersions}
								<div class="flex items-center justify-center py-8">
									<RefreshCw class="h-6 w-6 animate-spin text-muted-foreground" />
								</div>
							{:else if availableVersions.length === 0}
								<p class="text-muted-foreground text-center py-8">No versions available</p>
							{:else}
								<div class="space-y-2 max-h-[600px] overflow-y-auto">
									{#each availableVersions as version}
										{@const isInstalled = installedVersions.includes(version)}
										<div class="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50">
											<div class="flex items-center gap-3">
												<span class="font-mono text-sm">{version}</span>
												{#if isInstalled}
													<Badge variant="secondary" class="text-xs">
														Installed
													</Badge>
												{/if}
											</div>
											<div class="flex items-center gap-2">
												{#if !isInstalled && manager?.supports_installation}
													<Button
														variant="default"
														size="sm"
														onclick={() => installVersion(version)}
														disabled={installingVersion === version}
													>
														{#if installingVersion === version}
															<RefreshCw class="w-4 h-4 mr-1 animate-spin" />
															Installing...
														{:else}
															<Download class="w-4 h-4 mr-1" />
															Install
														{/if}
													</Button>
												{:else if isInstalled}
													<Button
														variant="outline"
														size="sm"
														onclick={() => switchVersion(version)}
														disabled={switchingVersion === version}
													>
														{#if switchingVersion === version}
															<RefreshCw class="w-4 h-4 mr-1 animate-spin" />
														{:else}
															<Play class="w-4 h-4 mr-1" />
														{/if}
														Switch
													</Button>
												{/if}
											</div>
										</div>
									{/each}
								</div>
							{/if}
						</CardContent>
					</Card>
				</TabsContent>
			</Tabs>

			<!-- Manager Info -->
			<div class="grid gap-6 md:grid-cols-2">
				<Card>
					<CardHeader>
						<CardTitle>Manager Information</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div>
							<p class="text-sm font-medium">ID</p>
							<p class="text-sm text-muted-foreground font-mono">{manager.id}</p>
						</div>
						{#if manager.version}
							<div>
								<p class="text-sm font-medium">Manager Version</p>
								<p class="text-sm text-muted-foreground">{manager.version}</p>
							</div>
						{/if}
						{#if manager.website}
							<div>
								<p class="text-sm font-medium">Website</p>
								<a 
									href={manager.website} 
									target="_blank" 
									class="text-sm text-blue-600 hover:underline flex items-center gap-1"
								>
									{manager.website}
									<ExternalLink class="w-3 h-3" />
								</a>
							</div>
						{/if}
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Capabilities</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex flex-wrap gap-2">
							{#if manager.supports_installation}
								<Badge variant="secondary" class="text-xs">Installation</Badge>
							{/if}
							{#if manager.supports_version_switching}
								<Badge variant="secondary" class="text-xs">Version Switching</Badge>
							{/if}
						</div>
					</CardContent>
				</Card>
			</div>
		{/if}
	</div>
{/if}

