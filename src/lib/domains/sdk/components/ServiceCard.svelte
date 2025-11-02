<!--
	Service Card - FlyEnv-style service management card
	Individual service card with start/stop/status controls
-->

<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import Select from '$lib/components/ui/select.svelte';
	import { Progress } from '$lib/components/ui/progress';
	import { Power, Settings, Logs, ExternalLink, RefreshCw, Activity, X } from 'lucide-svelte';
	import ServiceLogViewer from './ServiceLogViewer.svelte';
	import ServiceConfigEditor from './ServiceConfigEditor.svelte';
	import ServiceHealthIndicator from './ServiceHealthIndicator.svelte';

	interface ServiceInfo {
		id: string;
		name: string;
		description: string;
		version: string;
		status: 'running' | 'stopped' | 'error' | 'starting' | 'stopping';
		port?: number;
		pid?: number;
		progress?: number;
	}

	interface Props {
		service: ServiceInfo;
		availableVersions: string[];
		onToggle: (service: ServiceInfo) => void;
		onVersionChange: (service: ServiceInfo, version: string) => void;
		onConfigure: (service: ServiceInfo) => void;
		onViewLogs: (service: ServiceInfo) => void;
		onOpenUrl: (service: ServiceInfo) => void;
	}

	let { 
		service, 
		availableVersions, 
		onToggle, 
		onVersionChange, 
		onConfigure, 
		onViewLogs, 
		onOpenUrl 
	}: Props = $props();

	// Modal states
	let showLogs = $state(false);
	let showConfig = $state(false);
	let showHealth = $state(false);

	// Derived state
	let statusColor = $derived(() => {
		switch (service.status) {
			case 'running': return 'text-green-600';
			case 'stopped': return 'text-gray-500';
			case 'error': return 'text-red-600';
			case 'starting': return 'text-yellow-600';
			case 'stopping': return 'text-orange-600';
			default: return 'text-gray-500';
		}
	});

	let statusText = $derived(() => {
		switch (service.status) {
			case 'running': return 'Running';
			case 'stopped': return 'Stopped';
			case 'error': return 'Error';
			case 'starting': return 'Starting...';
			case 'stopping': return 'Stopping...';
			default: return 'Unknown';
		}
	});

	let canToggle = $derived(() => 
		service.status === 'running' || service.status === 'stopped'
	);

	let buttonText = $derived(() => {
		switch (service.status) {
			case 'running': return 'Stop';
			case 'stopped': return 'Start';
			case 'starting': return 'Starting...';
			case 'stopping': return 'Stopping...';
			default: return 'Start';
		}
	});

	let buttonVariant = $derived(() => {
		switch (service.status) {
			case 'running': return 'destructive';
			case 'stopped': return 'default';
			default: return 'secondary';
		}
	});

	// Event handlers
	function handleToggle() {
		if (canToggle()) {
			onToggle(service);
		}
	}

	function handleVersionChange(version: string) {
		onVersionChange(service, version);
	}

	function handleConfigure() {
		showConfig = true;
		onConfigure(service);
	}

	function handleViewLogs() {
		showLogs = true;
		onViewLogs(service);
	}

	function handleOpenUrl() {
		onOpenUrl(service);
	}

	function handleShowHealth() {
		showHealth = true;
	}
</script>

<Card class="w-full hover:shadow-md transition-shadow">
	<CardHeader class="pb-3">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
					<Power class="w-5 h-5 text-primary" />
				</div>
				<div>
					<CardTitle class="text-lg">{service.name}</CardTitle>
					<p class="text-sm text-muted-foreground">{service.description}</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Badge 
					variant={service.status === 'running' ? 'default' : 'secondary'} 
					class="capitalize {statusColor()}"
				>
					{statusText()}
				</Badge>
				{#if service.port}
					<Badge variant="outline" class="text-xs">
						Port {service.port}
					</Badge>
				{/if}
			</div>
		</div>
	</CardHeader>

	<CardContent class="space-y-4">
		<!-- Version Selector -->
		<div class="flex items-center gap-3">
			<label for="version-select" class="text-sm font-medium w-16">Version:</label>
			<Select 
				options={availableVersions}
				defaultValue={service.version}
				onSelect={handleVersionChange}
				disabled={service.status === 'running' || service.status === 'starting'}
				placeholder="Select version"
				class="flex-1"
			/>
		</div>

		<!-- Progress Bar (for starting/stopping) -->
		{#if service.status === 'starting' || service.status === 'stopping'}
			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span>{statusText()}</span>
					<span>{service.progress || 0}%</span>
				</div>
				<Progress value={service.progress || 0} class="h-2" />
			</div>
		{/if}

		<!-- Service Info -->
		{#if service.status === 'running'}
			<div class="grid grid-cols-2 gap-4 text-sm">
				{#if service.port}
					<div>
						<span class="text-muted-foreground">Port:</span>
						<span class="ml-2 font-mono">{service.port}</span>
					</div>
				{/if}
				{#if service.pid}
					<div>
						<span class="text-muted-foreground">PID:</span>
						<span class="ml-2 font-mono">{service.pid}</span>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Action Buttons -->
		<div class="flex items-center gap-2 pt-2">
			<Button 
				onclick={handleToggle}
				disabled={!canToggle()}
				variant={buttonVariant()}
				size="sm"
				class="flex-1"
			>
				<Power class="w-4 h-4 mr-2" />
				{buttonText()}
			</Button>
			
			<Button 
				variant="outline" 
				size="sm" 
				onclick={handleConfigure}
				title="Configure Service"
			>
				<Settings class="w-4 h-4" />
			</Button>
			
			<Button 
				variant="outline" 
				size="sm" 
				onclick={handleViewLogs}
				title="View Logs"
			>
				<Logs class="w-4 h-4" />
			</Button>

			{#if service.status === 'running'}
				<Button 
					variant="outline" 
					size="sm" 
					onclick={handleShowHealth}
					title="Health Status"
				>
					<Activity class="w-4 h-4" />
				</Button>
			{/if}
			
			{#if service.status === 'running' && service.port}
				<Button 
					variant="outline" 
					size="sm" 
					onclick={handleOpenUrl}
					title="Open in Browser"
				>
					<ExternalLink class="w-4 h-4" />
				</Button>
			{/if}
		</div>
	</CardContent>
</Card>

<!-- Modals -->
<ServiceLogViewer 
	serviceId={service.id}
	serviceName={service.name}
	isOpen={showLogs}
	onClose={() => showLogs = false}
/>

<ServiceConfigEditor 
	serviceId={service.id}
	serviceName={service.name}
	isOpen={showConfig}
	onClose={() => showConfig = false}
/>

{#if showHealth}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
		<div class="bg-background rounded-lg shadow-lg max-w-2xl w-full mx-4">
			<div class="p-6">
				<div class="flex items-center justify-between mb-4">
					<h3 class="text-lg font-semibold">Health Status: {service.name}</h3>
					<Button variant="ghost" size="sm" onclick={() => showHealth = false}>
						<X class="w-4 h-4" />
					</Button>
				</div>
				<ServiceHealthIndicator 
					serviceId={service.id}
					serviceName={service.name}
					showDetails={true}
					refreshInterval={10}
				/>
			</div>
		</div>
	</div>
{/if}
