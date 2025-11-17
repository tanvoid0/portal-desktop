<!--
	Path Status - FlyEnv-style PATH environment status indicator
	Shows whether SDK is in PATH and how it's managed (app vs system)
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		CheckCircle, 
		XCircle, 
		AlertTriangle, 
		Settings, 
		RefreshCw,
		ExternalLink,
		Shield,
		ShieldOff,
		Wifi,
		WifiOff
	} from '@lucide/svelte';

	interface PathStatus {
		sdk_type: string;
		current_version?: string;
		path_managed_by: 'app' | 'system' | 'none';
		binaries_in_path: string[];
		environment_variables: EnvironmentVariable[];
		last_updated: string;
	}

	interface EnvironmentVariable {
		name: string;
		value: string;
		scope: 'global' | 'session' | 'project' | 'service';
		is_exported: boolean;
		created_at: string;
		updated_at: string;
	}

	let { sdkType, showDetails = false }: { sdkType: string; showDetails?: boolean } = $props();

	// State
	let status = $state<PathStatus | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let lastUpdate = $state<string | null>(null);

	// Initialize
	onMount(() => {
		loadPathStatus();
	});

	async function loadPathStatus() {
		loading = true;
		error = null;
		
		try {
			const result = await invoke('get_path_status', { sdkType });
			status = result as PathStatus;
			lastUpdate = new Date().toISOString();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load PATH status';
			console.error('Failed to load PATH status:', err);
		} finally {
			loading = false;
		}
	}

	async function setPathEnvironment() {
		if (!status) return;
		
		loading = true;
		error = null;

		try {
			await invoke('set_path_environment', {
				sdkType,
				version: status.current_version || 'latest',
				scope: 'session'
			});
			await loadPathStatus();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to set PATH environment';
			console.error('Failed to set PATH environment:', err);
		} finally {
			loading = false;
		}
	}

	function getPathManagementIcon(managedBy: string) {
		switch (managedBy) {
			case 'app': return Shield;
			case 'system': return Settings;
			case 'none': return WifiOff;
			default: return AlertTriangle;
		}
	}

	function getPathManagementColor(managedBy: string) {
		switch (managedBy) {
			case 'app': return 'text-blue-600';
			case 'system': return 'text-yellow-600';
			case 'none': return 'text-gray-600';
			default: return 'text-red-600';
		}
	}

	function getPathManagementBadgeVariant(managedBy: string) {
		switch (managedBy) {
			case 'app': return 'default';
			case 'system': return 'secondary';
			case 'none': return 'outline';
			default: return 'destructive';
		}
	}

	function getPathManagementText(managedBy: string) {
		switch (managedBy) {
			case 'app': return 'App Managed';
			case 'system': return 'System Managed';
			case 'none': return 'Not in PATH';
			default: return 'Unknown';
		}
	}

	function formatLastUpdated(timestamp: string) {
		return new Date(timestamp).toLocaleString();
	}
</script>

<Card class="w-full">
	<CardHeader class="pb-3">
		<div class="flex items-center justify-between">
			<CardTitle class="text-lg flex items-center gap-2">
				PATH Status
				{#if loading}
					<RefreshCw class="w-4 h-4 animate-spin" />
				{/if}
			</CardTitle>
			<Button variant="outline" size="sm" onclick={loadPathStatus} disabled={loading}>
				<RefreshCw class="w-4 h-4" />
			</Button>
		</div>
	</CardHeader>
	
	<CardContent class="space-y-4">
		{#if error}
			<Alert variant="destructive">
				<AlertTriangle class="w-4 h-4" />
				<AlertDescription>{error}</AlertDescription>
			</Alert>
		{:else if loading && !status}
			<div class="flex items-center justify-center py-4">
				<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
				<span class="ml-2 text-sm text-muted-foreground">Loading PATH status...</span>
			</div>
		{:else if status}
			<!-- Main Status -->
			<div class="flex items-center gap-3">
				{#if status.path_managed_by === 'none'}
					<XCircle class="w-6 h-6 text-red-500" />
				{:else}
					<CheckCircle class="w-6 h-6 text-green-500" />
				{/if}
				
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<Badge variant={getPathManagementBadgeVariant(status.path_managed_by)}>
							{@const ManagementIcon = getPathManagementIcon(status.path_managed_by)}
							<ManagementIcon class="w-3 h-3 mr-1" />
							{getPathManagementText(status.path_managed_by)}
						</Badge>
						
						{#if status.current_version}
							<Badge variant="outline">
								{status.current_version}
							</Badge>
						{/if}
					</div>
					
					{#if lastUpdate}
						<p class="text-xs text-muted-foreground mt-1">
							Last updated: {formatLastUpdated(lastUpdate)}
						</p>
					{/if}
				</div>
			</div>

			<!-- Action Button -->
			{#if status.path_managed_by === 'none'}
				<Button onclick={setPathEnvironment} disabled={loading} class="w-full">
					<Shield class="w-4 h-4 mr-2" />
					Add to PATH
				</Button>
			{/if}

			<!-- Detailed Information -->
			{#if showDetails}
				<div class="space-y-4 pt-4 border-t">
					<!-- Binaries in PATH -->
					{#if status.binaries_in_path.length > 0}
						<div>
							<h4 class="text-sm font-medium mb-2">Binaries in PATH</h4>
							<div class="flex flex-wrap gap-1">
								{#each status.binaries_in_path as binary}
									<Badge variant="secondary" class="text-xs">
										{binary}
									</Badge>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Environment Variables -->
					{#if status.environment_variables.length > 0}
						<div>
							<h4 class="text-sm font-medium mb-2">Environment Variables</h4>
							<div class="space-y-2">
								{#each status.environment_variables as envVar}
									<div class="flex items-center justify-between p-2 bg-muted rounded">
										<div class="flex-1 min-w-0">
											<div class="flex items-center gap-2">
												<code class="text-sm font-mono">{envVar.name}</code>
												{#if envVar.is_exported}
													<Badge variant="outline" class="text-xs">exported</Badge>
												{/if}
											</div>
											<code class="text-xs text-muted-foreground truncate block">
												{envVar.value}
											</code>
										</div>
										<Badge variant="outline" class="text-xs">
											{envVar.scope}
										</Badge>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Management Info -->
					<div class="p-3 bg-muted rounded">
						<div class="flex items-center gap-2 mb-2">
							{#if status.path_managed_by === 'app'}
								<Shield class="w-4 h-4 text-blue-600" />
								<span class="text-sm font-medium">App Managed</span>
							{:else if status.path_managed_by === 'system'}
								<Settings class="w-4 h-4 text-yellow-600" />
								<span class="text-sm font-medium">System Managed</span>
							{:else}
								<WifiOff class="w-4 h-4 text-gray-600" />
								<span class="text-sm font-medium">Not in PATH</span>
							{/if}
						</div>
						
						<p class="text-xs text-muted-foreground">
							{#if status.path_managed_by === 'app'}
								This SDK is managed by Portal Desktop. Changes will be applied to your shell configuration.
							{:else if status.path_managed_by === 'system'}
								This SDK is managed by your system. Portal Desktop cannot modify the PATH for this SDK.
							{:else}
								This SDK is not currently in your PATH. Click "Add to PATH" to enable it.
							{/if}
						</p>
					</div>
				</div>
			{/if}
		{:else}
			<div class="text-center py-4 text-muted-foreground">
				<AlertTriangle class="w-8 h-8 mx-auto mb-2 opacity-50" />
				<p>No PATH status available</p>
			</div>
		{/if}
	</CardContent>
</Card>
