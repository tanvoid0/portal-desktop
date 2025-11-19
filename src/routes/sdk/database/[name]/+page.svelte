<!--
	Database SDK Detail Page
	Shows details for a specific database SDK
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { CheckCircle, XCircle, Download, Play, Square } from '@lucide/svelte';

	// Get database name from URL
	let databaseName = $derived($page.params.name);
	
	// State
	let loading = $state(true);
	let error = $state<string | null>(null);
	let database = $state<any>(null);
	let isInstalled = $state(false);
	let isRunning = $state(false);

	// Initialize data
	onMount(async () => {
		await loadDatabaseDetails();
	});

	async function loadDatabaseDetails() {
		loading = true;
		error = null;
		
		try {
			// Load database details from backend
			const sdkDetails = await invoke('get_sdk_details', { sdkType: databaseName });
			
			if (!sdkDetails) {
				error = `Database '${databaseName}' not found`;
				return;
			}
			
			database = sdkDetails;
			
			// Check installation status
			try {
				const result = await invoke<[boolean, string | null]>('check_sdk_status', { sdkName: databaseName });
				const [installed, version] = result;
				isInstalled = installed;
			} catch {
				isInstalled = false;
			}
			
			// Check if running
			try {
				const serviceStatus = await invoke<{ running?: boolean }>('get_service_status', { sdkType: databaseName });
				isRunning = serviceStatus?.running || false;
			} catch {
				isRunning = false;
			}
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load database details';
			console.error('Failed to load database details:', err);
		} finally {
			loading = false;
		}
	}

	async function startDatabase() {
		try {
			const config = {
				port: database.default_port,
				host: 'localhost',
				data_dir: null,
				config_file: null,
				environment: {}
			};
			await invoke('start_sdk_service', { 
				sdkType: databaseName, 
				version: 'latest',
				config
			});
			isRunning = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to start database';
		}
	}

	async function stopDatabase() {
		try {
			await invoke('stop_sdk_service', { 
				sdkType: databaseName, 
				pid: 12345 // FUTURE: Get actual PID from service status
			});
			isRunning = false;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to stop database';
		}
	}
</script>

<svelte:head>
	<title>{database?.name || 'Database'} - Portal Desktop</title>
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
{:else if database}
	<div class="space-y-6 p-6 w-full max-w-none">
		<!-- Header -->
		<div class="flex items-center gap-4">
			<div class="flex-1">
				<h1 class="text-3xl font-bold">{database.name}</h1>
				<p class="text-muted-foreground">{database.description}</p>
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
				
				{#if isRunning}
					<Badge variant="default" class="bg-blue-100 text-blue-800">
						<Play class="w-3 h-3 mr-1" />
						Running
					</Badge>
				{:else}
					<Badge variant="outline" class="text-gray-500">
						<Square class="w-3 h-3 mr-1" />
						Stopped
					</Badge>
				{/if}
			</div>
		</div>

		<!-- Database Details -->
		<div class="flex flex-col gap-6 lg:flex-row">
			<Card>
				<CardHeader>
					<CardTitle>Database Information</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<p class="text-sm font-medium">Category</p>
						<p class="text-sm text-muted-foreground">{database.category}</p>
					</div>
					<div>
						<p class="text-sm font-medium">Default Port</p>
						<p class="text-sm text-muted-foreground">{database.default_port || 'N/A'}</p>
					</div>
					<div>
						<p class="text-sm font-medium">Website</p>
						<a 
							href={database.website} 
							target="_blank" 
							class="text-sm text-blue-600 hover:underline"
						>
							{database.website}
						</a>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Features</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex flex-wrap gap-2">
						{#each database.features as feature}
							<Badge variant="secondary" class="text-xs">{feature}</Badge>
						{/each}
					</div>
				</CardContent>
			</Card>
		</div>

		<!-- Service Management -->
		{#if isInstalled}
			<Card>
				<CardHeader>
					<CardTitle>Service Management</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="flex items-center gap-4">
						{#if isRunning}
							<Button 
								variant="outline" 
								onclick={stopDatabase}
							>
								<Square class="w-4 h-4 mr-2" />
								Stop Database
							</Button>
							<Button 
								variant="outline"
								onclick={() => window.open(`http://localhost:${database.default_port}`, '_blank')}
							>
								Open Connection
							</Button>
						{:else}
							<Button 
								onclick={startDatabase}
							>
								<Play class="w-4 h-4 mr-2" />
								Start Database
							</Button>
						{/if}
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
{/if}
