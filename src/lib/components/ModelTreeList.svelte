<script lang="ts">
	import { Button } from './ui/button';
	import { Badge } from './ui/badge';
	import { parseAndFormatSize } from '$lib/utils/fileSize';
	
	console.log('ModelTreeList: Component mounted');
	
	let {
		models = {},
		loading = false,
		error = null,
		installingModel = null,
		installationProgress = 0,
		installationStatus = '',
		onInstall = undefined,
		onRemove = undefined,
		onRetry = undefined,
		onCancel = undefined
	}: {
		models?: Record<string, any[]>;
		loading?: boolean;
		error?: string | null;
		installingModel?: string | null;
		installationProgress?: number;
		installationStatus?: string;
		onInstall?: ((modelName: string) => void) | undefined;
		onRemove?: ((modelName: string) => void) | undefined;
		onRetry?: (() => void) | undefined;
		onCancel?: (() => void) | undefined;
	} = $props();
	
	// Debug: Log props on mount and changes
	$effect(() => {
		console.log('ModelTreeList: props changed', { 
			models, 
			modelsType: typeof models,
			modelsKeys: Object.keys(models || {}),
			loading, 
			error 
		});
	});
	
	// Search functionality
	let searchQuery = $state('');
	let expandedFamilies = $state<string[]>([]);
	
	// Filter models based on search
	let filteredModels = $state<Record<string, any[]>>({});
	
	// Update filtered models when models or search query changes
	$effect(() => {
		console.log('ModelTreeList: filtering models called', { 
			models, 
			modelsType: typeof models,
			modelsKeys: Object.keys(models || {}),
			searchQuery,
			modelsIsEmpty: Object.keys(models || {}).length === 0
		});
		
		if (!searchQuery.trim()) {
			console.log('ModelTreeList: returning all models (no search)', models);
			filteredModels = models;
			return;
		}
		
		const query = searchQuery.toLowerCase();
		const filtered: Record<string, any[]> = {};
		
		for (const [family, modelList] of Object.entries(models)) {
			const matchingModels = modelList.filter(model => 
				model.name.toLowerCase().includes(query) || 
				model.family?.toLowerCase().includes(query)
			);
			
			if (matchingModels.length > 0) {
				filtered[family] = matchingModels;
			}
		}
		
		console.log('ModelTreeList: filtered models', filtered);
		filteredModels = filtered;
	});
	
	function toggleFamily(family: string) {
		if (expandedFamilies.includes(family)) {
			expandedFamilies = expandedFamilies.filter(f => f !== family);
		} else {
			expandedFamilies = [...expandedFamilies, family];
		}
	}
	
	function isInstalled(modelName: string): boolean {
		// This would need to be passed from parent or checked against local models
		return false; // Placeholder - would need actual implementation
	}
	
	function handleModelAction(model: any) {
		if (isInstalled(model.name) && onRemove) {
			onRemove(model.name);
		} else if (!isInstalled(model.name) && onInstall) {
			onInstall(model.name);
		}
	}
</script>

{#if loading}
	<div class="flex items-center justify-center p-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		<span class="ml-2">Loading models...</span>
	</div>
{:else if error}
	<div class="text-center p-8">
		<p class="text-destructive">Error: {error}</p>
		{#if onRetry}
			<Button onclick={onRetry} class="mt-4">Retry</Button>
		{/if}
	</div>
{:else if Object.keys(filteredModels).length > 0 || installingModel}
	<!-- Debug info -->
	<div class="text-xs text-muted-foreground p-2 bg-gray-100 rounded">
		Debug: filteredModels keys: {Object.keys(filteredModels).length}, installingModel: {installingModel}
	</div>
	<div class="space-y-4">
		<!-- Search -->
		<div class="relative">
			<input
				type="text"
				placeholder="Search models..."
				bind:value={searchQuery}
				class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
			/>
			<svg class="absolute right-3 top-3 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
			</svg>
		</div>
		
		<!-- Model Families -->
		<div class="space-y-2">
			{#each Object.entries(filteredModels) as [family, modelList]}
				<div class="border rounded-lg">
					<!-- Family Header -->
					<button
						onclick={() => toggleFamily(family)}
						class="w-full flex items-center justify-between p-4 hover:bg-gray-50 transition-colors"
					>
						<div class="flex items-center gap-3">
							<svg 
								class="w-4 h-4 transition-transform {expandedFamilies.includes(family) ? 'rotate-90' : ''}" 
								fill="none" 
								stroke="currentColor" 
								viewBox="0 0 24 24"
							>
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
							</svg>
							<span class="font-medium">{family}</span>
							<Badge variant="secondary">{modelList.length} models</Badge>
						</div>
					</button>
					
					<!-- Family Models -->
					{#if expandedFamilies.includes(family)}
						<div class="border-t">
							{#each modelList as model}
								<div class="flex items-center justify-between p-4 hover:bg-gray-50 transition-colors border-b last:border-b-0">
									<div class="flex items-center gap-4">
										<div class="flex items-center gap-2">
											{#if isInstalled(model.name)}
												<svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
												</svg>
											{:else}
												<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
												</svg>
											{/if}
											<span class="font-medium">{model.name}</span>
										</div>
										<div class="text-sm text-muted-foreground">
											{#if model.size}
												{parseAndFormatSize(model.size)}
											{:else}
												Unknown size
											{/if}
										</div>
									</div>
									<div class="flex items-center gap-2">
										{#if installingModel === model.name}
											<!-- Installation Progress -->
											<div class="flex items-center gap-3 bg-blue-50 px-3 py-2 rounded-lg border border-blue-200 min-w-0 flex-1">
												<div class="relative">
													<div class="animate-spin rounded-full h-6 w-6 border-2 border-blue-200"></div>
													<div class="absolute inset-0 flex items-center justify-center">
														<Badge variant="default" class="bg-blue-600 text-white text-xs px-1.5 py-0.5">
															{Math.round(installationProgress)}%
														</Badge>
													</div>
												</div>
												<div class="flex-1 min-w-0">
													<div class="text-sm font-medium text-blue-800">Installing {model.name}</div>
													<div class="text-xs text-blue-600">{installationStatus}</div>
													<div class="w-full bg-blue-200 rounded-full h-1.5 mt-1">
														<div class="bg-blue-600 h-1.5 rounded-full transition-all duration-300" style="width: {installationProgress}%"></div>
													</div>
												</div>
												{#if onCancel}
													<Button 
														size="sm" 
														variant="outline" 
														onclick={onCancel}
														class="text-red-600 border-red-300 hover:bg-red-50 hover:border-red-400"
													>
														<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
														</svg>
														Stop
													</Button>
												{/if}
											</div>
										{:else if isInstalled(model.name)}
											<Badge variant="default" class="bg-green-100 text-green-800">
												<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
													<path d="M5 13l4 4L19 7"/>
												</svg>
												Installed
											</Badge>
											{#if onRemove}
												<Button size="sm" variant="destructive" onclick={() => onRemove(model.name)}>
													<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
													</svg>
													Remove
												</Button>
											{/if}
										{:else}
											<Badge variant="outline" class="text-gray-600">
												Available in Ollama Library
											</Badge>
											{#if onInstall}
												<Button size="sm" onclick={() => onInstall(model.name)}>
													<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
													</svg>
													Install
												</Button>
											{/if}
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
{:else}
	<!-- Debug info -->
	<div class="text-xs text-muted-foreground p-2 bg-yellow-100 rounded">
		Debug: No models condition - filteredModels keys: {Object.keys(filteredModels).length}, installingModel: {installingModel}
	</div>
	<div class="text-center p-8">
		<p class="text-muted-foreground">No models available.</p>
		{#if onRetry}
			<Button onclick={onRetry} class="mt-4">Load Models</Button>
		{/if}
	</div>
{/if}
