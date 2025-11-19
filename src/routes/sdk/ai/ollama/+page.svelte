<!--
	Ollama Service Page
	Generic service page structure that can be reused for other AI services
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Button } from '@/lib/components/ui/button';
	import { logger } from '@/lib/domains/shared';
	import { toast } from '@/lib/domains/shared/stores/toastStore';
	import ModelList from '$lib/components/ModelList.svelte';
	import ModelTreeList from '$lib/components/ModelTreeList.svelte';
	import ProgressIndicator from '$lib/components/ProgressIndicator.svelte';

	const log = logger.createScoped('OllamaService');

	// Service information
	let serviceInfo = $state<any>(null);
	let serviceLoading = $state(true);
	let serviceError = $state<string | null>(null);

	// Version management
	let versions = $state<any[]>([]);
	let versionsLoading = $state(false);
	let versionsError = $state<string | null>(null);

	// Model management
	let models = $state<any[]>([]);
	let availableModels = $state<Record<string, any[]>>({});
	let modelsLoading = $state(false);
	let modelsError = $state<string | null>(null);
	let modelTab = $state('local'); // 'local' or 'library'
	
	// Installation progress tracking
	let installingModel = $state<string | null>(null);
	let installationProgress = $state(0);
	let installationStatus = $state<string>('');

	// Configuration management
	let configContent = $state<string>('');
	let configLoading = $state(false);
	let configError = $state<string | null>(null);
	let configPath = $state<string>('');

	// Log management
	let logsContent = $state<string>('');
	let logsLoading = $state(false);
	let logsError = $state<string | null>(null);
	let logPath = $state<string>('');

	// Tab management
	let activeTab = $state('service');
	
	// Get tab from URL parameter
	$effect(() => {
		const urlParams = new URLSearchParams($page.url.search);
		const tab = urlParams.get('tab');
		if (tab && ['service', 'version', 'models', 'configuration', 'log'].includes(tab)) {
			activeTab = tab;
		}
	});

	onMount(async () => {
		await loadServiceInfo();
	});

	async function loadServiceInfo() {
		try {
			serviceLoading = true;
			serviceError = null;
			
			// Get real service information from backend
			const info = await invoke('get_service_status', { sdkType: 'ollama' }) as any;
			serviceInfo = info;
			
			log.info('Ollama service info loaded', info);
			console.log('Service status details:', {
				running: info.running,
				status: info.status,
				port: info.port,
				pid: info.pid
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			serviceError = errorMessage;
			log.error('Failed to load Ollama service info', { error: errorMessage });
		} finally {
			serviceLoading = false;
		}
	}


	async function loadVersions() {
		try {
			versionsLoading = true;
			versionsError = null;
			
			// Get real Ollama versions from GitHub API
			const versionsData = await invoke('get_ollama_versions');
			versions = versionsData as any[];
			
			log.info('Ollama versions loaded', versionsData);
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			versionsError = errorMessage;
			log.error('Failed to load Ollama versions', error);
			
			// Show user-friendly error for GitHub API issues
			if (errorMessage.includes('403') || errorMessage.includes('Forbidden')) {
				toast.warning('GitHub API Rate Limited', 'Unable to fetch versions due to GitHub API rate limiting. Please try again later.');
			} else {
				toast.error('Failed to Load Versions', errorMessage);
			}
		} finally {
			versionsLoading = false;
		}
	}

	async function loadModels() {
		try {
			console.log('Loading local models...', { serviceRunning: serviceInfo?.running });
			modelsLoading = true;
			modelsError = null;
			
			// Check if Ollama service is running first
			if (!serviceInfo?.running) {
				modelsError = 'Ollama service is not running. Please start the service first.';
				toast.warning('Service Not Running', 'Please start the Ollama service before managing models.');
				return;
			}
			
			// Get real installed Ollama models
			console.log('Calling get_ollama_models...');
			const modelsData = await invoke('get_ollama_models');
			console.log('Received models data:', modelsData);
			models = modelsData as any[];
			
			log.info('Ollama models loaded', modelsData);
			console.log('Models array updated:', $state.snapshot(models));
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load models';
			modelsError = errorMessage;
			log.error('Failed to load Ollama models', error);
			console.error('Error loading models:', error);
			
			// Show user-friendly error
			if (errorMessage.includes('not installed') || errorMessage.includes('program not found')) {
				toast.error('Ollama Not Installed', 'Please install Ollama first to manage models.');
			} else {
				toast.error('Failed to Load Models', errorMessage);
			}
		} finally {
			modelsLoading = false;
			console.log('Models loading finished:', { modelsLoading, modelsError, modelsCount: models.length });
		}
	}

	async function loadAvailableModels() {
		try {
			console.log('Loading available models from library...');
			modelsLoading = true;
			modelsError = null;
			
			// Available models can be loaded without service running - they're from online library
			// Get available models from Ollama library
			console.log('Calling get_available_ollama_models...');
			const availableModelsData = await invoke('get_available_ollama_models');
			console.log('Received available models data:', availableModelsData);
			availableModels = availableModelsData as Record<string, any[]>;
			
			log.info('Available Ollama models loaded', availableModelsData);
			console.log('Available models array updated:', $state.snapshot(availableModels));
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load available models';
			modelsError = errorMessage;
			log.error('Failed to load available Ollama models', error);
			console.error('Error loading available models:', error);
			
			// Show user-friendly error
			toast.error('Failed to Load Available Models', errorMessage);
		} finally {
			modelsLoading = false;
			console.log('Available models loading finished:', { modelsLoading, modelsError, availableModelsCount: Object.keys(availableModels).length });
		}
	}

	async function installModel(modelName: string) {
		try {
			// Check if Ollama service is running first
			if (!serviceInfo?.running) {
				toast.warning('Service Not Running', 'Please start the Ollama service before installing models.');
				return;
			}
			
			// Set up progress tracking
			installingModel = modelName;
			installationProgress = 0;
			installationStatus = 'Starting download...';
			
			log.info('Installing model:', modelName);
			console.log('Starting model installation:', modelName);
			
			// Show initial progress
			toast.info('Download Started', `Starting download of ${modelName}...`);
			
			// Set initial progress state
			installationProgress = 10;
			installationStatus = 'Starting download...';
			
			// Start the actual installation (this is a long-running operation)
			installationStatus = 'Downloading model from Ollama registry...';
			installationProgress = 50;
			
			// Show honest status - this is a long-running operation
			installationStatus = `Downloading ${modelName}... This may take several minutes for large models`;
			
			// Start the installation with a timeout
			const installPromise = invoke('install_ollama_model', { modelName });
			const timeoutPromise = new Promise((_, reject) => 
				setTimeout(() => reject(new Error('Installation timeout - this may take several minutes for large models')), 300000) // 5 minutes
			);
			
			const result = await Promise.race([installPromise, timeoutPromise]);
			
			// Installation completed
			installationProgress = 100;
			installationStatus = 'Installation complete!';
			
			await loadModels(); // Refresh models list
			log.info('Model installed successfully', { modelName, result });
			toast.success('Model Installed', `${modelName} has been installed successfully`);
			
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log.error('Failed to install model', { modelName, error: errorMessage });
			toast.error('Installation Failed', `Failed to install ${modelName}: ${errorMessage}`);
		} finally {
			// Reset progress tracking
			setTimeout(() => {
				installingModel = null;
				installationProgress = 0;
				installationStatus = '';
			}, 2000); // Keep progress visible for 2 seconds after completion
		}
	}

	function cancelInstallation() {
		if (installingModel) {
			const modelName = installingModel;
			log.info('Cancelling installation:', modelName);
			console.log('Cancelling installation:', modelName);
			
			// Reset installation state
			installingModel = null;
			installationProgress = 0;
			installationStatus = '';
			
			toast.info('Installation Cancelled', `Installation of ${modelName} has been cancelled. Note: The download may continue in the background.`);
		}
	}

	async function removeModel(modelName: string) {
		try {
			// Check if Ollama service is running first
			if (!serviceInfo?.running) {
				toast.warning('Service Not Running', 'Please start the Ollama service before removing models.');
				return;
			}
			
			log.info('Removing model:', modelName);
			const result = await invoke('remove_ollama_model', { modelName });
			await loadModels(); // Refresh models list
			log.info('Model removed successfully', { modelName, result });
			toast.success('Model Removed', `${modelName} has been removed successfully`);
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log.error('Failed to remove model', { modelName, error: errorMessage });
			toast.error('Removal Failed', `Failed to remove ${modelName}: ${errorMessage}`);
		}
	}

	async function installOllamaVersion(version: string) {
		try {
			log.info('Installing Ollama version:', version);
			// For now, show a message that manual installation is needed
			// In a real implementation, this would download and install Ollama
			toast.info(
				'Manual Installation Required',
				`To install Ollama version ${version}, please download it from GitHub.`,
				{
					action: {
						label: 'Download Version',
						onClick: () => window.open(`https://github.com/ollama/ollama/releases/tag/v${version}`, '_blank')
					}
				}
			);
			await loadVersions(); // Refresh versions list
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log.error('Failed to install Ollama version', { version, error: errorMessage });
		}
	}

	async function startService() {
		try {
			serviceError = null; // Clear any previous errors
			const result = await invoke('start_service', { sdkType: 'ollama' });
			
			// Wait a moment for the service to start
			await new Promise(resolve => setTimeout(resolve, 500));
			
			// Refresh service info to update UI
			await loadServiceInfo();
			
			log.info('Ollama service started successfully', result);
			toast.success('Service Started', 'Ollama service is now running');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			
			// Wait a moment and check if service actually started despite the error
			await new Promise(resolve => setTimeout(resolve, 500));
			await loadServiceInfo();
			
			// If service is actually running now, treat it as success
			if (serviceInfo?.running) {
				log.info('Ollama service started (verified after error)', errorMessage);
				serviceError = null;
				toast.success('Service Started', 'Ollama service is now running');
				return;
			}
			
			// Service didn't start, show error
			serviceError = errorMessage;
			log.error('Failed to start Ollama service', { error: errorMessage });
			
			// Show user-friendly message if Ollama is not installed
			if (errorMessage.includes('not installed') || errorMessage.includes('program not found')) {
				toast.error(
					'Ollama Not Installed',
					'Please install Ollama from https://ollama.com/download first.',
					{
						action: {
							label: 'Download Ollama',
							onClick: () => window.open('https://ollama.com/download', '_blank')
						}
					}
				);
			} else {
				toast.error('Failed to Start Service', errorMessage);
			}
		}
	}

	async function stopService() {
		try {
			serviceError = null; // Clear any previous errors
			const result = await invoke('stop_service', { sdkType: 'ollama' });
			
			// Wait a moment for the service to fully stop
			await new Promise(resolve => setTimeout(resolve, 500));
			
			// Refresh service info to update UI
			await loadServiceInfo();
			
			log.info('Ollama service stopped successfully', result);
			toast.success('Service Stopped', 'Ollama service has been stopped');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			
			// Wait a moment and check if service actually stopped despite the error
			await new Promise(resolve => setTimeout(resolve, 500));
			await loadServiceInfo();
			
			// If service is actually stopped now, treat it as success
			if (!serviceInfo?.running) {
				log.info('Ollama service stopped (verified after error)', errorMessage);
				serviceError = null;
				toast.success('Service Stopped', 'Ollama service has been stopped');
				return;
			}
			
			// Service is still running, show error
			serviceError = errorMessage;
			log.error('Failed to stop Ollama service', { error: errorMessage });
			
			// Show user-friendly message if Ollama is not installed
			if (errorMessage.includes('not installed') || errorMessage.includes('program not found')) {
				toast.error(
					'Ollama Not Installed',
					'Please install Ollama from https://ollama.com/download first.',
					{
						action: {
							label: 'Download Ollama',
							onClick: () => window.open('https://ollama.com/download', '_blank')
						}
					}
				);
			} else {
				toast.error('Failed to Stop Service', errorMessage);
			}
		}
	}

	async function restartService() {
		try {
			// Stop first, then start
			await invoke('stop_service', { sdkType: 'ollama' });
			await new Promise(resolve => setTimeout(resolve, 2000)); // Wait 2 seconds for graceful shutdown
			await invoke('start_service', { sdkType: 'ollama' });
			await loadServiceInfo(); // Refresh service info
			log.info('Ollama service restarted successfully');
			toast.success('Service Restarted', 'Ollama service has been restarted');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			serviceError = errorMessage;
			log.error('Failed to restart Ollama service', { error: errorMessage });
			
			// Show user-friendly message if Ollama is not installed
			if (errorMessage.includes('not installed') || errorMessage.includes('program not found')) {
				toast.error(
					'Ollama Not Installed',
					'Please install Ollama from https://ollama.com/download first.',
					{
						action: {
							label: 'Download Ollama',
							onClick: () => window.open('https://ollama.com/download', '_blank')
						}
					}
				);
			}
		}
	}

	async function viewLogs() {
		// Switch to log tab to view logs
		setActiveTab('log');
	}

	async function checkUpdates() {
		try {
			log.info('Checking for Ollama updates...');
			const result = await invoke('check_ollama_updates');
			log.info('Update check result:', result);
			// Show update status to user
			toast.success('Update Check Complete', result ? String(result) : 'No updates available');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log.error('Failed to check for updates', { error: errorMessage });
			toast.error('Update Check Failed', errorMessage);
		}
	}

	async function updateOllama() {
		try {
			log.info('Updating Ollama...');
			const result = await invoke('update_ollama');
			log.info('Update result:', result);
			toast.success('Ollama Updated', String(result));
			// Refresh service info after update
			await loadServiceInfo();
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : String(error);
			log.error('Failed to update Ollama', { error: errorMessage });
			toast.error('Update Failed', errorMessage);
		}
	}

	async function loadConfiguration() {
		try {
			configLoading = true;
			configError = null;
			
			// Note: Config file management will be implemented in backend
			// For now, show placeholder with default config
			configPath = 'ollama/ollama.conf';
			configContent = 'OLLAMA_HOST=0.0.0.0:11434\nOLLAMA_KEEP_ALIVE=5m\nOLLAMA_DEBUG=false\n\n# Add custom environment variables here\n# OLLAMA_MAX_LOADED_MODELS=3\n# OLLAMA_NUM_PARALLEL=2';
			
			log.info('Configuration loaded');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load configuration';
			configError = errorMessage;
			log.error('Failed to load configuration', error);
			toast.error('Failed to Load Configuration', errorMessage);
		} finally {
			configLoading = false;
		}
	}

	async function saveConfiguration() {
		try {
			configLoading = true;
			configError = null;
			
			// Note: Config file writing will be implemented in backend
			// For now, show a message that this is not yet implemented
			log.info('Configuration save requested');
			toast.info('Configuration Save', 'Config file management is being implemented. Changes are not yet persisted.');
			
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to save configuration';
			configError = errorMessage;
			log.error('Failed to save configuration', error);
			toast.error('Failed to Save Configuration', errorMessage);
		} finally {
			configLoading = false;
		}
	}

	async function resetConfiguration() {
		try {
			configContent = 'OLLAMA_HOST=0.0.0.0:11434\nOLLAMA_KEEP_ALIVE=5m\nOLLAMA_DEBUG=false';
			toast.success('Configuration Reset', 'Configuration has been reset to default values');
		} catch (error) {
			log.error('Failed to reset configuration', error);
		}
	}

	async function loadLogs() {
		try {
			logsLoading = true;
			logsError = null;
			
			// Note: Log file reading will be implemented in backend
			// For now, show a placeholder message
			logPath = serviceInfo?.logPath || 'ollama/ollama.log';
			
			if (serviceInfo?.running) {
				logsContent = `[${new Date().toISOString()}] Ollama service is running.\n[Note] Real-time log viewing will be implemented soon.\n\nTo view Ollama logs, check the console output or log files in the Ollama installation directory.`;
			} else {
				logsContent = 'No logs available. Start the Ollama service to see logs.';
			}
			
			log.info('Logs loaded');
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load logs';
			logsError = errorMessage;
			log.error('Failed to load logs', error);
			toast.error('Failed to Load Logs', errorMessage);
		} finally {
			logsLoading = false;
		}
	}

	async function clearLogs() {
		try {
			logsContent = '';
			toast.success('Logs Cleared', 'Log display has been cleared');
		} catch (error) {
			log.error('Failed to clear logs', error);
		}
	}

	async function downloadLogs() {
		try {
			if (!logsContent) {
				toast.warning('No Logs', 'No logs available to download');
				return;
			}
			
			// Create a download link
			const blob = new Blob([logsContent], { type: 'text/plain' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `ollama-logs-${new Date().toISOString().split('T')[0]}.log`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
			
			toast.success('Logs Downloaded', 'Logs have been downloaded successfully');
		} catch (error) {
			log.error('Failed to download logs', error);
			toast.error('Download Failed', 'Failed to download logs');
		}
	}

	function setActiveTab(tab: string) {
		activeTab = tab;
		// Update URL using SvelteKit navigation
		goto(`?tab=${tab}`, { replaceState: true });
		
		// Load data when switching to specific tabs
		if (tab === 'version') {
			if (versions.length === 0) {
				loadVersions();
			}
		} else if (tab === 'models') {
			// Always refresh when switching to models tab to ensure current data
			if (modelTab === 'local') {
				if (serviceInfo?.running) {
					loadModels();
				}
			} else {
				// Library tab doesn't need service running
				loadAvailableModels();
			}
		} else if (tab === 'configuration') {
			if (!configContent) {
				loadConfiguration();
			}
		} else if (tab === 'log') {
			loadLogs();
		}
	}
</script>

<!-- Service Header -->
<div class="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
	<div class="flex h-14 items-center px-4 w-full">
			<div class="mr-4 hidden md:flex">
				<a class="mr-6 flex items-center space-x-2" href="/sdk">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
					</svg>
					<span class="hidden font-bold lg:inline-block">Back to SDK</span>
				</a>
			</div>
			<div class="flex flex-1 items-center justify-between space-x-2 md:justify-end">
				<div class="w-full flex-1 md:w-auto md:flex-none">
					<div class="flex items-center gap-4">
						<div class="flex items-center gap-3">
							<div class="w-8 h-8 rounded-lg bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center text-white font-bold">
								<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
									<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.94-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
								</svg>
							</div>
							<div>
								<h1 class="text-xl font-semibold">Ollama</h1>
								<p class="text-sm text-muted-foreground">Local AI model runner</p>
							</div>
						</div>
						<div class="flex items-center gap-2">
							{#if serviceInfo?.running}
								<Badge variant="default" class="bg-green-100 text-green-800">
									<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
										<path d="M5 13l4 4L19 7"/>
									</svg>
									Running
								</Badge>
							{:else}
								<Badge variant="outline" class="text-gray-500">
									{serviceInfo?.status || 'Stopped'}
								</Badge>
							{/if}
							{#if serviceInfo?.port}
								<Badge variant="secondary">Port {serviceInfo.port}</Badge>
							{/if}
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Main Content -->
	<div class="flex-1 p-6">
		<Tabs value={activeTab} onValueChange={setActiveTab} class="w-full">
			<TabsList class="grid w-full grid-cols-5">
				<TabsTrigger value="service">Service</TabsTrigger>
				<TabsTrigger value="version">Version</TabsTrigger>
				<TabsTrigger value="models">Models</TabsTrigger>
				<TabsTrigger value="configuration">Configuration</TabsTrigger>
				<TabsTrigger value="log">Log</TabsTrigger>
			</TabsList>

			<!-- Service Tab -->
			<TabsContent value="service" class="mt-6">
				<Card>
					<CardHeader>
						<CardTitle>Ollama Service</CardTitle>
						<CardDescription>Manage your Ollama service instance</CardDescription>
					</CardHeader>
					<CardContent>
						{#if serviceLoading}
							<div class="flex items-center justify-center p-8">
								<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
								<span class="ml-2">Loading service information...</span>
							</div>
						{:else if serviceError}
							<div class="text-center p-8">
								<p class="text-destructive">Error: {serviceError}</p>
								<Button onclick={loadServiceInfo} class="mt-4">Retry</Button>
							</div>
						{:else}
							<div class="space-y-4">
								<div>
									<h4 class="font-medium mb-2">Service Status</h4>
									<p class="text-sm text-muted-foreground">
										{#if serviceInfo?.running}
											Service is running on port {serviceInfo.port || '11434'}
										{:else}
											Service is {serviceInfo?.status || 'not running'}
										{/if}
									</p>
								</div>
								
								<div class="flex gap-2">
									{#if serviceInfo?.running}
										<Button variant="destructive" onclick={stopService}>Stop Service</Button>
									{:else}
										<Button onclick={startService}>Start Service</Button>
									{/if}
									<Button variant="outline" onclick={restartService}>Restart Service</Button>
									<Button variant="outline" onclick={viewLogs}>View Logs</Button>
									<Button variant="outline" onclick={checkUpdates}>Check Updates</Button>
									<Button variant="outline" onclick={updateOllama}>Update Ollama</Button>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>

			<!-- Version Tab -->
			<TabsContent value="version" class="mt-6">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<div>
								<CardTitle>Ollama Versions</CardTitle>
								<CardDescription>Install and manage Ollama versions</CardDescription>
							</div>
							<Button variant="outline" onclick={loadVersions} disabled={versionsLoading}>
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
								</svg>
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if versionsLoading}
							<div class="flex items-center justify-center p-8">
								<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
								<span class="ml-2">Loading versions...</span>
							</div>
						{:else if versionsError}
							<div class="text-center p-8">
								<p class="text-destructive">Error: {versionsError}</p>
								<Button onclick={loadVersions} class="mt-4">Retry</Button>
							</div>
						{:else if versions.length > 0}
							<div class="space-y-4">
								<div class="grid gap-4">
									{#each versions as version}
										<div class="flex items-center justify-between p-4 border rounded-lg">
											<div class="flex items-center gap-4">
												<div class="flex items-center gap-2">
													<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
													</svg>
													<span class="font-medium">Ollama-{version.version}</span>
												</div>
												<div class="text-sm text-muted-foreground">
													Version {version.version}
												</div>
												{#if version.size}
													<div class="text-sm text-muted-foreground">
														{version.size}
													</div>
												{/if}
											</div>
											<div class="flex items-center gap-2">
												{#if version.installed}
													<Badge variant="default" class="bg-green-100 text-green-800">
														<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
															<path d="M5 13l4 4L19 7"/>
														</svg>
														Installed
													</Badge>
													{#if version.active}
														<Badge variant="secondary" class="bg-blue-100 text-blue-800">
															Active
														</Badge>
													{/if}
												{:else}
													<Button size="sm" onclick={() => installOllamaVersion(version.version)}>
														<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
														</svg>
														Install
													</Button>
												{/if}
											</div>
										</div>
									{/each}
								</div>
							</div>
						{:else}
							<div class="text-center p-8">
								<p class="text-muted-foreground">No versions available.</p>
								<Button onclick={loadVersions} class="mt-4">Load Versions</Button>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>

			<!-- Models Tab -->
			<TabsContent value="models" class="mt-6">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-4">
								<div>
									<CardTitle>Models</CardTitle>
									<CardDescription>Manage your AI models</CardDescription>
								</div>
							<div class="flex items-center gap-2">
								<Button 
									variant={modelTab === 'local' ? 'default' : 'outline'} 
									size="sm"
									onclick={() => {
										modelTab = 'local';
										console.log('Switched to local tab');
										loadModels();
									}}
								>
									Local
								</Button>
								<Button 
									variant={modelTab === 'library' ? 'default' : 'outline'} 
									size="sm"
									onclick={() => {
										console.log('Library button clicked, current modelTab:', modelTab);
										modelTab = 'library';
										console.log('Switched to library tab, new modelTab:', modelTab);
										loadAvailableModels();
									}}
								>
									Library
								</Button>
							</div>
							</div>
							<Button variant="outline" onclick={() => modelTab === 'local' ? loadModels() : loadAvailableModels()} disabled={modelsLoading}>
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
								</svg>
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if modelTab === 'local'}
							{#if !serviceInfo?.running}
								<div class="text-center p-8">
									<div class="flex flex-col items-center gap-4">
										<svg class="w-12 h-12 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"/>
										</svg>
										<div>
											<h3 class="text-lg font-semibold">Ollama Service Not Running</h3>
											<p class="text-muted-foreground">Please start the Ollama service to view installed models.</p>
										</div>
										<Button onclick={startService} class="mt-2">
											<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1m-6-8h8a2 2 0 012 2v8a2 2 0 01-2 2H8a2 2 0 01-2-2V8a2 2 0 012-2z"/>
											</svg>
											Start Service
										</Button>
									</div>
								</div>
							{:else}
								<ModelList 
									models={models}
									isInstalled={true}
									
									error={modelsError}
									installingModel={installingModel}
									installationProgress={installationProgress}
									installationStatus={installationStatus}
									onInstall={installModel}
									onRemove={removeModel}
									onRetry={loadModels}
									onBrowseAvailable={() => {
										modelTab = 'library';
										loadAvailableModels();
									}}
								/>
							{/if}
						{:else if modelTab === 'library'}
							<ModelTreeList 
								models={availableModels}
								
								error={modelsError}
								installingModel={installingModel}
								installationProgress={installationProgress}
								installationStatus={installationStatus}
								onInstall={installModel}
								onRetry={loadAvailableModels}
								onCancel={cancelInstallation}
							/>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>

			<!-- Configuration Tab -->
			<TabsContent value="configuration" class="mt-6">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<div>
								<CardTitle>Configuration</CardTitle>
								<CardDescription>Configure Ollama settings</CardDescription>
							</div>
							<Button variant="outline" onclick={loadConfiguration} disabled={configLoading}>
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
								</svg>
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if configLoading}
							<div class="flex items-center justify-center p-8">
								<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
								<span class="ml-2">Loading configuration...</span>
							</div>
						{:else if configError}
							<div class="text-center p-8">
								<p class="text-destructive">Error: {configError}</p>
								<Button onclick={loadConfiguration} class="mt-4">Retry</Button>
							</div>
						{:else}
							<div class="space-y-4">
								{#if configPath}
									<div class="text-sm text-muted-foreground">
										Config file: <code class="px-2 py-1 bg-muted rounded">{configPath}</code>
									</div>
								{/if}
								<div>
									<label for="config-content" class="text-sm font-medium mb-2 block">Configuration File Content</label>
									<textarea
										id="config-content"
										class="w-full h-64 p-3 border rounded-md font-mono text-sm"
										bind:value={configContent}
										placeholder="OLLAMA_HOST=0.0.0.0:11434&#10;OLLAMA_KEEP_ALIVE=5m&#10;OLLAMA_DEBUG=false"
									></textarea>
								</div>
								<div class="flex gap-2">
									<Button onclick={saveConfiguration}>Save Configuration</Button>
									<Button variant="outline" onclick={resetConfiguration}>Reset to Default</Button>
								</div>
								<div class="text-sm text-muted-foreground">
									<p class="font-medium mb-2">Available Settings:</p>
									<ul class="list-disc list-inside space-y-1 text-xs">
										<li><code>OLLAMA_HOST</code> - Host and port (default: 0.0.0.0:11434)</li>
										<li><code>OLLAMA_KEEP_ALIVE</code> - Keep models in memory duration (default: 5m)</li>
										<li><code>OLLAMA_DEBUG</code> - Enable debug logging</li>
										<li><code>OLLAMA_MAX_LOADED_MODELS</code> - Maximum number of loaded models</li>
										<li><code>OLLAMA_NUM_PARALLEL</code> - Number of parallel requests</li>
									</ul>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>

			<!-- Log Tab -->
			<TabsContent value="log" class="mt-6">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<div>
								<CardTitle>Service Logs</CardTitle>
								<CardDescription>View Ollama service logs</CardDescription>
							</div>
							<Button variant="outline" onclick={loadLogs} disabled={logsLoading}>
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
								</svg>
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if logsLoading}
							<div class="flex items-center justify-center p-8">
								<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
								<span class="ml-2">Loading logs...</span>
							</div>
						{:else if logsError}
							<div class="text-center p-8">
								<p class="text-destructive">Error: {logsError}</p>
								<Button onclick={loadLogs} class="mt-4">Retry</Button>
							</div>
						{:else}
							<div class="space-y-4">
								{#if logPath}
									<div class="text-sm text-muted-foreground">
										Log file: <code class="px-2 py-1 bg-muted rounded">{logPath}</code>
									</div>
								{/if}
								<div class="border rounded-md p-4 bg-muted/50">
									<div class="flex items-center justify-between mb-2">
										<span class="text-sm font-medium">Log Output</span>
										<div class="flex gap-2">
											<Button size="sm" variant="outline" onclick={clearLogs}>Clear</Button>
											<Button size="sm" variant="outline" onclick={downloadLogs}>Download</Button>
										</div>
									</div>
									<div class="font-mono text-xs overflow-auto max-h-96 bg-background p-3 rounded border">
										{#if logsContent}
											<pre class="whitespace-pre-wrap">{logsContent}</pre>
										{:else}
											<p class="text-muted-foreground">No logs available. Start the service to see logs.</p>
										{/if}
									</div>
								</div>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
		</Tabs>
	</div>
