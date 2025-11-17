<!-- Namespace Selector Component -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudStore, loadResources, setSelectedNamespace } from '$lib/domains/cloud/stores';
	import { ResourceType } from '$lib/domains/cloud/core/types';
	import { SearchableSelect } from '@/lib/components/ui/searchable-select';
	import { invoke } from '@tauri-apps/api/core';
	import { useNamespaceShortcuts } from '$lib/domains/k8s-navigation';
	import KeyboardShortcutHint from '$lib/domains/k8s-navigation/components/KeyboardShortcutHint.svelte';
	
	// Get namespace list for selector
	const namespaceOptions = $derived(() => {
		const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];
		const names = namespaces.map((ns: any) => ns.name).sort();
		return [
			{ value: '', label: 'All Namespaces' },
			...names.map(name => ({ value: name, label: name }))
		];
	});
	
	// Derived namespace shortcuts array
	const namespaceShortcutsArray = $derived(() => {
		return namespaceOptions().slice(1).map((opt, index) => ({
			value: opt.value,
			label: opt.label,
			shortcut: index < 9 ? index + 1 : undefined
		}));
	});
	
	// Namespace shortcuts - pass getter function to make it reactive
	const namespaceShortcuts = useNamespaceShortcuts({
		namespaces: () => namespaceShortcutsArray(),
		selectedNamespace: $cloudStore.selectedNamespace || '',
		onSelect: async (namespace) => {
			await handleNamespaceChange(namespace);
		},
		enabled: $cloudStore.connection.isConnected
	});
	
	function handleKeydown(event: KeyboardEvent) {
		namespaceShortcuts.handleKeydown(event);
	}
	
	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
	
	async function handleNamespaceChange(namespace: string) {
		await setSelectedNamespace(namespace);
		
		// Reload resources for the new namespace
		if ($cloudStore.connection.isConnected) {
			await Promise.all([
				loadResources(ResourceType.POD, namespace || undefined),
				loadResources(ResourceType.SERVICE, namespace || undefined),
				loadResources(ResourceType.DEPLOYMENT, namespace || undefined),
				loadResources(ResourceType.JOB, namespace || undefined),
				loadResources(ResourceType.CRONJOB, namespace || undefined),
				loadResources(ResourceType.CONFIGMAP, namespace || undefined),
				loadResources(ResourceType.SECRET, namespace || undefined),
				loadResources(ResourceType.INGRESS, namespace || undefined)
			]);
			
			// Reload metrics if available
			try {
				const metrics = await invoke<Record<string, any>>('k8s_get_all_pods_metrics', {
					namespace: namespace || null
				});
				// Metrics will be handled by individual pages that need them
			} catch (error) {
				// Metrics might not be available, ignore
				console.debug('Metrics not available:', error);
			}
		}
	}
	
	// Load namespaces on mount if connected
	let hasLoadedNamespaces = $state(false);
	$effect(() => {
		const isConnected = $cloudStore.connection.isConnected;
		const namespaceCount = $cloudStore.resources[ResourceType.NAMESPACE].length;
		
		// Reset flag when disconnected
		if (!isConnected) {
			hasLoadedNamespaces = false;
			return;
		}
		
		// Load namespaces only once when connected and namespaces list is empty
		if (isConnected && !hasLoadedNamespaces && namespaceCount === 0) {
			hasLoadedNamespaces = true;
			// Use setTimeout to break reactive cycle
			setTimeout(() => {
				loadResources(ResourceType.NAMESPACE).catch(err => {
					console.error('Failed to load namespaces:', err);
					hasLoadedNamespaces = false; // Reset on error so we can retry
				});
			}, 0);
		}
	});
</script>

{#if $cloudStore.connection.isConnected}
	<div class="flex items-center gap-2 px-4 py-2 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
		<div class="flex items-center gap-2 flex-1">
			<span class="text-sm font-medium text-muted-foreground">Namespace:</span>
			<SearchableSelect
				options={namespaceOptions()}
				value={$cloudStore.selectedNamespace || ''}
				placeholder="All Namespaces"
				searchPlaceholder="Search namespaces..."
				onValueChange={handleNamespaceChange}
				disabled={!$cloudStore.connection.isConnected}
				class="w-64"
			/>
			{#if namespaceShortcuts.namespaceShortcuts().size > 1}
				<div class="flex items-center gap-1 text-xs text-muted-foreground">
					<span>Quick:</span>
					{#each Array.from(namespaceShortcuts.namespaceShortcuts().entries()).slice(0, 5) as [number, namespace]}
						<span class="px-1">
							<KeyboardShortcutHint shortcut={number.toString()} variant="subtle" />
							<span class="ml-1">{namespace.label.slice(0, 8)}</span>
						</span>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}

