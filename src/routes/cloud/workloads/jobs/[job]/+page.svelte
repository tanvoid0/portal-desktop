<!-- Job Detail Page -->
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
	import { ArrowLeft, RefreshCw, FileCode } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import YamlEditor from '$lib/domains/cloud/components/YamlEditor.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	
	const jobName = $derived($page.params.job);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	let job = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	onMount(async () => {
		await loadJob();
		if (activeTab === 'yaml') {
			await loadYAML();
		}
	});
	
	$effect(() => {
		if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadJob() {
		if (!jobName || !$cloudStore.connection.isConnected) {
			error = 'Job name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			await loadResources(ResourceType.JOB, namespace);
			const resources = $cloudStore.resources[ResourceType.JOB] || [];
			job = resources.find(j => j.name === jobName) || null;
			
			if (!job) {
				error = `Job "${jobName}" not found in namespace "${namespace}".`;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load job';
			console.error('Failed to load job:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadYAML() {
		if (!job) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			const yamlContent = await invoke<string>('k8s_get_resource_yaml', {
				kind: 'Job',
				namespace: job.namespace,
				name: job.name
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
		if (!job) return;
		
		try {
			const result = await invoke<string>('k8s_apply_resource_yaml', {
				namespace: job.namespace,
				yamlContent: yamlContent
			});
			
			toastActions.success(result);
			
			// Reload job to get updated data
			await loadJob();
			await loadYAML();
		} catch (err) {
			const errorMsg = err instanceof Error ? err.message : 'Failed to apply YAML';
			toastActions.error(errorMsg);
			throw err;
		}
	}
	
	function handleTabChange(tab: string) {
		activeTab = tab;
		const url = new URL(window.location.href);
		url.searchParams.set('tab', tab);
		window.history.replaceState({}, '', url.toString());
	}
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading text="Loading job details..." />
		</div>
	{:else if error}
		<div class="text-center py-12 text-destructive">
			<p>{error}</p>
			<Button onclick={() => goto('/cloud/workloads/jobs')} class="mt-4">Back to Jobs</Button>
		</div>
	{:else if job}
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Job: {job.name}</h1>
				<p class="text-muted-foreground">Namespace: {job.namespace}</p>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={loadJob}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={() => goto('/cloud/workloads/jobs')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Jobs
				</Button>
			</div>
		</div>
		
		<!-- Tabs -->
		<Tabs value={activeTab} onValueChange={handleTabChange}>
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="yaml">YAML</TabsTrigger>
			</TabsList>
			
			<!-- Overview Tab -->
			<TabsContent value="overview" class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Card>
						<CardHeader>
							<CardTitle>Job Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Status</p>
								<Badge class="mt-1" variant="default">{job.status}</Badge>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Completions</p>
								<p class="font-medium">{job.metadata?.completions || 'N/A'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Succeeded</p>
								<p class="font-medium">{job.metadata?.succeeded || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Failed</p>
								<p class="font-medium">{job.metadata?.failed || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Active</p>
								<p class="font-medium">{job.metadata?.active || 0}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Age</p>
								<p class="font-medium">{job.metadata?.age || 'N/A'}</p>
							</div>
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader>
							<CardTitle>Configuration</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<div>
								<p class="text-sm text-muted-foreground">Parallelism</p>
								<p class="font-medium">{job.metadata?.parallelism || 'N/A'}</p>
							</div>
							<div>
								<p class="text-sm text-muted-foreground">Backoff Limit</p>
								<p class="font-medium">{job.metadata?.backoff_limit || 'N/A'}</p>
							</div>
							{#if job.metadata?.image}
								<div>
									<p class="text-sm text-muted-foreground">Image</p>
									<p class="font-medium">{job.metadata.image}</p>
								</div>
							{/if}
						</CardContent>
					</Card>
				</div>
				
				{#if job.metadata && Object.keys(job.metadata).length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Metadata</CardTitle>
						</CardHeader>
						<CardContent>
							<pre class="text-xs bg-muted p-4 rounded-lg overflow-auto">{JSON.stringify(job.metadata, null, 2)}</pre>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="h-[600px]">
				<Card class="h-full flex flex-col">
					<CardHeader>
						<CardTitle>Job YAML</CardTitle>
					</CardHeader>
					<CardContent class="flex-1 overflow-hidden">
						{#if yamlLoading}
							<div class="flex items-center justify-center h-full">
								<Loading text="Loading YAML..." />
							</div>
						{:else if yamlError}
							<div class="text-destructive text-center h-full flex items-center justify-center">
								<p>{yamlError}</p>
							</div>
						{:else if yaml}
							<YamlEditor
								value={yaml}
								onSave={handleSaveYAML}
								resourceName={job.name}
								resourceKind="Job"
								namespace={job.namespace}
							/>
						{:else}
							<div class="text-muted-foreground text-center h-full flex items-center justify-center">
								<p>No YAML available.</p>
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
		</Tabs>
	{/if}
</div>

