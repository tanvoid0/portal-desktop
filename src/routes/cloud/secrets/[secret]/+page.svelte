<!-- Secret Detail Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { ArrowLeft, RefreshCw, Trash2, Eye, EyeOff } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	
	const secretName = $derived($page.params.secret);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let secret = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let showSecretValues = $state<Record<string, boolean>>({});
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadSecret();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading && secret) {
			loadYAML();
		}
	});
	
	async function loadSecret() {
		if (!secretName || !$cloudStore.connection.isConnected) {
			error = 'Secret name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.SECRET, namespace);
			const resources = $cloudStore.resources[ResourceType.SECRET] || [];
			secret = resources.find(s => s.name === secretName) || null;
			
			if (!secret) {
				error = `Secret "${secretName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load Secret';
			console.error('Failed to load Secret:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!secret) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'Secret',
				namespace: secret.namespace,
				name: secret.name
			});
			
			yaml = yamlContent;
		} catch (err) {
			yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
			console.error('Failed to load YAML:', err);
		} finally {
			yamlLoading = false;
		}
	}
	
	async function handleSaveYAML(yamlContent: string) {
		if (!secret) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: secret.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload Secret to get updated data
			await loadSecret();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to apply YAML';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	async function handleDelete() {
		if (!secret) return;
		
		if (!confirm(`Are you sure you want to delete Secret "${secret.name}"? This action cannot be undone.`)) {
			return;
		}
		
		try {
			await invoke('k8s_delete_secret', {
				namespace: secret.namespace,
				name: secret.name
			});
			
			toastActions.success(`Secret "${secret.name}" deleted successfully`);
			goto('/cloud/secrets');
		} catch (err) {
			toastActions.error(err instanceof Error ? err.message : 'Failed to delete Secret');
		}
	}
	
	function handleTabChange(tab: string) {
		activeTab = tab;
		const url = new URL(window.location.href);
		url.searchParams.set('tab', tab);
		window.history.replaceState({}, '', url.toString());
	}
	
	function toggleSecretValue(key: string) {
		showSecretValues[key] = !showSecretValues[key];
		showSecretValues = { ...showSecretValues };
	}
	
	function decodeBase64(base64: string): string {
		try {
			// The data is already base64-encoded from the backend
			// We need to decode it to show the actual value
			const decoded = atob(base64);
			return decoded;
		} catch (error) {
			return '[Decode Error]';
		}
	}
	
	const dataEntries = $derived.by(() => {
		if (!secret || !secret.metadata?.data) return [];
		return Object.entries(secret.metadata.data);
	});
	
	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
		toastActions.success('Copied to clipboard');
	}
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading Secret details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/secrets')} class="mt-4">Back to Secrets</Button>
		</div>
	{:else if secret}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Secret: {secret.name}</h1>
				<p class="text-muted-foreground">Namespace: {secret.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadSecret}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="destructive" size="sm" onclick={handleDelete}>
					<Trash2 class="mr-2 h-4 w-4" />
					Delete
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/secrets')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Secrets
				</Button>
			</div>
		</div>
		
		<!-- Warning -->
		<div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
			<p class="text-sm text-yellow-800 dark:text-yellow-200">
				<strong>Security Warning:</strong> Secret values contain sensitive information. Be careful when viewing or editing.
			</p>
		</div>
		
		<!-- Tabs -->
		<Tabs value={activeTab} onValueChange={handleTabChange}>
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="data">Data</TabsTrigger>
				<TabsTrigger value="yaml">YAML</TabsTrigger>
			</TabsList>
			
			<!-- Overview Tab -->
			<TabsContent value="overview" class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Card>
						<CardHeader>
							<CardTitle>Secret Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Name</p>
								<p class="font-medium">{secret.name}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Namespace</p>
								<p class="font-medium">{secret.namespace}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Type</p>
								<Badge variant="outline">{secret.metadata?.type || 'Opaque'}</Badge>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Data Keys</p>
								<p class="font-medium">{secret.metadata?.dataCount || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{secret.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					{#if secret.metadata?.labels && Object.keys(secret.metadata.labels).length > 0}
						<Card>
							<CardHeader>
								<CardTitle>Labels</CardTitle>
							</CardHeader>
							<CardContent>
								<div class="flex flex-wrap gap-2">
									{#each Object.entries(secret.metadata.labels) as [key, value]}
										<Badge variant="outline">{key}={value}</Badge>
									{/each}
								</div>
							</CardContent>
						</Card>
					{/if}
				</div>
			</TabsContent>
			
			<!-- Data Tab -->
			<TabsContent value="data" class="space-y-4">
				<Card>
					<CardHeader>
						<CardTitle>Secret Data</CardTitle>
					</CardHeader>
					<CardContent>
						{#if dataEntries.length === 0}
							<div class="text-center py-8 text-muted-foreground">
								<p>No data entries</p>
							</div>
						{:else}
							<div class="space-y-4">
								{#each dataEntries as [key, base64Value]}
									<div class="border rounded-lg p-4">
										<div class="flex items-center justify-between mb-2">
											<span class="font-medium">{key}</span>
											<div class="flex items-center gap-2">
												<Button 
													variant="ghost" 
													size="sm"
													onclick={() => toggleSecretValue(key)}
												>
													{#if showSecretValues[key]}
														<EyeOff class="h-4 w-4 mr-1" />
													{:else}
														<Eye class="h-4 w-4 mr-1" />
													{/if}
													{showSecretValues[key] ? 'Hide' : 'Show'}
												</Button>
												<Button 
													variant="ghost" 
													size="sm"
													onclick={() => {
														const value = showSecretValues[key] 
															? decodeBase64(String(base64Value)) 
															: String(base64Value);
														copyToClipboard(value);
													}}
												>
													Copy
												</Button>
											</div>
										</div>
										<pre class="text-sm bg-muted p-3 rounded overflow-auto whitespace-pre-wrap break-words">
											{#if showSecretValues[key]}
												{decodeBase64(String(base64Value))}
											{:else}
												{'*'.repeat(20)} (hidden)
											{/if}
										</pre>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="space-y-4">
				{#if yamlLoading}
					<div class="flex items-center justify-center h-[600px]">
						<Loading text="Loading YAML..." />
					</div>
				{:else if yamlError}
					<div class="text-destructive text-center h-[600px] flex items-center justify-center">
						<p>{yamlError}</p>
					</div>
				{:else if yaml}
					<YamlEditor
						value={yaml}
						onSave={handleSaveYAML}
						resourceName={secret.name}
						resourceKind="Secret"
						namespace={secret.namespace}
					/>
				{:else}
					<div class="text-muted-foreground text-center h-[600px] flex items-center justify-center">
						<p>No YAML available.</p>
					</div>
				{/if}
			</TabsContent>
		</Tabs>
	{/if}
</div>

