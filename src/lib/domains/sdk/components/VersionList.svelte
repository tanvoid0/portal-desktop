<!--
	Version List - FlyEnv-style version management
	List of all versions (installed + available) with install/uninstall actions
-->

<script lang="ts">
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Progress } from '$lib/components/ui/progress';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Download, Trash2, Check, Clock, AlertCircle } from 'lucide-svelte';

	interface VersionInfo {
		version: string;
		installed: boolean;
		active: boolean;
		downloading: boolean;
		progress?: number;
		error?: string;
		lts?: boolean;
		releaseDate?: string;
		description?: string;
	}

	interface Props {
		versions: VersionInfo[];
		onInstall: (version: VersionInfo) => void;
		onUninstall: (version: VersionInfo) => void;
		onSetActive: (version: VersionInfo) => void;
		loading?: boolean;
	}

	let { versions, onInstall, onUninstall, onSetActive, loading = false }: Props = $props();

	// Derived state
	let installedVersions = $derived(
		versions.filter(v => v.installed)
	);
	
	let availableVersions = $derived(
		versions.filter(v => !v.installed)
	);

	let activeVersion = $derived(
		versions.find(v => v.active)
	);

	// Status helpers
	function getStatusBadge(version: VersionInfo) {
		if (version.error) {
			return { variant: 'destructive' as const, text: 'Error', icon: AlertCircle };
		}
		if (version.downloading) {
			return { variant: 'secondary' as const, text: 'Downloading', icon: Clock };
		}
		if (version.active) {
			return { variant: 'default' as const, text: 'Active', icon: Check };
		}
		if (version.installed) {
			return { variant: 'secondary' as const, text: 'Installed', icon: Check };
		}
		return { variant: 'outline' as const, text: 'Available', icon: Download };
	}

	function canInstall(version: VersionInfo) {
		return !version.installed && !version.downloading && !version.error;
	}

	function canUninstall(version: VersionInfo) {
		return version.installed && !version.active && !version.downloading;
	}

	function canSetActive(version: VersionInfo) {
		return version.installed && !version.active && !version.downloading;
	}

	// Event handlers
	function handleInstall(version: VersionInfo) {
		if (canInstall(version)) {
			onInstall(version);
		}
	}

	function handleUninstall(version: VersionInfo) {
		if (canUninstall(version)) {
			onUninstall(version);
		}
	}

	function handleSetActive(version: VersionInfo) {
		if (canSetActive(version)) {
			onSetActive(version);
		}
	}
</script>

<div class="space-y-6">
	<!-- Active Version Card -->
	{#if activeVersion}
		<Card class="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
			<CardHeader class="pb-3">
				<CardTitle class="text-lg flex items-center gap-2">
					<Check class="w-5 h-5 text-green-600" />
					Active Version
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex items-center justify-between">
					<div>
						<div class="font-mono text-xl font-bold">{activeVersion.version}</div>
						{#if activeVersion.lts}
							<Badge variant="secondary" class="mt-1">LTS</Badge>
						{/if}
					</div>
					<Badge variant="default" class="bg-green-600">
						Currently Active
					</Badge>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Version Management Tabs -->
	<Tabs value="installed" class="w-full">
		<TabsList class="grid w-full grid-cols-2">
			<TabsTrigger value="installed">
				Installed ({installedVersions.length})
			</TabsTrigger>
			<TabsTrigger value="available">
				Available ({availableVersions.length})
			</TabsTrigger>
		</TabsList>

		<TabsContent value="installed" class="space-y-4">
			{#if installedVersions.length === 0}
				<Card>
					<CardContent class="flex flex-col items-center justify-center py-8 text-center">
						<Download class="w-12 h-12 text-muted-foreground mb-4" />
						<h3 class="text-lg font-semibold mb-2">No Versions Installed</h3>
						<p class="text-muted-foreground">
							Install versions from the Available tab to get started.
						</p>
					</CardContent>
				</Card>
			{:else}
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-32">Version</TableHead>
							<TableHead class="w-32">Status</TableHead>
							<TableHead>Description</TableHead>
							<TableHead class="w-48">Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each installedVersions as version}
							<TableRow>
								<TableCell class="font-mono">
									<div class="flex items-center gap-2">
										{version.version}
										{#if version.lts}
											<Badge variant="outline" class="text-xs">LTS</Badge>
										{/if}
									</div>
								</TableCell>
								<TableCell>
									{#if version.downloading}
										<div class="space-y-2">
											<Badge variant="secondary" class="flex items-center gap-1 w-fit">
												<Clock class="w-3 h-3" />
												Downloading
											</Badge>
											{#if version.progress !== undefined}
												<Progress value={version.progress} class="h-1" />
											{/if}
										</div>
									{:else if version.error}
										<Badge variant="destructive" class="flex items-center gap-1 w-fit">
											<AlertCircle class="w-3 h-3" />
											Error
										</Badge>
									{:else}
										{@const status = getStatusBadge(version)}
										<Badge variant={status.variant} class="flex items-center gap-1 w-fit">
											<status.icon class="w-3 h-3" />
											{status.text}
										</Badge>
									{/if}
								</TableCell>
								<TableCell>
									<div class="text-sm text-muted-foreground">
										{version.description || 'No description available'}
									</div>
									{#if version.releaseDate}
										<div class="text-xs text-muted-foreground mt-1">
											Released: {new Date(version.releaseDate).toLocaleDateString()}
										</div>
									{/if}
								</TableCell>
								<TableCell>
									<div class="flex gap-2">
										{#if canSetActive(version)}
											<Button 
												size="sm" 
												onclick={() => handleSetActive(version)}
											>
												Set Active
											</Button>
										{/if}
										{#if canUninstall(version)}
											<Button 
												size="sm" 
												variant="destructive" 
												onclick={() => handleUninstall(version)}
											>
												<Trash2 class="w-4 h-4 mr-1" />
												Uninstall
											</Button>
										{/if}
									</div>
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			{/if}
		</TabsContent>

		<TabsContent value="available" class="space-y-4">
			{#if availableVersions.length === 0}
				<Card>
					<CardContent class="flex flex-col items-center justify-center py-8 text-center">
						<Check class="w-12 h-12 text-muted-foreground mb-4" />
						<h3 class="text-lg font-semibold mb-2">All Versions Installed</h3>
						<p class="text-muted-foreground">
							You have all available versions installed.
						</p>
					</CardContent>
				</Card>
			{:else}
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-32">Version</TableHead>
							<TableHead class="w-32">Status</TableHead>
							<TableHead>Description</TableHead>
							<TableHead class="w-32">Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each availableVersions as version}
							<TableRow>
								<TableCell class="font-mono">
									<div class="flex items-center gap-2">
										{version.version}
										{#if version.lts}
											<Badge variant="outline" class="text-xs">LTS</Badge>
										{/if}
									</div>
								</TableCell>
								<TableCell>
									{@const status = getStatusBadge(version)}
									<Badge variant={status.variant} class="flex items-center gap-1 w-fit">
										<status.icon class="w-3 h-3" />
										{status.text}
									</Badge>
								</TableCell>
								<TableCell>
									<div class="text-sm text-muted-foreground">
										{version.description || 'No description available'}
									</div>
									{#if version.releaseDate}
										<div class="text-xs text-muted-foreground mt-1">
											Released: {new Date(version.releaseDate).toLocaleDateString()}
										</div>
									{/if}
								</TableCell>
								<TableCell>
									{#if canInstall(version)}
										<Button 
											size="sm" 
											onclick={() => handleInstall(version)}
											disabled={loading}
										>
											<Download class="w-4 h-4 mr-1" />
											Install
										</Button>
									{/if}
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			{/if}
		</TabsContent>
	</Tabs>
</div>
