<!--
	Node.js SDK Detail Page - FlyEnv-style tabbed interface
	Service management, version management, configuration, and projects
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		Code, 
		Settings, 
		Database, 
		FolderOpen,
		Play,
		Square,
		Download,
		Trash2,
		CheckCircle,
		XCircle,
		AlertCircle,
		RefreshCw
	} from 'lucide-svelte';
	import ServiceCard from '$lib/domains/sdk/components/ServiceCard.svelte';
	import VersionList from '$lib/domains/sdk/components/VersionList.svelte';

	// State
	let loading = $state(false);
	let error = $state<string | null>(null);
	let activeTab = $state('service');
	
	// Service data
	let services = $state<any[]>([]);
	let serviceRunning = $state(false);
	
	// Version data
	let versions = $state<any[]>([]);
	let availableVersions = $state<any[]>([]);
	
	// Configuration data
	let configFiles = $state<any[]>([]);
	let environmentVars = $state<any[]>([]);
	
	// Project data (for future implementation)
	let projects = $state<any[]>([]);

	// Initialize data
	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		
		try {
			// Load services
			const serviceStatus = await invoke('get_service_status', { sdkType: 'nodejs' });
			services = Array.isArray(serviceStatus) ? serviceStatus : [];
			
			// Load versions
			const versionData = await invoke('fetch_available_versions', { sdkType: 'nodejs' });
			versions = Array.isArray(versionData) ? versionData : [];
			availableVersions = versions.filter(v => !v.installed);
			
			// Load configuration files
			configFiles = [
				{
					name: 'package.json',
					path: '/path/to/project/package.json',
					exists: true,
					lastModified: new Date().toISOString()
				},
				{
					name: '.nvmrc',
					path: '/path/to/project/.nvmrc',
					exists: true,
					lastModified: new Date().toISOString()
				}
			];
			
			// Load environment variables
			environmentVars = [
				{ name: 'NODE_ENV', value: 'development', scope: 'project' },
				{ name: 'NODE_PATH', value: '/usr/local/lib/node_modules', scope: 'global' }
			];
			
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load Node.js data';
			console.error('Failed to load Node.js data:', err);
		} finally {
			loading = false;
		}
	}

	// Service management
	async function toggleService(service: any) {
		try {
			if (service.status === 'running') {
				await invoke('stop_sdk_service', { 
					sdkType: 'nodejs', 
					pid: service.pid 
				});
				service.status = 'stopped';
				service.pid = null;
			} else {
				const config = {
					port: service.port || 3000,
					host: 'localhost',
					data_dir: null,
					config_file: null,
					environment: {}
				};
				const pid = await invoke('start_sdk_service', { 
					sdkType: 'nodejs', 
					version: service.version,
					config
				});
				service.status = 'running';
				service.pid = pid;
			}
		} catch (err) {
			service.status = 'error';
			error = err instanceof Error ? err.message : 'Failed to toggle service';
		}
	}

	// Version management
	async function installVersion(version: any) {
		version.downloading = true;
		version.progress = 0;
		
		try {
			await invoke('download_and_install_version', { 
				sdkType: 'nodejs', 
				version: version.version,
				use_manager: false
			});
			
			version.downloading = false;
			version.installed = true;
			await loadData();
		} catch (err) {
			version.downloading = false;
			version.error = err instanceof Error ? err.message : 'Installation failed';
		}
	}

	async function uninstallVersion(version: any) {
		try {
			await invoke('uninstall_sdk_version', { 
				sdkType: 'nodejs', 
				version: version.version 
			});
			version.installed = false;
			await loadData();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to uninstall version';
		}
	}

	async function setActiveVersion(version: any) {
		try {
			await invoke('switch_sdk_version', { 
				sdkType: 'nodejs', 
				version: version.version 
			});
			versions.forEach(v => v.active = false);
			version.active = true;
			await loadData();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to set active version';
		}
	}

	// Configuration management
	async function openConfigFile(configFile: any) {
		// TODO: Open file in editor
		console.log('Opening config file:', configFile.path);
	}

	async function editEnvironmentVar(envVar: any) {
		// TODO: Open environment variable editor
		console.log('Editing environment variable:', envVar.name);
	}
</script>

<svelte:head>
	<title>Node.js SDK - Portal Desktop</title>
</svelte:head>

<div class="space-y-6 p-6 w-full max-w-none">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<div class="w-12 h-12 rounded-lg bg-gradient-to-br from-green-500 to-green-600 flex items-center justify-center text-white font-bold text-xl">
				N
			</div>
			<div>
				<h1 class="text-3xl font-bold">Node.js SDK</h1>
				<p class="text-muted-foreground">
					Manage Node.js versions, services, and project configurations
				</p>
			</div>
		</div>
		<div class="flex items-center gap-2">
			<Button variant="outline" onclick={loadData} disabled={loading}>
				<RefreshCw class="w-4 h-4 mr-2" />
				Refresh
			</Button>
		</div>
	</div>

	<!-- Error Alert -->
	{#if error}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>{error}</AlertDescription>
		</Alert>
	{/if}

	<!-- Main Content Tabs -->
	<Tabs bind:value={activeTab} class="w-full">
		<TabsList class="grid w-full grid-cols-4">
			<TabsTrigger value="service">
				<Database class="w-4 h-4 mr-2" />
				Service
			</TabsTrigger>
			<TabsTrigger value="versions">
				<Code class="w-4 h-4 mr-2" />
				Version Manager
			</TabsTrigger>
			<TabsTrigger value="config">
				<Settings class="w-4 h-4 mr-2" />
				Configuration
			</TabsTrigger>
			<TabsTrigger value="projects">
				<FolderOpen class="w-4 h-4 mr-2" />
				Projects
			</TabsTrigger>
		</TabsList>

		<!-- Service Tab -->
		<TabsContent value="service" class="space-y-6">
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<Database class="w-5 h-5" />
						Node.js Services
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="space-y-4">
						{#each services as service}
							<ServiceCard 
								{service}
								availableVersions={versions.map(v => v.version)}
								onToggle={toggleService}
								onVersionChange={(s, v) => console.log('Version change:', s, v)}
								onConfigure={(s) => console.log('Configure:', s)}
								onViewLogs={(s) => console.log('View logs:', s)}
								onOpenUrl={(s) => console.log('Open URL:', s)}
							/>
						{:else}
							<div class="text-center py-8 text-muted-foreground">
								<Database class="w-12 h-12 mx-auto mb-4 opacity-50" />
								<p>No Node.js services running</p>
								<p class="text-sm">Start a service to see it here</p>
							</div>
						{/each}
					</div>
				</CardContent>
			</Card>
		</TabsContent>

		<!-- Version Manager Tab -->
		<TabsContent value="versions" class="space-y-6">
			<VersionList 
				{versions}
				onInstall={installVersion}
				onUninstall={uninstallVersion}
				onSetActive={setActiveVersion}
				{loading}
			/>
		</TabsContent>

		<!-- Configuration Tab -->
		<TabsContent value="config" class="space-y-6">
			<div class="grid gap-6 md:grid-cols-2">
				<!-- Configuration Files -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Settings class="w-5 h-5" />
							Configuration Files
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="space-y-3">
							{#each configFiles as configFile}
								<div class="flex items-center justify-between p-3 border rounded-lg">
									<div class="flex items-center gap-3">
										<Badge variant={configFile.exists ? 'default' : 'secondary'}>
											{configFile.exists ? 'Exists' : 'Missing'}
										</Badge>
										<div>
											<div class="font-medium">{configFile.name}</div>
											<div class="text-sm text-muted-foreground">{configFile.path}</div>
										</div>
									</div>
									<Button 
										variant="outline" 
										size="sm"
										onclick={() => openConfigFile(configFile)}
									>
										Open
									</Button>
								</div>
							{/each}
						</div>
					</CardContent>
				</Card>

				<!-- Environment Variables -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Settings class="w-5 h-5" />
							Environment Variables
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="space-y-3">
							{#each environmentVars as envVar}
								<div class="flex items-center justify-between p-3 border rounded-lg">
									<div>
										<div class="font-medium">{envVar.name}</div>
										<div class="text-sm text-muted-foreground">{envVar.value}</div>
										<Badge variant="outline" class="text-xs mt-1">{envVar.scope}</Badge>
									</div>
									<Button 
										variant="outline" 
										size="sm"
										onclick={() => editEnvironmentVar(envVar)}
									>
										Edit
									</Button>
								</div>
							{/each}
						</div>
					</CardContent>
				</Card>
			</div>
		</TabsContent>

		<!-- Projects Tab -->
		<TabsContent value="projects" class="space-y-6">
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<FolderOpen class="w-5 h-5" />
						Node.js Projects
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="text-center py-8 text-muted-foreground">
						<FolderOpen class="w-12 h-12 mx-auto mb-4 opacity-50" />
						<p>Project management coming soon</p>
						<p class="text-sm">This feature will be implemented in a future update</p>
					</div>
				</CardContent>
			</Card>
		</TabsContent>
	</Tabs>
</div>
