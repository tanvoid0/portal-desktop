<!-- Pod Detail Page -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
	import { get } from 'svelte/store';
	import { ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/lib/components/ui/tabs';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';
	import { Label } from '@/lib/components/ui/label';
	import { Input } from '@/lib/components/ui/input';
	import { ArrowLeft, RefreshCw, Download, Terminal, FileCode, Network } from '@lucide/svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Loading from '@/lib/components/ui/loading.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import type { PortForwardInfo } from '$lib/domains/cloud/providers/gcp/GCPTypes';
	import LogSearchPanel from '$lib/domains/cloud/components/logs/LogSearchPanel.svelte';
	import LogsDisplay from '$lib/domains/cloud/components/logs/LogsDisplay.svelte';
	import { parseRawLogs } from '$lib/domains/cloud/utils/logParser';
	import type { K8sLog } from '$lib/domains/cloud/types/k8s';
	import MetricsDisplay from '$lib/domains/cloud/components/MetricsDisplay.svelte';
	
	const podName = $derived($page.params.pod);
	const namespace = $derived($page.url.searchParams.get('namespace') || $cloudStore.selectedNamespace || 'default');
	const tabParam = $derived($page.url.searchParams.get('tab') || 'overview');
	
	let pod = $state<ICloudResource | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let activeTab = $state('overview');
	
	// Sync activeTab with tabParam when it changes
	$effect(() => {
		activeTab = tabParam;
	});
	
	// Logs state
	let logs = $state('');
	let logsLoading = $state(false);
	let logsError = $state<string | null>(null);
	let logsFollowing = $state(false);
	
	// Log search/filter state
	let logSearchQuery = $state('');
	let selectedLogContainer = $state('');
	let selectedLogSeverity = $state('');
	let logTailLines = $state(1000);
	let logViewMode = $state<'detailed' | 'compact' | 'raw'>('detailed');
	let logContainers = $derived.by(() => {
		if (!pod?.metadata?.containers) return [];
		const containers = pod.metadata.containers;
		if (Array.isArray(containers)) {
			return containers.map((c: any) => c.name || c).filter(Boolean);
		}
		return [];
	});
	
	// Parse logs into structured format
	const parsedLogs = $derived.by(() => {
		if (!logs || !pod) return [];
		const containerName = selectedLogContainer || logContainers[0] || 'app';
		return parseRawLogs(logs, pod.name, containerName);
	});
	
	// YAML state
	let yaml = $state('');
	let yamlLoading = $state(false);
	let yamlError = $state<string | null>(null);
	
	// Port forwarding state
	let showPortForwardDialog = $state(false);
	let portForwardLocalPort = $state(8080);
	let portForwardRemotePort = $state(80);
	let isPortForwarding = $state(false);
	let portForwardMessage = $state<string | null>(null);
	let activePortForwards = $state<any[]>([]);
	let loadingPortForwards = $state(false);
	
	// Metrics state
	let podMetrics = $state<any | null>(null);
	let metricsLoading = $state(false);
	let metricsError = $state<string | null>(null);
	
	onMount(async () => {
		await loadPod();
		if (activeTab === 'logs') {
			await loadLogs();
		} else if (activeTab === 'yaml') {
			await loadYAML();
		}
		await loadPortForwards();
		await loadMetrics();
	});
	
	async function loadPortForwards() {
		if (!pod || !$cloudStore.connection.isConnected) return;
		
		loadingPortForwards = true;
		try {
			const forwards = await invoke<PortForwardInfo[]>('k8s_list_port_forwards');
			// Filter to only show forwards for this pod
			activePortForwards = forwards.filter(f => 
				f.pod_name === pod.name && f.namespace === pod.namespace
			);
		} catch (error) {
			console.error('Failed to load port forwards:', error);
		} finally {
			loadingPortForwards = false;
		}
	}
	
	async function stopPortForward(id: string) {
		try {
			await invoke('k8s_stop_port_forward', { id });
			toastActions.success('Port forward stopped');
			await loadPortForwards();
		} catch (error) {
			toastActions.error(`Failed to stop port forward: ${error instanceof Error ? error.message : 'Unknown error'}`);
		}
	}
	
	$effect(() => {
		if (activeTab === 'logs' && !logs && !logsLoading) {
			loadLogs();
		} else if (activeTab === 'yaml' && !yaml && !yamlLoading) {
			loadYAML();
		}
	});
	
	async function loadPod() {
		if (!podName || !$cloudStore.connection.isConnected) {
			error = 'Pod name or connection required';
			isLoading = false;
			return;
		}
		
		try {
			isLoading = true;
			error = null;
			
			// Load pods into store
			await loadResources(ResourceType.POD, namespace);
			
			// Get pods from store after loading
			const pods = get(cloudStore).resources[ResourceType.POD];
			if (!pods || !Array.isArray(pods)) {
				error = `Failed to load pods from namespace "${namespace}"`;
				isLoading = false;
				return;
			}
			
			const foundPod = pods.find((p: ICloudResource) => p.name === podName);
			
			if (!foundPod) {
				error = `Pod "${podName}" not found in namespace "${namespace}"`;
				isLoading = false;
				return;
			}
			
			pod = foundPod;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load pod';
			console.error('Failed to load pod:', err);
		} finally {
			isLoading = false;
		}
	}
	
	async function loadLogs(container?: string, tailLines?: number) {
		if (!pod || logsLoading) return;
		
		try {
			logsLoading = true;
			logsError = null;
			
			const containerToUse = container || selectedLogContainer || null;
			const tailLinesToUse = tailLines || logTailLines || 1000;
			
			// Use the getLogs method from the pod resource if available
			if (pod.getLogs) {
				logs = await pod.getLogs(containerToUse || undefined, tailLinesToUse || undefined);
			} else {
				// Fallback to direct invoke
				logs = await invoke<string>('k8s_get_pod_logs', {
					namespace: pod.namespace,
					podName: pod.name,
					container: containerToUse,
					follow: false,
					tailLines: tailLinesToUse
				});
			}
		} catch (err) {
			logsError = err instanceof Error ? err.message : 'Failed to load logs';
			console.error('Failed to load logs:', err);
			toastActions.error(`Failed to load logs: ${logsError}`);
		} finally {
			logsLoading = false;
		}
	}
	
	function handleLogSearchChange(query: string) {
		logSearchQuery = query;
		// Client-side filtering, no need to reload
	}
	
	function handleLogContainerChange(container: string) {
		selectedLogContainer = container;
		// Reload logs with new container filter
		loadLogs(container, logTailLines);
	}
	
	function handleLogSeverityChange(severity: string) {
		selectedLogSeverity = severity;
		// Client-side filtering, no need to reload
	}
	
	function handleLogTailLinesChange(lines: number) {
		logTailLines = lines;
		// Reload logs with new tail lines
		loadLogs(selectedLogContainer || undefined, lines);
	}
	
	function handleClearLogFilters() {
		logSearchQuery = '';
		selectedLogContainer = '';
		selectedLogSeverity = '';
		logTailLines = 1000;
		// Reload logs with default filters
		loadLogs();
	}
	
	async function loadYAML() {
		if (!pod || yamlLoading) return;
		
		try {
			yamlLoading = true;
			yamlError = null;
			
			yaml = await invoke<string>('k8s_get_pod_yaml', {
				namespace: pod.namespace,
				podName: pod.name
			});
		} catch (err) {
			yamlError = err instanceof Error ? err.message : 'Failed to load YAML';
			console.error('Failed to load YAML:', err);
		} finally {
			yamlLoading = false;
		}
	}
	
	function generateYAMLFromPod(pod: ICloudResource): string {
		// Generate a basic YAML representation
		return `apiVersion: v1
kind: Pod
metadata:
  name: ${pod.name}
  namespace: ${pod.namespace}
  labels:
    ${Object.entries(pod.metadata || {}).map(([k, v]) => `    ${k}: ${v}`).join('\n')}
spec:
  containers:
    - name: ${pod.name}
      image: ${pod.metadata?.image || 'unknown'}
status:
  phase: ${pod.status}
  podIP: ${pod.metadata?.ip || 'N/A'}
`;
	}
	
	function handleTabChange(tab: string) {
		activeTab = tab;
		// Update URL without navigation
		const url = new URL(window.location.href);
		url.searchParams.set('tab', tab);
		window.history.replaceState({}, '', url.toString());
	}
	
	function handleExec() {
		// Navigate to exec tab or open terminal
		handleTabChange('exec');
	}
	
	async function handleDelete() {
		if (!pod) return;
		
		if (confirm(`Are you sure you want to delete pod "${pod.name}"?`)) {
			try {
				await invoke('k8s_delete_pod', {
					namespace: pod.namespace,
					podName: pod.name
				});
				// Navigate back to pods list
				goto('/cloud/workloads/pods');
			} catch (err) {
				alert(`Failed to delete pod: ${err instanceof Error ? err.message : 'Unknown error'}`);
			}
		}
	}
	
	async function loadMetrics() {
		if (!pod || !$cloudStore.connection.isConnected) return;
		
		try {
			metricsLoading = true;
			metricsError = null;
			const metrics = await invoke<any>('k8s_get_pod_metrics', {
				namespace: pod.namespace,
				pod_name: pod.name
			});
			podMetrics = metrics;
		} catch (err) {
			metricsError = err instanceof Error ? err.message : 'Failed to load metrics';
			console.error('Failed to load metrics:', err);
			podMetrics = null;
		} finally {
			metricsLoading = false;
		}
	}
	
	const containers = $derived(() => {
		if (!pod || !pod.metadata?.containers) return [];
		return Array.isArray(pod.metadata.containers) 
			? pod.metadata.containers 
			: [pod.metadata.containers];
	});
</script>


<div class="p-6 space-y-6">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-[400px]">
			<Loading size="lg" text="Loading pod details..." />
		</div>
	{:else if error}
		<Card>
			<CardHeader>
				<CardTitle>Error</CardTitle>
			</CardHeader>
			<CardContent>
				<p class="text-destructive">{error}</p>
				<Button onclick={() => goto('/cloud/workloads/pods')} variant="outline" class="mt-4">
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back to Pods
				</Button>
			</CardContent>
		</Card>
	{:else if pod}
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<Button variant="ghost" size="sm" onclick={() => goto('/cloud/workloads/pods')}>
					<ArrowLeft class="mr-2 h-4 w-4" />
					Back
				</Button>
				<div>
					<h1 class="text-3xl font-bold">{pod.name}</h1>
					<p class="text-muted-foreground">
						Namespace: {pod.namespace} • Status: 
						<Badge variant={pod.status === 'running' ? 'default' : 'secondary'} class="ml-2">
							{pod.status}
						</Badge>
					</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={async () => { await loadPod(); await loadMetrics(); }}>
					<RefreshCw class="mr-2 h-4 w-4" />
					Refresh
				</Button>
				<Button variant="outline" size="sm" onclick={() => showPortForwardDialog = true}>
					<Network class="mr-2 h-4 w-4" />
					Port Forward
					{#if activePortForwards.length > 0}
						<Badge variant="secondary" class="ml-2">{activePortForwards.length}</Badge>
					{/if}
				</Button>
				<Button variant="outline" size="sm" onclick={handleExec}>
					<Terminal class="mr-2 h-4 w-4" />
					Exec
				</Button>
				<Button variant="destructive" size="sm" onclick={handleDelete}>
					Delete
				</Button>
			</div>
		</div>
		
		<!-- Tabs -->
		<Tabs value={activeTab} onValueChange={handleTabChange}>
			<TabsList>
				<TabsTrigger value="overview">Overview</TabsTrigger>
				<TabsTrigger value="logs">Logs</TabsTrigger>
				<TabsTrigger value="yaml">YAML</TabsTrigger>
				<TabsTrigger value="port-forwards">
					Port Forwards
					{#if activePortForwards.length > 0}
						<Badge variant="secondary" class="ml-2">{activePortForwards.length}</Badge>
					{/if}
				</TabsTrigger>
				<TabsTrigger value="exec">Exec</TabsTrigger>
			</TabsList>
			
			<!-- Overview Tab -->
			<TabsContent value="overview" class="space-y-4">
				<!-- Metrics -->
				{#if podMetrics || metricsLoading}
					<MetricsDisplay
						metrics={podMetrics}
						title="Pod Resource Usage"
						showDetails={true}
					/>
				{/if}
				
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<Card>
						<CardHeader>
							<CardTitle>Pod Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-2">
							<div class="flex justify-between">
								<span class="text-muted-foreground">Name:</span>
								<span class="font-medium">{pod.name}</span>
							</div>
							<div class="flex justify-between">
								<span class="text-muted-foreground">Namespace:</span>
								<span class="font-medium">{pod.namespace}</span>
							</div>
							<div class="flex justify-between">
								<span class="text-muted-foreground">Status:</span>
								<Badge variant={pod.status === 'running' ? 'default' : 'secondary'}>
									{pod.status}
								</Badge>
							</div>
							<div class="flex justify-between">
								<span class="text-muted-foreground">Age:</span>
								<span class="font-medium">{pod.metadata?.age || 'N/A'}</span>
							</div>
							<div class="flex justify-between">
								<span class="text-muted-foreground">Restarts:</span>
								<span class="font-medium">{pod.metadata?.restarts || 0}</span>
							</div>
							{#if pod.metadata?.ip}
								<div class="flex justify-between">
									<span class="text-muted-foreground">Pod IP:</span>
									<span class="font-medium">{pod.metadata.ip}</span>
								</div>
							{/if}
							{#if pod.metadata?.node}
								<div class="flex justify-between">
									<span class="text-muted-foreground">Node:</span>
									<span class="font-medium">{pod.metadata.node}</span>
								</div>
							{/if}
						</CardContent>
					</Card>
					
					<Card>
						<CardHeader>
							<CardTitle>Containers</CardTitle>
						</CardHeader>
						<CardContent>
							{#if containers().length > 0}
								<div class="space-y-3">
									{#each containers() as container}
										<div class="border rounded-lg p-3">
											<div class="flex items-center justify-between mb-2">
												<span class="font-medium">{container.name || container}</span>
												{#if container.ready !== undefined}
													<Badge variant={container.ready ? 'default' : 'secondary'}>
														{container.ready ? 'Ready' : 'Not Ready'}
													</Badge>
												{/if}
											</div>
											{#if container.image}
												<p class="text-sm text-muted-foreground">Image: {container.image}</p>
											{/if}
											{#if container.restartCount !== undefined}
												<p class="text-sm text-muted-foreground">Restarts: {container.restartCount}</p>
											{/if}
										</div>
									{/each}
								</div>
							{:else}
								<p class="text-muted-foreground">No container information available</p>
							{/if}
						</CardContent>
					</Card>
				</div>
				
				{#if pod.metadata && Object.keys(pod.metadata).length > 0}
					<Card>
						<CardHeader>
							<CardTitle>Metadata</CardTitle>
						</CardHeader>
						<CardContent>
							<pre class="text-xs bg-muted p-4 rounded-lg overflow-auto">{JSON.stringify(pod.metadata, null, 2)}</pre>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>
			
			<!-- Logs Tab -->
			<TabsContent value="logs" class="space-y-4">
				<!-- Log Search Panel -->
				<LogSearchPanel
					bind:searchQuery={logSearchQuery}
					bind:selectedContainer={selectedLogContainer}
					bind:selectedSeverity={selectedLogSeverity}
					bind:tailLines={logTailLines}
					containers={logContainers}
					onSearchChange={handleLogSearchChange}
					onContainerChange={handleLogContainerChange}
					onSeverityChange={handleLogSeverityChange}
					onTailLinesChange={handleLogTailLinesChange}
					onClearFilters={handleClearLogFilters}
				/>
				
				<!-- Logs Display -->
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>
								Pod Logs
								{#if parsedLogs.length > 0}
									<span class="text-sm font-normal text-muted-foreground ml-2">
										({parsedLogs.length} entries)
									</span>
								{/if}
							</CardTitle>
							<div class="flex items-center gap-2">
								<!-- View Mode Toggle -->
								<div class="flex items-center gap-1 border rounded-md">
									<button
										class="px-2 py-1 text-xs {logViewMode === 'detailed' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
										onclick={(e) => { e.preventDefault(); e.stopPropagation(); logViewMode = 'detailed'; }}
										type="button"
									>
										Detailed
									</button>
									<button
										class="px-2 py-1 text-xs {logViewMode === 'compact' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
										onclick={(e) => { e.preventDefault(); e.stopPropagation(); logViewMode = 'compact'; }}
										type="button"
									>
										Compact
									</button>
									<button
										class="px-2 py-1 text-xs {logViewMode === 'raw' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}"
										onclick={(e) => { e.preventDefault(); e.stopPropagation(); logViewMode = 'raw'; }}
										type="button"
									>
										Raw
									</button>
								</div>
								<Button variant="outline" size="sm" onclick={() => loadLogs()} disabled={logsLoading}>
									<RefreshCw class="mr-2 h-4 w-4 {logsLoading ? 'animate-spin' : ''}" />
									Refresh
								</Button>
								{#if logs}
									<Button variant="outline" size="sm" onclick={() => {
										const blob = new Blob([logs], { type: 'text/plain' });
										const url = URL.createObjectURL(blob);
										const a = document.createElement('a');
										a.href = url;
										a.download = `${pod.name}-logs.txt`;
										a.click();
										URL.revokeObjectURL(url);
									}}>
										<Download class="mr-2 h-4 w-4" />
										Download
									</Button>
								{/if}
							</div>
						</div>
					</CardHeader>
					<CardContent>
						{#if logsLoading}
							<div class="flex items-center justify-center py-8">
								<Loading text="Loading logs..." />
							</div>
						{:else if logsError}
							<div class="text-destructive">{logsError}</div>
						{:else if parsedLogs.length > 0}
							<LogsDisplay
								logs={parsedLogs}
								searchQuery={logSearchQuery}
								severityFilter={selectedLogSeverity}
								viewMode={logViewMode}
								onFilterBySeverity={handleLogSeverityChange}
							/>
						{:else if logs}
							<div class="text-muted-foreground text-center py-8">
								<p>No logs available to parse</p>
								<p class="text-xs mt-2">Raw logs: {logs.split('\n').length} lines</p>
							</div>
						{:else}
							<p class="text-muted-foreground">No logs available</p>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- YAML Tab -->
			<TabsContent value="yaml" class="space-y-4">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Pod YAML</CardTitle>
							<div class="flex items-center gap-2">
								<Button variant="outline" size="sm" onclick={loadYAML} disabled={yamlLoading}>
									<RefreshCw class="mr-2 h-4 w-4 {yamlLoading ? 'animate-spin' : ''}" />
									Refresh
								</Button>
								{#if yaml}
									<Button variant="outline" size="sm" onclick={() => {
										const blob = new Blob([yaml], { type: 'text/yaml' });
										const url = URL.createObjectURL(blob);
										const a = document.createElement('a');
										a.href = url;
										a.download = `${pod.name}.yaml`;
										a.click();
										URL.revokeObjectURL(url);
									}}>
										<Download class="mr-2 h-4 w-4" />
										Download
									</Button>
								{/if}
							</div>
						</div>
					</CardHeader>
					<CardContent>
						{#if yamlLoading}
							<div class="flex items-center justify-center py-8">
								<Loading text="Loading YAML..." />
							</div>
						{:else if yamlError}
							<div class="text-destructive">{yamlError}</div>
						{:else if yaml}
							<pre class="text-xs bg-muted p-4 rounded-lg overflow-auto max-h-[600px] font-mono whitespace-pre-wrap">{yaml}</pre>
						{:else}
							<p class="text-muted-foreground">No YAML available</p>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- Port Forwards Tab -->
			<TabsContent value="port-forwards" class="space-y-4">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Active Port Forwards</CardTitle>
							<Button variant="outline" size="sm" onclick={() => showPortForwardDialog = true}>
								<Network class="mr-2 h-4 w-4" />
								New Port Forward
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						{#if loadingPortForwards}
							<div class="flex items-center justify-center py-8">
								<Loading text="Loading port forwards..." />
							</div>
						{:else if activePortForwards.length === 0}
							<p class="text-muted-foreground text-center py-8">
								No active port forwards for this pod.
							</p>
						{:else}
							<div class="space-y-3">
								{#each activePortForwards as forward (forward.id)}
									<div class="border rounded-lg p-4 flex items-center justify-between">
										<div class="flex-1">
											<div class="flex items-center gap-2 mb-1">
												<Badge variant="default">Active</Badge>
												<span class="font-medium">{forward.url}</span>
											</div>
											<p class="text-sm text-muted-foreground">
												localhost:{forward.local_port} → {forward.pod_name}:{forward.remote_port}
											</p>
											<p class="text-xs text-muted-foreground mt-1">
												Started: {new Date(forward.created_at).toLocaleString()}
											</p>
										</div>
										<Button variant="destructive" size="sm" onclick={() => stopPortForward(forward.id)}>
											Stop
										</Button>
									</div>
								{/each}
							</div>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
			
			<!-- Exec Tab -->
			<TabsContent value="exec" class="space-y-4">
				<Card>
					<CardHeader>
						<CardTitle>Exec into Pod</CardTitle>
					</CardHeader>
					<CardContent>
						<p class="text-muted-foreground mb-4">
							Execute commands in a container within this pod. This will open a terminal session.
						</p>
						{#if containers().length > 0}
							<div class="space-y-4">
								{#each containers() as container}
									<div class="border rounded-lg p-4">
										<div class="flex items-center justify-between">
											<div>
												<p class="font-medium">{container.name || container}</p>
												{#if container.image}
													<p class="text-sm text-muted-foreground">{container.image}</p>
												{/if}
											</div>
											<Button onclick={() => {
												// Navigate to terminal with kubectl exec command
												const containerFlag = container.name ? `-c ${container.name}` : '';
												const execCommand = `kubectl exec -it ${pod.name} ${containerFlag} -n ${pod.namespace} -- sh`;
												goto(`/terminal?command=${encodeURIComponent(execCommand)}`);
											}}>
												<Terminal class="mr-2 h-4 w-4" />
												Exec
											</Button>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<p class="text-muted-foreground">No containers available</p>
						{/if}
					</CardContent>
				</Card>
			</TabsContent>
		</Tabs>
		
		<!-- Port Forward Dialog -->
		<Dialog bind:open={showPortForwardDialog}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Port Forward</DialogTitle>
					<DialogDescription>
						Forward a local port to a port in pod {pod?.name}
					</DialogDescription>
				</DialogHeader>
				<div class="space-y-4 py-4">
					<div class="space-y-2">
						<Label for="local-port">Local Port</Label>
						<Input
							id="local-port"
							type="number"
							min="1"
							max="65535"
							bind:value={portForwardLocalPort}
							disabled={isPortForwarding}
						/>
					</div>
					<div class="space-y-2">
						<Label for="remote-port">Remote Port</Label>
						<Input
							id="remote-port"
							type="number"
							min="1"
							max="65535"
							bind:value={portForwardRemotePort}
							disabled={isPortForwarding}
						/>
					</div>
					{#if portForwardMessage}
						<div class="rounded-md bg-muted p-3">
							<p class="text-sm">{portForwardMessage}</p>
						</div>
					{/if}
				</div>
				<DialogFooter>
					<Button variant="outline" onclick={() => { showPortForwardDialog = false; portForwardMessage = null; }} disabled={isPortForwarding}>
						Cancel
					</Button>
					<Button onclick={async () => {
						if (!pod) return;
						try {
							isPortForwarding = true;
							portForwardMessage = null;
							const result = await invoke<PortForwardInfo>('k8s_start_port_forward', {
								namespace: pod.namespace,
								podName: pod.name,
								localPort: portForwardLocalPort,
								remotePort: portForwardRemotePort
							});
							portForwardMessage = `Port forward started: ${result.url}`;
							toastActions.success(`Port forward started: ${result.url}`);
							showPortForwardDialog = false;
							await loadPortForwards();
						} catch (error) {
							portForwardMessage = error instanceof Error ? error.message : 'Failed to start port forward';
							toastActions.error(portForwardMessage);
						} finally {
							isPortForwarding = false;
						}
					}} disabled={isPortForwarding}>
						{isPortForwarding ? 'Starting...' : 'Start Port Forward'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	{/if}
</div>

