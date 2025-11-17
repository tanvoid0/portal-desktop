<!--
	Version Manager Table - FlyEnv-style version management table
	Shows versions with Path, ENV status, and Alias columns
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { sdkService } from '@/lib/domains/sdk/services/sdkService';
	import { Sparkles } from '@lucide/svelte';
	import { 
		CheckCircle, 
		XCircle, 
		AlertTriangle, 
		RefreshCw, 
		Search,
		Download,
		Trash2,
		Settings,
		ExternalLink,
		Shield,
		ShieldOff,
		Tag,
		FolderOpen,
		Play,
		Square
	} from '@lucide/svelte';

	interface VersionInfo {
		version: string;
		path: string;
		env_status: 'app-managed' | 'system-managed' | 'not-in-path';
		aliases: string[];
		is_active: boolean;
		is_installed: boolean;
		install_date?: string;
		size?: number;
	}

	let { sdkType }: { sdkType: string } = $props();

	// State
	let versions = $state<VersionInfo[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let searchTerm = $state('');
	let selectedVersions = $state<Set<string>>(new Set());
	let suggestedVersion = $state<string | null>(null);

	// Initialize
	onMount(async () => {
		await loadVersions();
		await loadSuggestedVersion();
	});

	async function loadVersions() {
		loading = true;
		error = null;
		
		try {
			const result = await invoke('get_sdk_versions', { sdkType });
			versions = Array.isArray(result) ? result : [];
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load versions';
			console.error('Failed to load versions:', err);
		} finally {
			loading = false;
		}
	}

	async function loadSuggestedVersion() {
		try {
			const suggested = await sdkService.getSuggestedVersion(sdkType);
			suggestedVersion = suggested;
		} catch (err) {
			console.error('Failed to load suggested version:', err);
			suggestedVersion = null;
		}
	}

	async function installVersion(version: string) {
		loading = true;
		error = null;

		try {
			await invoke('install_sdk_version', { sdkType, version });
			await loadVersions();
			// Reload suggestion after installing (may have changed)
			await loadSuggestedVersion();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to install version';
			console.error('Failed to install version:', err);
		} finally {
			loading = false;
		}
	}

	async function uninstallVersion(version: string) {
		if (!confirm(`Are you sure you want to uninstall version ${version}?`)) {
			return;
		}

		loading = true;
		error = null;

		try {
			await invoke('uninstall_sdk_version', { sdkType, version });
			await loadVersions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall version';
			console.error('Failed to uninstall version:', err);
		} finally {
			loading = false;
		}
	}

	async function switchToVersion(version: string) {
		loading = true;
		error = null;

		try {
			await invoke('switch_sdk_version', { sdkType, version });
			await loadVersions();
			// Reload suggestion after switching (may have changed)
			await loadSuggestedVersion();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to switch version';
			console.error('Failed to switch version:', err);
		} finally {
			loading = false;
		}
	}

	function getEnvStatusIcon(status: string) {
		switch (status) {
			case 'app-managed': return Shield;
			case 'system-managed': return Settings;
			case 'not-in-path': return ShieldOff;
			default: return AlertTriangle;
		}
	}

	function getEnvStatusColor(status: string) {
		switch (status) {
			case 'app-managed': return 'text-blue-600';
			case 'system-managed': return 'text-yellow-600';
			case 'not-in-path': return 'text-gray-600';
			default: return 'text-red-600';
		}
	}

	function getEnvStatusBadgeVariant(status: string) {
		switch (status) {
			case 'app-managed': return 'default';
			case 'system-managed': return 'secondary';
			case 'not-in-path': return 'outline';
			default: return 'destructive';
		}
	}

	function getEnvStatusText(status: string) {
		switch (status) {
			case 'app-managed': return 'App Managed';
			case 'system-managed': return 'System Managed';
			case 'not-in-path': return 'Not in PATH';
			default: return 'Unknown';
		}
	}

	function formatFileSize(bytes?: number) {
		if (!bytes) return 'Unknown';
		const sizes = ['Bytes', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
	}

	function formatDate(timestamp?: string) {
		if (!timestamp) return 'Unknown';
		return new Date(timestamp).toLocaleDateString();
	}

	function openInExplorer(path: string) {
		// This would open the path in the system file explorer
		console.log('Opening in explorer:', path);
	}

	// Filter versions based on search term
	let filteredVersions = $derived(() => {
		if (!searchTerm.trim()) return versions;
		
		const term = searchTerm.toLowerCase();
		return versions.filter(version => 
			version.version.toLowerCase().includes(term) ||
			version.path.toLowerCase().includes(term) ||
			version.aliases.some(alias => alias.toLowerCase().includes(term))
		);
	});

	function toggleVersionSelection(version: string) {
		const newSelection = new Set(selectedVersions);
		if (newSelection.has(version)) {
			newSelection.delete(version);
		} else {
			newSelection.add(version);
		}
		selectedVersions = newSelection;
	}

	function selectAllVersions() {
		selectedVersions = new Set(filteredVersions().map(v => v.version));
	}

	function clearSelection() {
		selectedVersions = new Set();
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle class="text-xl">Version Manager</CardTitle>
				{#if suggestedVersion}
					<p class="text-sm text-muted-foreground mt-1 flex items-center gap-1">
						<Sparkles class="w-3 h-3" />
						Suggested version: <code class="text-primary font-mono">{suggestedVersion}</code>
					</p>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadVersions} disabled={loading}>
					<RefreshCw class="w-4 h-4" />
				</Button>
			</div>
		</div>
	</CardHeader>
	
	<CardContent class="space-y-4">
		<!-- Search and Actions -->
		<div class="flex items-center gap-4">
			<div class="flex-1 relative">
				<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
				<Input 
					placeholder="Search versions..." 
					bind:value={searchTerm}
					class="pl-10"
				/>
			</div>
			
			{#if selectedVersions.size > 0}
				<div class="flex items-center gap-2">
					<span class="text-sm text-muted-foreground">
						{selectedVersions.size} selected
					</span>
					<Button variant="outline" size="sm" onclick={clearSelection}>
						Clear
					</Button>
				</div>
			{/if}
		</div>

		<!-- Error Alert -->
		{#if error}
			<Alert variant="destructive">
				<AlertTriangle class="w-4 h-4" />
				<AlertDescription>{error}</AlertDescription>
			</Alert>
		{/if}

		<!-- Loading State -->
		{#if loading && versions.length === 0}
			<div class="flex items-center justify-center py-8">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
				<span class="ml-2">Loading versions...</span>
			</div>
		{:else if filteredVersions().length === 0}
			<div class="text-center py-8 text-muted-foreground">
				<Download class="w-12 h-12 mx-auto mb-4 opacity-50" />
				<p>No versions found</p>
				{#if searchTerm}
					<p class="text-sm">Try adjusting your search terms</p>
				{:else}
					<p class="text-sm">Install a version to get started</p>
				{/if}
			</div>
		{:else}
			<!-- Versions Table -->
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead>
						<tr class="border-b">
							<th class="text-left p-3">
								<input 
									type="checkbox" 
									onchange={(e) => {
										if ((e.target as HTMLInputElement).checked) {
											selectAllVersions();
										} else {
											clearSelection();
										}
									}}
									class="form-checkbox"
								/>
							</th>
							<th class="text-left p-3 font-medium">Version</th>
							<th class="text-left p-3 font-medium">Path</th>
							<th class="text-left p-3 font-medium">ENV Status</th>
							<th class="text-left p-3 font-medium">Aliases</th>
							<th class="text-left p-3 font-medium">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredVersions() as version}
							<tr class="border-b hover:bg-muted/50 transition-colors">
								<td class="p-3">
									<input 
										type="checkbox" 
										checked={selectedVersions.has(version.version)}
										onchange={() => toggleVersionSelection(version.version)}
										class="form-checkbox"
									/>
								</td>
								
								<td class="p-3">
									<div class="flex items-center gap-2">
										<code class="text-sm font-mono font-medium">{version.version}</code>
										{#if version.is_active}
											<Badge variant="default" class="text-xs">
												<Play class="w-3 h-3 mr-1" />
												Active
											</Badge>
										{/if}
										{#if suggestedVersion && version.version === suggestedVersion}
											<Badge variant="outline" class="text-xs border-primary text-primary bg-primary/5">
												<Sparkles class="w-3 h-3 mr-1" />
												Suggested
											</Badge>
										{/if}
									</div>
								</td>
								
								<td class="p-3">
									<div class="flex items-center gap-2 min-w-0">
										<code class="text-sm font-mono text-muted-foreground truncate max-w-xs">
											{version.path}
										</code>
										<Button 
											variant="ghost" 
											size="sm" 
											onclick={() => openInExplorer(version.path)}
											title="Open in Explorer"
										>
											<ExternalLink class="w-3 h-3" />
										</Button>
									</div>
								</td>
								
								<td class="p-3">
									<Badge variant={getEnvStatusBadgeVariant(version.env_status)}>
										{@const StatusIcon = getEnvStatusIcon(version.env_status)}
										<StatusIcon class="w-3 h-3 mr-1" />
										{getEnvStatusText(version.env_status)}
									</Badge>
								</td>
								
								<td class="p-3">
									<div class="flex flex-wrap gap-1">
										{#if version.aliases.length > 0}
											{#each version.aliases as alias}
												<Badge variant="secondary" class="text-xs">
													<Tag class="w-3 h-3 mr-1" />
													{alias}
												</Badge>
											{/each}
										{:else}
											<span class="text-xs text-muted-foreground">None</span>
										{/if}
									</div>
								</td>
								
								<td class="p-3">
									<div class="flex items-center gap-1">
										{#if version.is_installed}
											{#if !version.is_active}
												<Button 
													variant="outline" 
													size="sm" 
													onclick={() => switchToVersion(version.version)}
													disabled={loading}
												>
													<Play class="w-3 h-3 mr-1" />
													Use
												</Button>
											{/if}
											<Button 
												variant="outline" 
												size="sm" 
												onclick={() => uninstallVersion(version.version)}
												disabled={loading}
												class="text-red-500 hover:text-red-700"
											>
												<Trash2 class="w-3 h-3 mr-1" />
												Remove
											</Button>
										{:else}
											<Button 
												variant="default" 
												size="sm" 
												onclick={() => installVersion(version.version)}
												disabled={loading}
											>
												<Download class="w-3 h-3 mr-1" />
												Install
											</Button>
										{/if}
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<!-- Version Stats -->
			<div class="flex items-center justify-between pt-4 border-t">
				<div class="text-sm text-muted-foreground">
					{filteredVersions().length} versions
					{#if selectedVersions.size > 0}
						• {selectedVersions.size} selected
					{/if}
				</div>
				
				{#if selectedVersions.size > 0}
					<div class="flex items-center gap-2">
						<Button variant="outline" size="sm">
							<Download class="w-4 h-4 mr-2" />
							Install Selected
						</Button>
						<Button variant="outline" size="sm" class="text-red-500 hover:text-red-700">
							<Trash2 class="w-4 h-4 mr-2" />
							Remove Selected
						</Button>
					</div>
				{/if}
			</div>
		{/if}
	</CardContent>
</Card>
