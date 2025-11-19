<!--
	AI Provider Settings - Configure Ollama AI Provider
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Switch } from '$lib/components/ui/switch';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
	import { Separator } from '$lib/components/ui/separator';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { aiProviderService } from '../../services/aiProviderService.js';
	import type { ProviderConfig, ProviderType, ConfigurationStatus } from '../../types/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { 
		Brain, 
		CheckCircle2, 
		XCircle, 
		RefreshCw, 
		AlertTriangle,
		Server,
		Sparkles,
		Loader2,
		Circle,
		Download,
		Search,
		ChevronDown,
		ChevronUp,
		Package
	} from '@lucide/svelte';
	import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '$lib/components/ui/collapsible';
	import Icon from '@iconify/svelte';

	const providerType: ProviderType = 'Ollama';
	
	let providerConfig = $state<ProviderConfig | null>(null);
	let defaultProvider = $state<ProviderType | null>(null);
	let configStatus = $state<ConfigurationStatus | null>(null);
	let isLoading = $state(false);
	let testingProvider = $state(false);
	let availableModels = $state<string[]>([]);
	let availableOllamaModels = $state<Record<string, Array<{ name: string; size?: string }>>>({});
	let isLoadingAvailableModels = $state(false);
	let modelSearchQuery = $state('');
	let showAvailableModels = $state(false);
	let installingModel = $state<string | null>(null);
	let serviceStatus = $state<{ running: boolean; status: string; port?: number; pid?: number } | null>(null);
	let isCheckingStatus = $state(false);
	let statusCheckInterval: ReturnType<typeof setInterval> | null = $state(null);

	onMount(() => {
		(async () => {
			await loadProvider();
			await checkServiceStatus();
			
			// Poll service status every 5 seconds
			statusCheckInterval = setInterval(async () => {
				await checkServiceStatus();
			}, 5000);
		})();
		
		return () => {
			if (statusCheckInterval) {
				clearInterval(statusCheckInterval);
			}
		};
	});

	async function loadProvider() {
		isLoading = true;
		try {
			const [config, defaultProviderType, status] = await Promise.all([
				aiProviderService.getProviderConfig(providerType).catch(() => null),
				aiProviderService.getDefaultProvider(),
				aiProviderService.getConfigStatus(providerType).catch(() => null),
			]);

			providerConfig = config;
			defaultProvider = defaultProviderType;
			configStatus = status;
		} catch (error) {
			console.error('Failed to load Ollama configuration:', error);
			toastActions.error('Failed to load Ollama configuration', error);
		} finally {
			isLoading = false;
		}
	}

	async function updateConfig(updates: Partial<ProviderConfig>) {
		if (!providerConfig) return;

		const updated: ProviderConfig = {
			...providerConfig,
			...updates,
		};

		try {
			await aiProviderService.saveProviderConfig(updated);
			providerConfig = updated;
			
			// Reload status
			try {
				const status = await aiProviderService.getConfigStatus(providerType);
				configStatus = status;
				
				// Auto-set as default if:
				// 1. Provider is enabled and configured
				// 2. No default provider is currently set
				// 3. This provider is now properly configured
				if (updated.enabled && status.is_configured && !defaultProvider) {
					try {
						await aiProviderService.setDefaultProvider(providerType);
						defaultProvider = providerType;
						toastActions.info('Ollama set as default provider', 'Since no default was set, Ollama has been automatically set as the default AI provider.');
					} catch (error) {
						console.error('Failed to auto-set default provider:', error);
						// Don't show error, just log it
					}
				}
			} catch (error) {
				console.error('Failed to reload status:', error);
			}
			
			toastActions.success('Ollama configuration saved');
		} catch (error) {
			console.error('Failed to save Ollama config:', error);
			toastActions.error('Failed to save Ollama configuration', error);
		}
	}

	async function setAsDefault() {
		try {
			await aiProviderService.setDefaultProvider(providerType);
			defaultProvider = providerType;
			toastActions.success('Ollama set as default provider');
		} catch (error) {
			console.error('Failed to set default provider:', error);
			toastActions.error('Failed to set default provider', error);
		}
	}

	async function testConnection() {
		if (!providerConfig) {
			toastActions.error('Configuration not loaded', 'Please wait for configuration to load');
			return;
		}

		// Check if service is running first
		if (serviceStatus && !serviceStatus.running) {
			toastActions.error('Ollama service not running', 'Please start the Ollama service before testing the connection.');
			return;
		}

		// Ensure provider is registered before testing
		// Save the config first to register it in the AI service
		// Make sure enabled is true so it gets registered
		const configToSave = {
			...providerConfig,
			enabled: true, // Ensure enabled so provider gets registered
		};

		try {
			await aiProviderService.saveProviderConfig(configToSave);
			// Update local state
			providerConfig = configToSave;
			
			// Auto-set as default if no default is set
			if (!defaultProvider) {
				try {
					await aiProviderService.setDefaultProvider(providerType);
					defaultProvider = providerType;
				} catch (error) {
					console.error('Failed to set default provider:', error);
				}
			}
			
			// Small delay to ensure backend has processed the registration
			await new Promise(resolve => setTimeout(resolve, 100));
		} catch (error) {
			console.error('Failed to save config before testing:', error);
			toastActions.error('Failed to register provider', 'Could not save configuration. Please try again.');
			return;
		}

		testingProvider = true;
		try {
			await aiProviderService.testProvider(providerType);
			toastActions.success('Ollama connection test successful');
			
			// Reload status after successful test
			try {
				const status = await aiProviderService.getConfigStatus(providerType);
				configStatus = status;
				await checkServiceStatus();
			} catch (error) {
				console.error('Failed to reload status:', error);
			}
		} catch (error) {
			console.error('Failed to test Ollama:', error);
			toastActions.error('Ollama connection test failed', error);
		} finally {
			testingProvider = false;
		}
	}

	async function checkServiceStatus() {
		if (isCheckingStatus) return;
		
		isCheckingStatus = true;
		try {
			const status = await invoke<{ running: boolean; status: string; port?: number; pid?: number }>('get_service_status', {
				sdkType: 'ollama'
			});
			serviceStatus = status;
		} catch (error) {
			console.error('Failed to check Ollama service status:', error);
			serviceStatus = { running: false, status: 'unknown' };
		} finally {
			isCheckingStatus = false;
		}
	}

	async function loadModels() {
		if (availableModels.length > 0) {
			return; // Already loaded
		}

		if (!providerConfig) {
			toastActions.error('Configuration not loaded', 'Please wait for configuration to load');
			return;
		}

		// Ensure provider is registered before loading models
		// Save the config first to register it in the AI service
		// Make sure enabled is true so it gets registered
		const configToSave = {
			...providerConfig,
			enabled: true, // Ensure enabled so provider gets registered
		};

		try {
			await aiProviderService.saveProviderConfig(configToSave);
			// Update local state
			providerConfig = configToSave;
			// Small delay to ensure backend has processed the registration
			await new Promise(resolve => setTimeout(resolve, 100));
		} catch (error) {
			console.error('Failed to save config before loading models:', error);
			toastActions.error('Failed to register provider', 'Could not save configuration. Please try again.');
			return;
		}

		try {
			const models = await aiProviderService.getAvailableModels(providerType);
			availableModels = models;
			if (models.length === 0) {
				toastActions.info('No models found', 'Make sure Ollama is running and has models installed');
			} else {
				toastActions.success(`Loaded ${models.length} model(s)`);
			}
		} catch (error) {
			console.error('Failed to load models:', error);
			const errorMessage = error instanceof Error ? error.message : String(error);
			if (errorMessage.includes('not found') || errorMessage.includes('not available')) {
				toastActions.error('Ollama provider not registered', 'The provider may not be properly registered. Please try saving your configuration again.');
			} else {
				toastActions.error('Failed to load models', error);
			}
		}
	}

	async function loadAvailableOllamaModels() {
		if (Object.keys(availableOllamaModels).length > 0) {
			return; // Already loaded
		}

		isLoadingAvailableModels = true;
		try {
			const models = await aiProviderService.getAvailableOllamaModels();
			availableOllamaModels = models;
			const totalModels = Object.values(models).reduce((sum, family) => sum + family.length, 0);
			if (totalModels === 0) {
				toastActions.info('No models available', 'Unable to fetch available models. This may be a network issue.');
			} else {
				toastActions.success(`Loaded ${totalModels} available model(s) from ${Object.keys(models).length} families`);
			}
		} catch (error) {
			console.error('Failed to load available Ollama models:', error);
			toastActions.error('Failed to load available models', error);
		} finally {
			isLoadingAvailableModels = false;
		}
	}

	async function installModel(modelName: string) {
		if (installingModel) {
			return; // Already installing
		}

		if (!serviceStatus?.running) {
			toastActions.error('Service not running', 'Please start the Ollama service before installing models.');
			return;
		}

		installingModel = modelName;
		try {
			toastActions.info('Installing model', `Starting download of ${modelName}... This may take several minutes.`);
			
			await invoke('install_ollama_model', { modelName });
			
			// Wait a moment for installation to complete
			await new Promise(resolve => setTimeout(resolve, 2000));
			
			// Refresh installed models list
			availableModels = [];
			await loadModels();
			
			toastActions.success('Model installed', `${modelName} has been installed successfully`);
		} catch (error) {
			console.error('Failed to install model:', error);
			const errorMessage = error instanceof Error ? error.message : String(error);
			toastActions.error('Installation failed', errorMessage);
		} finally {
			installingModel = null;
		}
	}

	function isModelInstalled(modelName: string): boolean {
		return availableModels.includes(modelName);
	}

	function getFilteredModels(): Record<string, Array<{ name: string; size?: string }>> {
		if (!modelSearchQuery.trim()) {
			return availableOllamaModels;
		}

		const query = modelSearchQuery.toLowerCase();
		const filtered: Record<string, Array<{ name: string; size?: string }>> = {};

		for (const [family, models] of Object.entries(availableOllamaModels)) {
			const familyModels = models.filter(model => 
				model.name.toLowerCase().includes(query) || 
				family.toLowerCase().includes(query)
			);
			if (familyModels.length > 0) {
				filtered[family] = familyModels;
			}
		}

		return filtered;
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div>
		<h2 class="text-2xl font-bold flex items-center gap-2">
			<Brain class="w-6 h-6" />
			AI Provider Configuration
		</h2>
		<p class="text-muted-foreground mt-1">
			Configure Ollama for AI-powered features like task generation. Ollama runs locally on your machine and requires no API key.
		</p>
	</div>

	<!-- Default Provider Status -->
	{#if defaultProvider === providerType}
		<Alert>
			<CheckCircle2 class="h-4 w-4" />
			<AlertTitle>Ollama is Default Provider</AlertTitle>
			<AlertDescription>
				Ollama is currently set as the default AI provider and will be used for all AI features.
			</AlertDescription>
		</Alert>
	{:else if configStatus?.is_configured}
		<Alert>
			<AlertTriangle class="h-4 w-4" />
			<AlertTitle>Ollama Not Set as Default</AlertTitle>
			<AlertDescription>
				Ollama is configured but not set as the default provider. Click "Set as Default" below to use it.
			</AlertDescription>
		</Alert>
	{/if}

	<!-- Ollama Configuration -->
	{#if isLoading || !providerConfig}
		<Card>
			<CardContent class="py-8">
				<div class="text-center text-muted-foreground">
					Loading Ollama configuration...
				</div>
			</CardContent>
		</Card>
	{:else}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<Icon icon="lucide:server" class="w-5 h-5" />
					Ollama Configuration
					{#if serviceStatus}
						<Badge 
							variant={serviceStatus.running ? 'default' : 'secondary'}
							class={serviceStatus.running ? 'bg-green-500 hover:bg-green-600' : 'ml-2'}
						>
							{#if isCheckingStatus}
								<Loader2 class="w-3 h-3 mr-1 animate-spin" />
							{:else if serviceStatus.running}
								<Circle class="w-3 h-3 mr-1 fill-current" />
							{:else}
								<Circle class="w-3 h-3 mr-1" />
							{/if}
							{serviceStatus.running ? 'Running' : 'Stopped'}
						</Badge>
						<Button
							variant="ghost"
							size="sm"
							onclick={checkServiceStatus}
							disabled={isCheckingStatus}
							class="h-6 w-6 p-0 ml-1"
							title="Refresh status"
						>
							<RefreshCw class={`w-3 h-3 ${isCheckingStatus ? 'animate-spin' : ''}`} />
						</Button>
					{/if}
				</CardTitle>
				<CardDescription>
					Local AI models running on your machine. No API key required.
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-6">
				<!-- Service Status Warning (only show when not running) -->
				{#if serviceStatus && !serviceStatus.running}
					<Alert variant="destructive">
						<AlertTriangle class="h-4 w-4" />
						<AlertTitle>Ollama Service Not Running</AlertTitle>
						<AlertDescription>
							The Ollama service is not running. You need to start it before you can use Ollama for AI features.
						</AlertDescription>
					</Alert>
				{/if}

				<!-- Configuration Status -->
				{#if configStatus}
					{#if !configStatus.is_configured}
						<Alert variant="destructive">
							<AlertTriangle class="h-4 w-4" />
							<AlertTitle>Configuration Incomplete</AlertTitle>
							<AlertDescription>
								{#if configStatus.missing_fields.length > 0}
									<p class="font-medium mb-1">Missing fields:</p>
									<ul class="list-disc list-inside space-y-1">
										{#each configStatus.missing_fields as field}
											<li>{field}</li>
										{/each}
									</ul>
								{/if}
								{#if configStatus.warnings.length > 0}
									<p class="font-medium mt-2 mb-1">Warnings:</p>
									<ul class="list-disc list-inside space-y-1">
										{#each configStatus.warnings as warning}
											<li>{warning}</li>
										{/each}
									</ul>
								{/if}
							</AlertDescription>
						</Alert>
					{:else}
						<Alert>
							<CheckCircle2 class="h-4 w-4" />
							<AlertTitle>Configured</AlertTitle>
							<AlertDescription>
								Ollama is properly configured and ready to use.
							</AlertDescription>
						</Alert>
					{/if}
				{/if}

				<Separator />

				<!-- Enable/Disable -->
				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<Label>Enable Ollama</Label>
						<p class="text-sm text-muted-foreground">
							Enable Ollama to use it for AI features
						</p>
					</div>
					<Switch
						checked={providerConfig.enabled}
						onCheckedChange={(checked: boolean) => updateConfig({ enabled: checked })}
					/>
				</div>

				<Separator />

				<!-- Base URL -->
				<div class="space-y-2">
					<Label for="base-url">
						<Server class="w-4 h-4 inline mr-1" />
						Base URL
					</Label>
					<Input
						id="base-url"
						placeholder="http://localhost:11434"
						value={providerConfig.base_url || ''}
						oninput={(e: Event & { currentTarget: HTMLInputElement }) => updateConfig({ base_url: e.currentTarget.value || null })}
					/>
					<p class="text-xs text-muted-foreground">
						URL where Ollama is running (default: http://localhost:11434)
					</p>
				</div>

				<!-- Model Selection -->
				<div class="space-y-2">
					<Label for="model">
						<Sparkles class="w-4 h-4 inline mr-1" />
						Model
					</Label>
					<div class="flex gap-2">
						<Input
							id="model"
							placeholder="e.g., llama3.2:3b, llama3.2:7b, mistral"
							value={providerConfig.model}
							oninput={(e: Event & { currentTarget: HTMLInputElement }) => updateConfig({ model: e.currentTarget.value })}
							class="flex-1"
						/>
						<Button
							variant="outline"
							size="sm"
							onclick={loadModels}
						>
							<RefreshCw class="w-4 h-4 mr-2" />
							Load Models
						</Button>
					</div>
					{#if availableModels.length > 0}
						<div class="flex flex-wrap gap-2 mt-2">
							{#each availableModels as model}
								{@const modelName = String(model)}
								<Button
									variant="outline"
									size="sm"
									onclick={() => updateConfig({ model: modelName })}
									class={providerConfig.model === modelName ? 'bg-primary text-primary-foreground' : ''}
								>
									{modelName}
								</Button>
							{/each}
						</div>
					{/if}
					<p class="text-xs text-muted-foreground">
						Model name to use. Make sure the model is installed in Ollama (use <code class="bg-muted px-1 rounded">ollama pull &lt;model&gt;</code>).
					</p>
				</div>

				<Separator />

				<!-- Available Models Browser -->
				<Collapsible bind:open={showAvailableModels}>
					<CollapsibleTrigger
						class="flex items-center justify-between w-full rounded-lg border p-3 hover:bg-muted/50 transition-colors"
						onclick={() => {
							if (!showAvailableModels && Object.keys(availableOllamaModels).length === 0) {
								loadAvailableOllamaModels();
							}
						}}
					>
						<div class="flex items-center gap-2">
							<Package class="w-4 h-4" />
							<span class="font-medium">Browse Available Models</span>
						</div>
						{#if showAvailableModels}
							<ChevronUp class="w-4 h-4" />
						{:else}
							<ChevronDown class="w-4 h-4" />
						{/if}
					</CollapsibleTrigger>
					<CollapsibleContent>
						<div class="mt-4 space-y-4 p-4 border rounded-lg bg-muted/30">
							<!-- Search -->
							<div class="relative">
								<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
								<Input
									type="text"
									placeholder="Search models..."
									value={modelSearchQuery}
									oninput={(e: Event & { currentTarget: HTMLInputElement }) => modelSearchQuery = e.currentTarget.value}
									class="pl-10"
								/>
							</div>

							{#if isLoadingAvailableModels}
								<div class="flex items-center justify-center py-8">
									<Loader2 class="w-6 h-6 animate-spin text-muted-foreground" />
									<span class="ml-2 text-sm text-muted-foreground">Loading available models...</span>
								</div>
							{:else if Object.keys(availableOllamaModels).length === 0}
								<div class="text-center py-8">
									<p class="text-sm text-muted-foreground mb-4">No models loaded yet</p>
									<Button
										variant="outline"
										size="sm"
										onclick={loadAvailableOllamaModels}
									>
										<RefreshCw class="w-4 h-4 mr-2" />
										Load Available Models
									</Button>
								</div>
							{:else}
								{@const filtered = getFilteredModels()}
								<div class="space-y-4 max-h-[600px] overflow-y-auto">
									{#if Object.keys(filtered).length === 0}
										<div class="text-center py-8 text-sm text-muted-foreground">
											No models match your search query
										</div>
									{:else}
										{#each Object.entries(filtered) as [family, models]}
											<div class="space-y-2">
												<h4 class="font-semibold text-sm capitalize">{family}</h4>
												<div class="grid grid-cols-1 md:grid-cols-2 gap-2">
													{#each models as model}
														{@const isInstalled = isModelInstalled(model.name)}
														<div class="flex items-center justify-between p-2 rounded border bg-background hover:bg-muted/50">
															<div class="flex-1 min-w-0">
																<div class="font-medium text-sm truncate">{model.name}</div>
																{#if model.size}
																	<div class="text-xs text-muted-foreground">{model.size}</div>
																{/if}
															</div>
															<div class="flex items-center gap-2 ml-2">
																{#if isInstalled}
																	<Badge variant="outline" class="text-xs">
																		<CheckCircle2 class="w-3 h-3 mr-1" />
																		Installed
																	</Badge>
																{:else}
																	<Button
																		variant="ghost"
																		size="sm"
																		onclick={() => installModel(model.name)}
																		disabled={installingModel === model.name || !!installingModel}
																		class="h-8 px-2"
																	>
																		{#if installingModel === model.name}
																			<Loader2 class="w-3 h-3 mr-1 animate-spin" />
																		{:else}
																			<Download class="w-3 h-3 mr-1" />
																		{/if}
																		Install
																	</Button>
																{/if}
															</div>
														</div>
													{/each}
												</div>
											</div>
										{/each}
									{/if}
								</div>
							{/if}
						</div>
					</CollapsibleContent>
				</Collapsible>

				<Separator />

				<!-- Actions -->
				<div class="flex items-center justify-end gap-2">
					<Button
						variant="outline"
						onclick={testConnection}
						disabled={testingProvider || !providerConfig || (serviceStatus && !serviceStatus.running)}
					>
						{#if testingProvider}
							<Loader2 class="w-4 h-4 mr-2 animate-spin" />
						{:else}
							<RefreshCw class="w-4 h-4 mr-2" />
						{/if}
						Test Connection
					</Button>
					{#if defaultProvider !== providerType && configStatus?.is_configured}
						<Button
							variant="default"
							onclick={setAsDefault}
						>
							Set as Default
						</Button>
					{/if}
				</div>
			</CardContent>
		</Card>
	{/if}
</div>

