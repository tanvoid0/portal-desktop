<!--
	SDK Package Manager Page
	Package manager management for a specific SDK
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { invokeClient } from '@/lib/utils/invokeClient';
	import { sdkConfigService, type ProcessedSDKConfig } from '$lib/domains/sdk/services/sdkConfigService';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		CheckCircle,
		XCircle,
		AlertCircle,
		RefreshCw,
		ArrowLeft,
		Settings
	} from '@lucide/svelte';
	import { goto } from '$app/navigation';

	// Get SDK ID from URL
	let sdkId = $derived($page.params.sdk);
	
	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let sdkConfig = $state<ProcessedSDKConfig | null>(null);

	// Initialize data
	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		
		try {
			if (!sdkId) {
				error = 'SDK ID is required';
				return;
			}
			
			const config = await sdkConfigService.getSDKConfig(sdkId);
			if (!config) {
				error = `SDK '${sdkId}' not found`;
				return;
			}
			
			sdkConfig = config;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load SDK configuration';
			console.error('Failed to load SDK config:', err);
		} finally {
			loading = false;
		}
	}
</script>

<div class="p-6">
	{#if loading}
		<div class="flex items-center justify-center h-64">
			<RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>{error}</AlertDescription>
		</Alert>
	{:else if sdkConfig}
		<div class="space-y-6">
			<!-- Header -->
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Button variant="ghost" size="sm" onclick={() => goto(`/sdk/${sdkId}`)}>
						<ArrowLeft class="h-4 w-4 mr-2" />
						Back
					</Button>
					<div>
						<h1 class="text-3xl font-bold">Package Managers</h1>
						<p class="text-muted-foreground">{sdkConfig.display_name} Package Managers</p>
					</div>
				</div>
			</div>

			<!-- Package Manager Content -->
			<Card>
				<CardHeader>
					<CardTitle>Package Managers</CardTitle>
				</CardHeader>
				<CardContent>
					{#if sdkConfig.package_managers.length > 0}
						<div class="space-y-4">
							{#each sdkConfig.package_managers as pm}
								<div class="flex items-center justify-between p-4 border rounded-lg">
									<div>
										<h3 class="font-semibold">{pm.display_name}</h3>
										<p class="text-sm text-muted-foreground">
											{pm.installed ? `Installed (${pm.version || 'unknown'})` : 'Not installed'}
										</p>
									</div>
									<div class="flex items-center gap-2">
										{#if pm.installed}
											<CheckCircle class="h-5 w-5 text-green-500" />
										{:else}
											<XCircle class="h-5 w-5 text-muted-foreground" />
										{/if}
										{#if pm.website}
											<a href={pm.website} target="_blank" rel="noopener noreferrer">
												<Button variant="ghost" size="sm">
													<Settings class="h-4 w-4" />
												</Button>
											</a>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-muted-foreground">No package managers configured for this SDK.</p>
					{/if}
				</CardContent>
			</Card>
		</div>
	{/if}
</div>

