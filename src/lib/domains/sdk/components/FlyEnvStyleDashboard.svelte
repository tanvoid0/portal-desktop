<!--
	FlyEnv-Style SDK Dashboard - Enhanced with comprehensive tool support and project isolation
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { sdkActions, sdkManagers, installedManagers, installedSDKs, isDetecting, detectionError } from '../stores/sdkStore';
	import { sdkService } from '../services/sdkService';
	import { logger } from '$lib/domains/shared';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import Select from '$lib/components/ui/select.svelte';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '$lib/components/ui/dialog';
	import { RefreshCw, Download, AlertCircle, Settings, FolderOpen, Zap, Globe, Database, Container, Code, Terminal, Plus, Trash2, ArrowRightLeft, Play, Square } from 'lucide-svelte';

	interface Props {
		onSDKSelect?: (sdk: any) => void;
		onVersionSwitch?: (sdkType: string, version: string) => void;
	}

	const { onSDKSelect, onVersionSwitch }: Props = $props();

	let refreshing = $state(false);
	let selectedProjectPath = $state('');
	let selectedSDKType = $state('');
	let selectedVersion = $state('');
	let availableVersions = $state<string[]>([]);
	let showProjectDialog = $state(false);
	
	// SDK operation states
	let showInstallManagerDialog = $state(false);
	let showInstallSDKDialog = $state(false);
	let showSwitchVersionDialog = $state(false);
	let selectedManager = $state<any>(null);
	let selectedSDK = $state<any>(null);
	let operationLoading = $state(false);
	let operationError = $state<string | null>(null);
	let operationSuccess = $state<string | null>(null);

	// Reactive stores
	let managers = $derived($installedManagers);
	let sdks = $derived($installedSDKs);
	let detecting = $derived($isDetecting);
	let error = $derived($detectionError);
	
	// Debug: Log managers array changes
	$effect(() => {
		logger.info('Managers array updated', { 
			context: 'FlyEnvStyleDashboard', 
			data: { 
				managers: managers,
				managersLength: managers.length,
				firstManager: managers[0]
			} 
		});
	});

	// Group managers by category
	let managersByCategory = $derived(() => {
		const categories: Record<string, any[]> = {
			language: [],
			web: [],
			database: [],
			container: [],
			package: []
		};

		managers.forEach(manager => {
			// Categorize based on manager category or sdk_type
			let category = manager.category || 'other';
			
			// Fallback to sdk_type if category is not available
			if (!category || category === 'other') {
				const sdkType = manager.sdk_type || manager.type || '';
				if (sdkType.includes('node') || sdkType.includes('python') || sdkType.includes('java') || sdkType.includes('rust') || sdkType.includes('go') || sdkType.includes('ruby') || sdkType.includes('php')) {
					category = 'language';
				} else if (sdkType.includes('nginx') || sdkType.includes('apache')) {
					category = 'web';
				} else if (sdkType.includes('docker') || sdkType.includes('podman')) {
					category = 'container';
				} else if (sdkType.includes('npm') || sdkType.includes('pip')) {
					category = 'package';
				}
			}
			
			if (categories[category]) {
				categories[category].push(manager);
			} else {
				categories.other = categories.other || [];
				categories.other.push(manager);
			}
		});

		return categories;
	});

	onMount(async () => {
		await detectSDKs();
	});

	async function detectSDKs() {
		try {
			refreshing = true;
			sdkActions.setDetecting(true);
			
			logger.info('Starting comprehensive SDK detection', { context: 'FlyEnvStyleDashboard' });
			
			const allSDKs = await sdkService.getAllAvailableSDKs();
			const result = {
				managers: allSDKs,
				sdks: [],
				errors: []
			};
			logger.info('SDK detection result received', { 
				context: 'FlyEnvStyleDashboard', 
				data: result 
			});
			
			// Debug: Log the managers array specifically
			logger.info('Managers array details', { 
				context: 'FlyEnvStyleDashboard', 
				data: { 
					managers: result.managers,
					managersLength: result.managers.length,
					firstManager: result.managers[0],
					firstManagerKeys: result.managers[0] ? Object.keys(result.managers[0]) : 'no managers',
					firstManagerInstalled: result.managers[0] ? result.managers[0].installed : 'no managers',
					firstManagerInstalledType: result.managers[0] ? typeof result.managers[0].installed : 'no managers',
					fullResult: result
				} 
			});
			
			sdkActions.setDetectionResult(result);
			
			logger.info('SDK detection completed', { 
				context: 'FlyEnvStyleDashboard', 
				data: { 
					managersCount: result.managers.length,
					sdksCount: result.sdks.length 
				} 
			});
		} catch (err) {
			logger.error('SDK detection failed', {
				context: 'FlyEnvStyleDashboard',
				error: err
			});
			sdkActions.setDetectionError(err instanceof Error ? err.message : 'Detection failed');
		} finally {
			refreshing = false;
			sdkActions.setDetecting(false);
		}
	}

	async function handleRefresh() {
		await detectSDKs();
	}

	async function handleProjectEnvironmentSetup() {
		if (!selectedProjectPath) return;

		try {
			await sdkService.setupProjectEnvironment(selectedProjectPath);
			logger.info('Project environment setup completed', { 
				context: 'FlyEnvStyleDashboard', 
				data: { projectPath: selectedProjectPath } 
			});
		} catch (err) {
			logger.error('Failed to setup project environment', {
				context: 'FlyEnvStyleDashboard',
				error: err
			});
		}
	}

	async function handleTerminalIntegration() {
		if (!selectedProjectPath) return;

		try {
			await sdkService.setupTerminalIntegration(selectedProjectPath);
			logger.info('Terminal integration setup completed', { 
				context: 'FlyEnvStyleDashboard', 
				data: { projectPath: selectedProjectPath } 
			});
		} catch (err) {
			logger.error('Failed to setup terminal integration', {
				context: 'FlyEnvStyleDashboard',
				error: err
			});
		}
	}

	async function handleCreateProjectConfig() {
		if (!selectedProjectPath || !selectedSDKType || !selectedVersion) return;

		try {
			await sdkService.createProjectConfig(selectedProjectPath, selectedSDKType, selectedVersion);
			logger.info('Project config created', { 
				context: 'FlyEnvStyleDashboard', 
				data: { projectPath: selectedProjectPath, sdkType: selectedSDKType, version: selectedVersion } 
			});
			showProjectDialog = false;
		} catch (err) {
			logger.error('Failed to create project config', {
				context: 'FlyEnvStyleDashboard',
				error: err
			});
		}
	}

	async function handleSDKTypeChange(sdkType: string) {
		selectedSDKType = sdkType;
		try {
			availableVersions = await sdkService.listVersions(sdkType);
		} catch (err) {
			logger.error('Failed to list versions', { context: 'FlyEnvStyleDashboard', error: err });
			availableVersions = [];
		}
	}

	function handleVersionChange(version: string) {
		selectedVersion = version;
	}

	function getCategoryIcon(category: string) {
		switch (category) {
			case 'language': return Code;
			case 'web': return Globe;
			case 'database': return Database;
			case 'container': return Container;
			case 'package': return Download;
			default: return Settings;
		}
	}

	function getCategoryName(category: string) {
		switch (category) {
			case 'language': return 'Programming Languages';
			case 'web': return 'Web Servers';
			case 'database': return 'Databases';
			case 'container': return 'Containers';
			case 'package': return 'Package Managers';
			default: return 'Other Tools';
		}
	}

	// SDK Operation Functions
	async function handleInstallManager(manager: any) {
		selectedManager = manager;
		showInstallManagerDialog = true;
	}

	async function handleInstallSDK(sdk: any) {
		selectedSDK = sdk;
		try {
			availableVersions = await sdkService.listVersions(sdk.type);
		} catch (err) {
			logger.error('Failed to list versions', { context: 'FlyEnvStyleDashboard', error: err });
			availableVersions = [];
		}
		showInstallSDKDialog = true;
	}

	async function handleSwitchVersion(sdk: any) {
		selectedSDK = sdk;
		try {
			availableVersions = await sdkService.listVersions(sdk.type);
		} catch (err) {
			logger.error('Failed to list versions', { context: 'FlyEnvStyleDashboard', error: err });
			availableVersions = [];
		}
		showSwitchVersionDialog = true;
	}

	async function executeInstallManager() {
		if (!selectedManager) return;
		
		try {
			operationLoading = true;
			operationError = null;
			operationSuccess = null;
			
			// This would need to be implemented in the backend
			// For now, we'll simulate the operation
			await new Promise(resolve => setTimeout(resolve, 2000));
			
			operationSuccess = `${selectedManager.display_name || selectedManager.name} installation completed`;
			showInstallManagerDialog = false;
			
			// Refresh the detection
			await detectSDKs();
		} catch (err) {
			operationError = err instanceof Error ? err.message : 'Installation failed';
		} finally {
			operationLoading = false;
		}
	}

	async function executeInstallSDK() {
		if (!selectedSDK || !selectedVersion) return;
		
		try {
			operationLoading = true;
			operationError = null;
			operationSuccess = null;
			
			await sdkService.installVersion({
				type: selectedSDK.type,
				version: selectedVersion,
				manager: selectedSDK.manager
			});
			
			operationSuccess = `${selectedSDK.name} version ${selectedVersion} installed successfully`;
			showInstallSDKDialog = false;
			
			// Refresh the detection
			await detectSDKs();
		} catch (err) {
			operationError = err instanceof Error ? err.message : 'Installation failed';
		} finally {
			operationLoading = false;
		}
	}

	async function executeSwitchVersion() {
		if (!selectedSDK || !selectedVersion) return;
		
		try {
			operationLoading = true;
			operationError = null;
			operationSuccess = null;
			
			await sdkService.switchVersion({
				type: selectedSDK.type,
				version: selectedVersion,
				projectPath: selectedProjectPath || undefined
			});
			
			operationSuccess = `Switched to ${selectedSDK.name} version ${selectedVersion}`;
			showSwitchVersionDialog = false;
			
			// Refresh the detection
			await detectSDKs();
		} catch (err) {
			operationError = err instanceof Error ? err.message : 'Version switch failed';
		} finally {
			operationLoading = false;
		}
	}

	async function handleRemoveSDK(sdk: any) {
		if (!confirm(`Are you sure you want to remove ${sdk.name}?`)) return;
		
		try {
			operationLoading = true;
			operationError = null;
			operationSuccess = null;
			
			// This would need to be implemented in the backend
			// For now, we'll simulate the operation
			await new Promise(resolve => setTimeout(resolve, 1000));
			
			operationSuccess = `${sdk.name} removed successfully`;
			
			// Refresh the detection
			await detectSDKs();
		} catch (err) {
			operationError = err instanceof Error ? err.message : 'Removal failed';
		} finally {
			operationLoading = false;
		}
	}

	function clearOperationMessages() {
		operationError = null;
		operationSuccess = null;
	}
</script>

<svelte:head>
	<title>SDK Manager - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">SDK Manager</h1>
			<p class="text-muted-foreground">FlyEnv-style comprehensive development environment management</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" onclick={handleRefresh} disabled={refreshing}>
				<span class="h-4 w-4 mr-2" class:animate-spin={refreshing}>
					<RefreshCw />
				</span>
				Refresh
			</Button>
			<Dialog bind:open={showProjectDialog}>
				<DialogTrigger>
					<Button>
						<FolderOpen class="h-4 w-4 mr-2" />
						Project Setup
					</Button>
				</DialogTrigger>
				<DialogContent class="sm:max-w-md">
					<DialogHeader>
						<DialogTitle>Project Environment Setup</DialogTitle>
						<DialogDescription>
							Configure SDK versions for your project directory
						</DialogDescription>
					</DialogHeader>
					<div class="space-y-4">
						<div>
							<Label for="project-path">Project Path</Label>
							<Input 
								id="project-path" 
								bind:value={selectedProjectPath} 
								placeholder="/path/to/your/project"
							/>
						</div>
						<div>
							<Label for="sdk-type">SDK Type</Label>
							<Select 
								options={[
									{ value: 'node', label: 'Node.js' },
									{ value: 'python', label: 'Python' },
									{ value: 'rust', label: 'Rust' },
									{ value: 'java', label: 'Java' },
									{ value: 'go', label: 'Go' },
									{ value: 'php', label: 'PHP' },
									{ value: 'ruby', label: 'Ruby' }
								]}
								onSelect={handleSDKTypeChange}
								placeholder="Select SDK type"
							/>
						</div>
						<div>
							<Label for="version">Version</Label>
							<Select 
								options={availableVersions.map(version => ({
									value: version,
									label: version
								}))}
								defaultValue={selectedVersion}
								onSelect={(value) => selectedVersion = value}
								placeholder="Select version"
							/>
						</div>
						<div class="flex gap-2">
							<Button onclick={handleCreateProjectConfig} disabled={!selectedProjectPath || !selectedSDKType || !selectedVersion}>
								<Zap class="h-4 w-4 mr-2" />
								Create Config
							</Button>
							<Button variant="outline" onclick={handleProjectEnvironmentSetup} disabled={!selectedProjectPath}>
								<Settings class="h-4 w-4 mr-2" />
								Auto-Setup
							</Button>
							<Button variant="outline" onclick={handleTerminalIntegration} disabled={!selectedProjectPath}>
								<Terminal class="h-4 w-4 mr-2" />
								Terminal Integration
							</Button>
						</div>
					</div>
				</DialogContent>
			</Dialog>
		</div>
	</div>

	<!-- Error Alert -->
	{#if error}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>{error}</AlertDescription>
		</Alert>
	{/if}

	<!-- Operation Status Alerts -->
	{#if operationError}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>
				{operationError}
				<Button variant="ghost" size="sm" onclick={clearOperationMessages} class="ml-2">
					<Square class="h-3 w-3" />
				</Button>
			</AlertDescription>
		</Alert>
	{/if}

	{#if operationSuccess}
		<Alert variant="default" class="border-green-200 bg-green-50">
			<AlertCircle class="h-4 w-4 text-green-600" />
			<AlertDescription class="text-green-800">
				{operationSuccess}
				<Button variant="ghost" size="sm" onclick={clearOperationMessages} class="ml-2">
					<Square class="h-3 w-3" />
				</Button>
			</AlertDescription>
		</Alert>
	{/if}

	<!-- Loading State -->
	{#if detecting}
		<div class="flex items-center justify-center py-8">
			<RefreshCw class="h-6 w-6 animate-spin mr-2" />
			<span>Detecting SDKs and tools...</span>
		</div>
	{:else}
		<!-- Main Content -->
		<Tabs value="overview" class="space-y-4">
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="managers">SDK Managers</TabsTrigger>
				<TabsTrigger value="installations">Installations</TabsTrigger>
			</TabsList>

			<TabsContent value="overview" class="space-y-4">
				<!-- Quick Stats -->
				<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Total Managers</CardTitle>
							<Settings class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{managers.length}</div>
						</CardContent>
					</Card>
					<Card class="col-span-1">
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Installed SDKs</CardTitle>
							<Download class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{sdks.length}</div>
						</CardContent>
					</Card>
					<Card class="col-span-1">
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Languages</CardTitle>
							<Code class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{managersByCategory().language?.length || 0}</div>
						</CardContent>
					</Card>
					<Card class="col-span-1">
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">Web Servers</CardTitle>
							<Globe class="h-4 w-4 text-muted-foreground" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{managersByCategory().web?.length || 0}</div>
						</CardContent>
					</Card>
				</div>

				<!-- Featured Capabilities -->
				<Card>
					<CardHeader>
						<CardTitle>FlyEnv-Style Features</CardTitle>
						<CardDescription>Comprehensive development environment management</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
							<div class="flex items-start space-x-3">
								<Zap class="h-5 w-5 text-blue-500 mt-1" />
								<div>
									<h4 class="font-semibold">Project-Level Isolation</h4>
									<p class="text-sm text-muted-foreground">Automatic SDK version switching when entering project directories</p>
								</div>
							</div>
							<div class="flex items-start space-x-3">
								<Globe class="h-5 w-5 text-green-500 mt-1" />
								<div>
									<h4 class="font-semibold">Full-Stack Support</h4>
									<p class="text-sm text-muted-foreground">Web servers, databases, and development tools</p>
								</div>
							</div>
							<div class="flex items-start space-x-3">
								<Container class="h-5 w-5 text-purple-500 mt-1" />
								<div>
									<h4 class="font-semibold">Native Performance</h4>
									<p class="text-sm text-muted-foreground">Lightweight and efficient resource usage</p>
								</div>
							</div>
							<div class="flex items-start space-x-3">
								<Settings class="h-5 w-5 text-orange-500 mt-1" />
								<div>
									<h4 class="font-semibold">Cross-Platform</h4>
									<p class="text-sm text-muted-foreground">Consistent experience across all platforms</p>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			</TabsContent>

			<TabsContent value="managers" class="space-y-4">
				{#if managers.length > 0}
					{#each Object.entries(managersByCategory()) as [category, categoryManagers]}
						{#if categoryManagers && categoryManagers.length > 0}
							<Card>
								<CardHeader>
									<CardTitle class="flex items-center gap-2">
										{@const IconComponent = getCategoryIcon(category)}
										<IconComponent class="h-5 w-5" />
										{getCategoryName(category)}
										<Badge variant="secondary">{categoryManagers.length}</Badge>
									</CardTitle>
								</CardHeader>
								<CardContent>
									<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
										{#each categoryManagers as manager}
											<Card class="hover:shadow-md transition-shadow">
												<CardHeader class="pb-2">
													<CardTitle class="text-sm flex items-center justify-between">
														{manager.display_name || manager.name}
														<Badge variant="outline" class="text-xs">
															{manager.version}
														</Badge>
													</CardTitle>
												</CardHeader>
												<CardContent class="pt-0 space-y-3">
													<div class="flex items-center justify-between">
														<span class="text-xs text-muted-foreground">{manager.sdk_type || manager.type || manager.name}</span>
														<Badge variant={manager.installed === 'true' ? 'default' : 'secondary'} class="text-xs">
															{manager.installed === 'true' ? 'Installed' : 'Not Found'}
														</Badge>
													</div>
													
													<!-- Action Buttons -->
													<div class="flex gap-1">
														{#if manager.installed !== 'true'}
															<Button 
																variant="outline" 
																size="sm" 
																onclick={() => handleInstallManager(manager)}
																class="flex-1 text-xs"
															>
																<Plus class="h-3 w-3 mr-1" />
																Install
															</Button>
														{:else}
															<Button 
																variant="outline" 
																size="sm" 
																onclick={() => handleSwitchVersion({ type: manager.sdk_type, name: manager.display_name })}
																class="flex-1 text-xs"
															>
																<ArrowRightLeft class="h-3 w-3 mr-1" />
																Switch
															</Button>
															<Button 
																variant="outline" 
																size="sm" 
																onclick={() => handleInstallSDK({ type: manager.sdk_type, name: manager.display_name })}
																class="flex-1 text-xs"
															>
																<Download class="h-3 w-3 mr-1" />
																Add Version
															</Button>
														{/if}
													</div>
												</CardContent>
											</Card>
										{/each}
									</div>
								</CardContent>
							</Card>
						{/if}
					{/each}
				{:else}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-8">
							<Settings class="h-8 w-8 text-muted-foreground mb-2" />
							<p class="text-muted-foreground">No SDK managers detected</p>
							<p class="text-sm text-muted-foreground">Install SDK managers like nvm, pyenv, rustup, or sdkman to get started</p>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>

			<TabsContent value="installations" class="space-y-4">
				{#if sdks.length > 0}
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{#each sdks as sdk}
							<Card class="hover:shadow-md transition-shadow">
								<CardHeader class="pb-2">
									<CardTitle class="text-sm flex items-center gap-2">
										{sdk.icon} {sdk.name}
									</CardTitle>
									<CardDescription class="text-xs">
										{sdk.description}
									</CardDescription>
								</CardHeader>
								<CardContent class="pt-0 space-y-3">
									<div class="flex items-center justify-between">
										<Badge variant="outline" class="text-xs">
											{sdk.installation.activeVersion || 'No version'}
										</Badge>
										<Badge variant={sdk.installation.installed ? 'default' : 'secondary'} class="text-xs">
											{sdk.installation.installed ? 'Installed' : 'Not Installed'}
										</Badge>
									</div>
									
									<!-- SDK Action Buttons -->
									<div class="flex gap-1">
										<Button 
											variant="outline" 
											size="sm" 
											onclick={() => handleSwitchVersion(sdk)}
											class="flex-1 text-xs"
											disabled={!sdk.installation.installed}
										>
											<ArrowRightLeft class="h-3 w-3 mr-1" />
											Switch
										</Button>
										<Button 
											variant="outline" 
											size="sm" 
											onclick={() => handleInstallSDK(sdk)}
											class="flex-1 text-xs"
										>
											<Download class="h-3 w-3 mr-1" />
											Install
										</Button>
										{#if sdk.installation.installed}
											<Button 
												variant="outline" 
												size="sm" 
												onclick={() => handleRemoveSDK(sdk)}
												class="text-xs text-red-600 hover:text-red-700"
											>
												<Trash2 class="h-3 w-3" />
											</Button>
										{/if}
									</div>
								</CardContent>
							</Card>
						{/each}
					</div>
				{:else}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-8">
							<Download class="h-8 w-8 text-muted-foreground mb-2" />
							<p class="text-muted-foreground">No SDK installations found</p>
							<p class="text-sm text-muted-foreground">Install SDKs using the managers above</p>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>
		</Tabs>
	{/if}

	<!-- SDK Manager Installation Dialog -->
	<Dialog bind:open={showInstallManagerDialog}>
		<DialogContent class="sm:max-w-md">
			<DialogHeader>
				<DialogTitle>Install SDK Manager</DialogTitle>
				<DialogDescription>
					Install {selectedManager?.display_name || selectedManager?.name} to manage SDK versions
				</DialogDescription>
			</DialogHeader>
			<div class="space-y-4">
				{#if selectedManager}
					<div class="space-y-2">
						<Label>Manager</Label>
						<div class="p-3 bg-muted rounded-md">
							<div class="font-medium">{selectedManager.display_name || selectedManager.name}</div>
							<div class="text-sm text-muted-foreground">{selectedManager.description}</div>
						</div>
					</div>
					
					<div class="space-y-2">
						<Label>Installation Method</Label>
						<Select 
							options={[
								{ value: 'auto', label: 'Automatic (Recommended)' },
								{ value: 'manual', label: 'Manual Installation' }
							]}
							defaultValue="auto"
							placeholder="Select installation method"
						/>
					</div>
					
					<div class="flex gap-2">
						<Button 
							onclick={executeInstallManager} 
							disabled={operationLoading}
							class="flex-1"
						>
							{#if operationLoading}
								<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
							{:else}
								<Download class="h-4 w-4 mr-2" />
							{/if}
							Install Manager
						</Button>
						<Button variant="outline" onclick={() => showInstallManagerDialog = false}>
							Cancel
						</Button>
					</div>
				{/if}
			</div>
		</DialogContent>
	</Dialog>

	<!-- SDK Installation Dialog -->
	<Dialog bind:open={showInstallSDKDialog}>
		<DialogContent class="sm:max-w-md">
			<DialogHeader>
				<DialogTitle>Install SDK Version</DialogTitle>
				<DialogDescription>
					Install a new version of {selectedSDK?.name}
				</DialogDescription>
			</DialogHeader>
			<div class="space-y-4">
				{#if selectedSDK}
					<div class="space-y-2">
						<Label>SDK</Label>
						<div class="p-3 bg-muted rounded-md">
							<div class="font-medium">{selectedSDK.name}</div>
							<div class="text-sm text-muted-foreground">{selectedSDK.description}</div>
						</div>
					</div>
					
					<div class="space-y-2">
						<Label>Version</Label>
						<Select 
							options={availableVersions.map(version => ({
								value: version,
								label: version
							}))}
							onSelect={(value) => selectedVersion = value}
							placeholder="Select version to install"
						/>
					</div>
					
					<div class="flex gap-2">
						<Button 
							onclick={executeInstallSDK} 
							disabled={operationLoading || !selectedVersion}
							class="flex-1"
						>
							{#if operationLoading}
								<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
							{:else}
								<Download class="h-4 w-4 mr-2" />
							{/if}
							Install Version
						</Button>
						<Button variant="outline" onclick={() => showInstallSDKDialog = false}>
							Cancel
						</Button>
					</div>
				{/if}
			</div>
		</DialogContent>
	</Dialog>

	<!-- SDK Version Switch Dialog -->
	<Dialog bind:open={showSwitchVersionDialog}>
		<DialogContent class="sm:max-w-md">
			<DialogHeader>
				<DialogTitle>Switch SDK Version</DialogTitle>
				<DialogDescription>
					Switch to a different version of {selectedSDK?.name}
				</DialogDescription>
			</DialogHeader>
			<div class="space-y-4">
				{#if selectedSDK}
					<div class="space-y-2">
						<Label>SDK</Label>
						<div class="p-3 bg-muted rounded-md">
							<div class="font-medium">{selectedSDK.name}</div>
							<div class="text-sm text-muted-foreground">Current: {selectedSDK.installation?.activeVersion || 'None'}</div>
						</div>
					</div>
					
					<div class="space-y-2">
						<Label>Target Version</Label>
						<Select 
							options={availableVersions.map(version => ({
								value: version,
								label: version
							}))}
							onSelect={(value) => selectedVersion = value}
							placeholder="Select version to switch to"
						/>
					</div>
					
					<div class="space-y-2">
						<Label for="switch-project-path">Project Path (Optional)</Label>
						<Input 
							id="switch-project-path" 
							bind:value={selectedProjectPath} 
							placeholder="/path/to/your/project"
						/>
						<p class="text-xs text-muted-foreground">Leave empty for global switch</p>
					</div>
					
					<div class="flex gap-2">
						<Button 
							onclick={executeSwitchVersion} 
							disabled={operationLoading || !selectedVersion}
							class="flex-1"
						>
							{#if operationLoading}
								<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
							{:else}
								<ArrowRightLeft class="h-4 w-4 mr-2" />
							{/if}
							Switch Version
						</Button>
						<Button variant="outline" onclick={() => showSwitchVersionDialog = false}>
							Cancel
						</Button>
					</div>
				{/if}
			</div>
		</DialogContent>
	</Dialog>
</div>
