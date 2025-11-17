<!--
	SDK Sidebar - FlyEnv-style sidebar with language and database categories
	Shows all available SDKs with toggle switches and selection states
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Switch } from '$lib/components/ui/switch';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Separator } from '$lib/components/ui/separator';
	import { Settings, Database, Code, Globe, Container, Package, Download } from '@lucide/svelte';
	import Devicon from '$lib/components/ui/devicon.svelte';
	import { logger } from '$lib/domains/shared';
	import { sdkService } from '$lib/domains/sdk/services/sdkService';
	import { sdkManagers, installedSDKs } from '$lib/domains/sdk/stores/sdkStore';
	import type { SDKManagerInfo } from '$lib/domains/sdk/types';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	interface Props {
		selectedSDK?: string;
		selectedView?: string;
		onSDKSelect?: (sdk: SDKItem) => void;
		onViewSelect?: (view: string) => void;
	}

	const { selectedSDK, selectedView = 'overview', onSDKSelect, onViewSelect }: Props = $props();

	// Get real data from stores
	let managers = $derived($sdkManagers);
	let sdks = $derived($installedSDKs);
	let currentPath = $derived($page.url.pathname);

	// Navigation items - using actual routes
	const navigationItems = [
		{
			id: '/sdk',
			name: 'Overview',
			icon: 'lucide:layout-dashboard',
			description: 'SDK overview and statistics'
		},
		{
			id: '/sdk/managers',
			name: 'SDK Managers',
			icon: 'lucide:settings',
			description: 'Manage SDK version managers'
		},
		{
			id: '/sdk/installations',
			name: 'Installations',
			icon: 'lucide:download',
			description: 'View installed SDKs and versions'
		}
	];

	interface SDKItem {
		id: string;
		name: string;
		displayName: string;
		icon: string;
		category: string;
		installed: boolean;
		enabled: boolean;
		version?: string;
		description?: string;
		hasToggle?: boolean;
	}

	// Convert real managers and SDKs to SDKItem format
	let languageSDKs = $derived(() => {
		return managers
			.filter((manager: SDKManagerInfo) => {
				const category = manager.category || '';
				return category === 'language';
			})
			.map((manager: SDKManagerInfo) => ({
				id: manager.type || manager.name,
				name: manager.type || manager.name,
				displayName: manager.display_name || manager.name,
				icon: getSDKIcon(manager.type || manager.name),
				category: 'language',
				installed: manager.installed === true || manager.installed === 'true',
				enabled: manager.installed === true || manager.installed === 'true',
				version: manager.version || 'Not installed',
				description: manager.description,
				hasToggle: true
			}));
	});

	let databaseSDKs = $derived(() => {
		return managers
			.filter((manager: SDKManagerInfo) => {
				const category = manager.category || '';
				return category === 'database';
			})
			.map((manager: SDKManagerInfo) => ({
				id: manager.type || manager.name,
				name: manager.type || manager.name,
				displayName: manager.display_name || manager.name,
				icon: getSDKIcon(manager.type || manager.name),
				category: 'database',
				installed: manager.installed === true || manager.installed === 'true',
				enabled: manager.installed === true || manager.installed === 'true',
				version: manager.version,
				description: manager.description,
				hasToggle: true
			}));
	});

	let webServerSDKs = $derived(() => {
		return managers
			.filter((manager: SDKManagerInfo) => {
				const category = manager.category || '';
				return category === 'web';
			})
			.map((manager: SDKManagerInfo) => ({
				id: manager.type || manager.name,
				name: manager.type || manager.name,
				displayName: manager.display_name || manager.name,
				icon: getSDKIcon(manager.type || manager.name),
				category: 'web',
				installed: manager.installed === true || manager.installed === 'true',
				enabled: manager.installed === true || manager.installed === 'true',
				version: manager.version,
				description: manager.description,
				hasToggle: true
			}));
	});

	let containerSDKs = $derived(() => {
		return managers
			.filter((manager: SDKManagerInfo) => {
				const category = manager.category || '';
				return category === 'container';
			})
			.map((manager: SDKManagerInfo) => ({
				id: manager.type || manager.name,
				name: manager.type || manager.name,
				displayName: manager.display_name || manager.name,
				icon: getSDKIcon(manager.type || manager.name),
				category: 'container',
				installed: manager.installed === true || manager.installed === 'true',
				enabled: manager.installed === true || manager.installed === 'true',
				version: manager.version,
				description: manager.description,
				hasToggle: true
			}));
	});

	// SDK Managers (tools that manage multiple SDKs)
	let sdkManagersList = $derived(() => {
		return managers
			.filter((manager: SDKManagerInfo) => {
				const category = manager.category || '';
				return category === 'manager';
			})
			.map((manager: SDKManagerInfo) => ({
				id: manager.name,
				name: manager.name,
				displayName: manager.display_name || manager.name,
				icon: getSDKIcon(manager.name),
				category: 'manager',
				installed: manager.installed === true || manager.installed === 'true',
				enabled: manager.installed === true || manager.installed === 'true',
				version: manager.version,
				description: manager.description,
				hasToggle: true
			}));
	});

	// Reactive state
	let allSDKs = $derived(() => [
		...languageSDKs(),
		...databaseSDKs(),
		...webServerSDKs(),
		...containerSDKs()
	]);

	let selectedItem = $state<string | null>(selectedSDK || null);

	// Category icons
	const categoryIcons = {
		language: Code,
		database: Database,
		web: Globe,
		container: Container,
		package: Package
	};

	// Map SDK types to their route paths
	function getSDKRoute(sdkId: string, category: string): string {
		// Normalize SDK ID for routing
		const normalizedId = sdkId.toLowerCase().trim();
		
		// SDK Managers go to /sdk/managers/[name]
		if (category === 'manager') {
			return `/sdk/managers/${normalizedId}`;
		}
		
		// Map common SDK IDs to their route paths
		const routeMap: Record<string, string> = {
			'node': '/sdk/nodejs',
			'nodejs': '/sdk/nodejs',
			'python': '/sdk/python',
			'java': '/sdk/java',
			'rust': '/sdk/rust',
			'go': '/sdk/go',
			'php': '/sdk/php',
			'ruby': '/sdk/ruby'
		};
		
		// Check if we have a specific route for this SDK
		if (routeMap[normalizedId]) {
			return routeMap[normalizedId];
		}
		
		// For database SDKs, check if there's a database route
		if (category === 'database') {
			return `/sdk/database/${normalizedId}`;
		}
		
		// Default: try the SDK ID as a route (fallback)
		return `/sdk/${normalizedId}`;
	}

	function handleSDKClick(sdk: SDKItem) {
		selectedItem = sdk.id;
		onSDKSelect?.(sdk);
		
		// Get the appropriate route for this SDK
		const route = getSDKRoute(sdk.id, sdk.category);
		
		// Debug: Log the SDK item and route
		console.log('SDK clicked:', {
			id: sdk.id,
			name: sdk.name,
			displayName: sdk.displayName,
			category: sdk.category,
			route: route
		});
		
		// Navigate to the appropriate route
		console.log('Navigating to:', route);
		goto(route);
		
		logger.info('SDK selected', { 
			context: 'SDKSidebar', 
			data: { sdkId: sdk.id, sdkName: sdk.displayName, category: sdk.category, route } 
		});
	}


	function getCategoryIcon(category: string) {
		return categoryIcons[category as keyof typeof categoryIcons] || Code;
	}

	function getCategoryName(category: string) {
		const names: Record<string, string> = {
			language: 'Language & Runtime',
			database: 'Database Server',
			web: 'Web Server',
			container: 'Container Platform',
			package: 'Package Manager'
		};
		return names[category] || category;
	}

	function getSDKIcon(sdkType: string): string {
		const iconMap: Record<string, string> = {
			// Language & Runtime
			'java': 'devicon-java-plain',
			'node': 'devicon-nodejs-plain',
			'python': 'devicon-python-plain',
			'rust': 'devicon-rust-plain',
			'go': 'devicon-go-plain',
			'php': 'devicon-php-plain',
			'ruby': 'devicon-ruby-plain',
			'bun': 'devicon-bun-plain',
			'deno': 'devicon-deno-plain',
			'gradle': 'devicon-gradle-plain',
			'kotlin': 'devicon-kotlin-plain',
			'scala': 'devicon-scala-plain',
			'erlang': 'devicon-erlang-plain',
			'perl': 'devicon-perl-plain',
			
			// Database
			'mysql': 'devicon-mysql-plain',
			'postgresql': 'devicon-postgresql-plain',
			'mongodb': 'devicon-mongodb-plain',
			'mariadb': 'devicon-mariadb-plain',
			
			// Web Server
			'nginx': 'devicon-nginx-original',
			'apache': 'devicon-apache-plain',
			'caddy': 'devicon-caddy-plain',
			
			// Container
			'docker': 'devicon-docker-plain',
			'kubernetes': 'devicon-kubernetes-plain',
			'podman': 'devicon-podman-plain',
			
			// Package Managers
			'npm': 'devicon-npm-original-wordmark',
			'yarn': 'devicon-yarn-plain',
			'pip': 'devicon-python-plain',
			'cargo': 'devicon-rust-plain',
			'composer': 'devicon-composer-plain',
			'gem': 'devicon-ruby-plain',
			
			// SDK Managers
			'nvm': 'devicon-nodejs-plain',
			'pyenv': 'devicon-python-plain',
			'rustup': 'devicon-rust-plain',
			'sdkman': 'devicon-sdkman-plain',
			'goenv': 'devicon-go-plain',
			'rbenv': 'devicon-ruby-plain',
			'phpenv': 'devicon-php-plain'
		};
		
		return iconMap[sdkType.toLowerCase()] || 'devicon-devicon-plain';
	}

	onMount(() => {
		logger.info('SDK Sidebar mounted', { context: 'SDKSidebar' });
		logger.info('Managers data', { context: 'SDKSidebar', data: { managers: managers.length, sdks: sdks.length } });
		logger.info('Language SDKs', { context: 'SDKSidebar', data: { count: languageSDKs().length } });
		logger.info('SDK Managers', { context: 'SDKSidebar', data: { count: sdkManagersList().length } });
	});
</script>

<div class="w-80 h-full bg-background border-r border-border flex flex-col">
	<!-- Header -->
	<div class="p-4 border-b border-border">
		<h2 class="text-lg font-semibold">SDK Manager</h2>
		<p class="text-sm text-muted-foreground">Manage your development environment</p>
	</div>


	<!-- Sidebar Content -->
	<ScrollArea class="flex-1">
		<div class="p-4 space-y-6">
			<!-- Navigation Section -->
			<div class="space-y-3">
				<h3 class="font-medium text-sm text-muted-foreground">Navigation</h3>
				<div class="space-y-1">
					{#each navigationItems as item}
						<button
							class="w-full flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors text-left
								{currentPath === item.id ? 'bg-muted' : ''}"
							onclick={() => goto(item.id)}
						>
							<div class="h-4 w-4 flex items-center justify-center">
								{#if item.icon === 'lucide:layout-dashboard'}
									<Settings class="h-4 w-4" />
								{:else if item.icon === 'lucide:settings'}
									<Settings class="h-4 w-4" />
								{:else if item.icon === 'lucide:download'}
									<Download class="h-4 w-4" />
								{/if}
							</div>
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{item.name}</div>
								<div class="text-xs text-muted-foreground truncate">{item.description}</div>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<Separator />


			<!-- SDK Managers Section -->
			{#if sdkManagersList().length > 0}
				<div class="space-y-3">
					<div class="flex items-center gap-2">
						{#if true}
							{@const ManagerIcon = getCategoryIcon('package')}
							<ManagerIcon class="h-4 w-4" />
						{/if}
						<h3 class="font-medium text-sm">SDK Managers</h3>
					</div>
					<div class="space-y-1">
						{#each sdkManagersList() as manager}
							<div 
								class="flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors
									{selectedItem === manager.id ? 'bg-muted' : ''}"
								role="button"
								tabindex="0"
								onclick={() => handleSDKClick(manager)}
								onkeydown={(e) => {
									if (e.key === 'Enter' || e.key === ' ') {
										e.preventDefault();
										handleSDKClick(manager);
									}
								}}
							>
								<Devicon icon={manager.icon} size="sm" />
								<div class="flex-1 min-w-0">
									<div class="text-sm font-medium truncate">{manager.displayName}</div>
									{#if manager.version}
										<div class="text-xs text-muted-foreground">{manager.version}</div>
									{/if}
								</div>
								{#if manager.hasToggle}
								<Switch 
									checked={manager.installed}
									disabled={true}
									onclick={(e) => e.stopPropagation()}
								/>
								{/if}
							</div>
						{/each}
					</div>
				</div>
				<Separator />
			{/if}

			<!-- Language & Runtime Section -->
			{#if languageSDKs().length > 0}
				<div class="space-y-3">
					<div class="flex items-center gap-2">
						{#if true}
							{@const LanguageIcon = getCategoryIcon('language')}
							<LanguageIcon class="h-4 w-4" />
						{/if}
						<h3 class="font-medium text-sm">{getCategoryName('language')}</h3>
					</div>
					<div class="space-y-1">
						{#each languageSDKs() as sdk}
							<div 
								class="flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors
									{selectedItem === sdk.id ? 'bg-muted' : ''}"
								role="button"
								tabindex="0"
								onclick={() => handleSDKClick(sdk)}
								onkeydown={(e) => {
									if (e.key === 'Enter' || e.key === ' ') {
										e.preventDefault();
										handleSDKClick(sdk);
									}
								}}
							>
							<Devicon icon={sdk.icon} size="sm" />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{/if}
							</div>
							{#if sdk.hasToggle}
								<Switch 
									checked={sdk.installed}
									disabled={true}
									onclick={(e) => e.stopPropagation()}
								/>
							{/if}
						</div>
					{/each}
					</div>
				</div>
			{/if}

			<Separator />

			<!-- Database Server Section -->
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					{#if true}
						{@const DatabaseIcon = getCategoryIcon('database')}
						<DatabaseIcon class="h-4 w-4" />
					{/if}
					<h3 class="font-medium text-sm">{getCategoryName('database')}</h3>
				</div>
				<div class="space-y-1">
					{#each databaseSDKs() as sdk}
						<div 
							class="flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors
								{selectedItem === sdk.id ? 'bg-muted' : ''}"
							role="button"
							tabindex="0"
							onclick={() => handleSDKClick(sdk)}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									handleSDKClick(sdk);
								}
							}}
						>
							<Devicon icon={sdk.icon} size="sm" />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{/if}
							</div>
							{#if sdk.hasToggle}
								<Switch 
									checked={sdk.installed}
									disabled={true}
									onclick={(e) => e.stopPropagation()}
								/>
							{:else}
								<Settings class="h-4 w-4 text-muted-foreground" />
							{/if}
						</div>
					{/each}
				</div>
			</div>

			<Separator />

			<!-- Web Server Section -->
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					{#if true}
						{@const WebIcon = getCategoryIcon('web')}
						<WebIcon class="h-4 w-4" />
					{/if}
					<h3 class="font-medium text-sm">{getCategoryName('web')}</h3>
				</div>
				<div class="space-y-1">
					{#each webServerSDKs() as sdk}
						<div 
							class="flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors
								{selectedItem === sdk.id ? 'bg-muted' : ''}"
							role="button"
							tabindex="0"
							onclick={() => handleSDKClick(sdk)}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									handleSDKClick(sdk);
								}
							}}
						>
							<Devicon icon={sdk.icon} size="sm" />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{/if}
							</div>
							{#if sdk.hasToggle}
								<Switch 
									checked={sdk.installed}
									disabled={true}
									onclick={(e) => e.stopPropagation()}
								/>
							{/if}
						</div>
					{/each}
				</div>
			</div>

			<Separator />

			<!-- Container Platform Section -->
			<div class="space-y-3">
				<div class="flex items-center gap-2">
					{#if true}
						{@const ContainerIcon = getCategoryIcon('container')}
						<ContainerIcon class="h-4 w-4" />
					{/if}
					<h3 class="font-medium text-sm">{getCategoryName('container')}</h3>
				</div>
				<div class="space-y-1">
					{#each containerSDKs() as sdk}
						<div 
							class="flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 cursor-pointer transition-colors
								{selectedItem === sdk.id ? 'bg-muted' : ''}"
							role="button"
							tabindex="0"
							onclick={() => handleSDKClick(sdk)}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									handleSDKClick(sdk);
								}
							}}
						>
							<Devicon icon={sdk.icon} size="sm" />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{/if}
							</div>
							{#if sdk.hasToggle}
								<Switch 
									checked={sdk.installed}
									disabled={true}
									onclick={(e) => e.stopPropagation()}
								/>
							{/if}
						</div>
					{/each}
				</div>
			</div>

			<!-- No Data Fallback -->
			{#if sdkManagersList().length === 0 && languageSDKs().length === 0 && databaseSDKs().length === 0 && webServerSDKs().length === 0 && containerSDKs().length === 0}
				<div class="text-center text-muted-foreground py-8">
					<div class="text-4xl mb-4">🔍</div>
					<h3 class="font-medium mb-2">No SDKs Detected</h3>
					<p class="text-sm">
						No SDK managers or SDKs were found on your system. 
						Install SDK managers like NVM, Pyenv, or SDKMAN to get started.
					</p>
				</div>
			{/if}
		</div>
	</ScrollArea>

	<!-- Footer -->
	<div class="p-4 border-t border-border">
		<div class="text-xs text-muted-foreground text-center space-y-1">
			<div>
				{sdkManagersList().filter(m => m.installed).length} of {sdkManagersList().length} SDK Managers
			</div>
			<div>
				{allSDKs().filter(sdk => sdk.installed).length} of {allSDKs().length} SDKs
			</div>
		</div>
	</div>
</div>
