<script lang="ts">
	import Select from '$lib/components/ui/select.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import type { ProviderType } from '../../types/index.js';
	import { aiProviderService } from '../../services/aiProviderService.js';

	interface Props {
		selectedProvider?: ProviderType | null;
		onProviderChange?: (provider: ProviderType) => void;
	}

	let {
		selectedProvider = $bindable<ProviderType | null>(null),
		onProviderChange
	}: Props = $props();

	let providers = $state<Array<{ value: ProviderType; label: string }>>([
		{ value: 'Ollama', label: 'Ollama' },
		{ value: 'Gemini', label: 'Gemini' }
	]);
	let defaultProvider = $state<ProviderType | null>(null);
	let isLoading = $state(false);

	async function loadDefaultProvider() {
		isLoading = true;
		try {
			defaultProvider = await aiProviderService.getDefaultProvider();
			if (!selectedProvider && defaultProvider) {
				selectedProvider = defaultProvider;
			}
		} catch (error) {
			console.error('Failed to load default provider:', error);
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		loadDefaultProvider();
	});

	function handleProviderChange(value: string) {
		const provider = value as ProviderType;
		selectedProvider = provider;
		if (onProviderChange) {
			onProviderChange(provider);
		}
	}
</script>

<div class="flex items-center gap-2">
	<Select
		options={providers}
		value={selectedProvider || undefined}
		onSelect={handleProviderChange}
		placeholder="Select provider"
		disabled={isLoading}
		class="w-[180px]"
	/>
	{#if selectedProvider === defaultProvider}
		<Badge variant="secondary" class="text-xs">Default</Badge>
	{/if}
</div>

