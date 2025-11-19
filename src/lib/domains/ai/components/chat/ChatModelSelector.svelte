<script lang="ts">
	import Select from '$lib/components/ui/select.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Loader } from 'lucide-svelte';
	import type { ProviderType } from '../../types/index.js';
	import { aiProviderService } from '../../services/aiProviderService.js';
	import { onMount } from 'svelte';

	interface Props {
		selectedProvider?: ProviderType | null;
		selectedModel?: string | null;
		onModelChange?: (model: string) => void;
	}

	let {
		selectedProvider = $bindable<ProviderType | null>(null),
		selectedModel = $bindable<string | null>(null),
		onModelChange
	}: Props = $props();

	let availableModels = $state<string[]>([]);
	let isLoading = $state(false);
	let defaultModel = $state<string | null>(null);

	async function loadModels() {
		if (!selectedProvider) {
			availableModels = [];
			return;
		}

		isLoading = true;
		try {
			// Get available models for the provider
			availableModels = await aiProviderService.getAvailableModels(selectedProvider);
			
			// Get the default model from provider config
			const config = await aiProviderService.getProviderConfig(selectedProvider);
			defaultModel = config.model;
			
			// Set selected model to default if not set
			if (!selectedModel && defaultModel) {
				selectedModel = defaultModel;
				if (onModelChange) {
					onModelChange(defaultModel);
				}
			}
		} catch (error) {
			console.error('Failed to load models:', error);
			availableModels = [];
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		if (selectedProvider) {
			loadModels();
		} else {
			availableModels = [];
			selectedModel = null;
		}
	});

	function handleModelChange(value: string) {
		selectedModel = value;
		if (onModelChange) {
			onModelChange(value);
		}
	}

	const modelOptions = $derived(
		availableModels.map((model) => ({ value: model, label: model }))
	);
</script>

<div class="flex items-center gap-2">
	<Select
		options={modelOptions}
		value={selectedModel || undefined}
		onSelect={handleModelChange}
		placeholder={isLoading ? 'Loading...' : 'Select model'}
		disabled={isLoading || !selectedProvider || availableModels.length === 0}
		class="w-[200px]"
	/>
	{#if selectedModel === defaultModel}
		<Badge variant="secondary" class="text-xs">Default</Badge>
	{/if}
	{#if isLoading}
		<Loader class="h-4 w-4 animate-spin text-muted-foreground" />
	{/if}
</div>

