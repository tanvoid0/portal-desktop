<!--
	SDK Sidebar - FlyEnv-style sidebar with language and database categories
	Shows all available SDKs with toggle switches and selection states
-->

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Separator } from '$lib/components/ui/separator';
	import { Settings, Database, Code, Globe, Container, Package, Download, CheckCircle, XCircle, ArrowLeft } from '@lucide/svelte';
	import Devicon from '$lib/components/ui/devicon.svelte';
	import { logger } from '$lib/domains/shared';
	import { sdkConfigService, type ProcessedSDKConfig } from '$lib/domains/sdk/services/sdkConfigService';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { useSidebar, MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';

	interface Props {
		selectedSDK?: string;
		selectedView?: string;
		onSDKSelect?: (sdk: SDKItem) => void;
		onViewSelect?: (view: string) => void;
	}

	const { selectedSDK, selectedView = 'overview', onSDKSelect, onViewSelect }: Props = $props();

	// State
	let sdkConfigs = $state<ProcessedSDKConfig[]>([]);
	let loading = $state(true);
	let currentPath = $derived($page.url.pathname);
	const sidebar = useSidebar();

	// Load SDK configs
	$effect(() => {
		loadSDKConfigs();
	});

	async function loadSDKConfigs() {
		try {
			loading = true;
			const configs = await sdkConfigService.getAllSDKConfigs();
			sdkConfigs = configs;
			logger.info('SDK configs loaded', { 
				context: 'SDKSidebar', 
				data: { count: configs.length } 
			});
		} catch (error) {
			logger.error('Failed to load SDK configs', {
				context: 'SDKSidebar',
				error
			});
		} finally {
			loading = false;
		}
	}

	// Navigation items - using actual routes
	const navigationItems = [
		{
			id: '/sdk',
			name: 'Overview',
			icon: 'lucide:layout-dashboard',
			description: 'SDK overview and statistics'
		},
		{
			id: '/sdk/manager',
			name: 'SDK Managers',
			icon: 'lucide:settings',
			description: 'Manage SDK version managers'
		},
		{
			id: '/sdk/software-installer',
			name: 'Software Installer',
			icon: 'lucide:package',
			description: 'Install and manage software via package managers'
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
		port?: number | null;
		isRunning?: boolean;
	}

	// Convert SDK configs to SDKItem format
	let languageSDKs = $derived(() => {
		return sdkConfigs
			.filter((config) => config.category === 'language')
			.map((config) => {
				// Get version, preferring SDK version over manager version
				const rawVersion = config.sdk_version || config.sdk_managers.find(m => m.installed)?.version || null;
				// Format version (remove 'v' prefix if present, but keep it clean)
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : null;
				
				return {
					id: config.id,
					name: config.name,
					displayName: config.display_name,
					icon: config.icon,
					category: 'language',
					installed: config.sdk_installed || config.sdk_managers.some(m => m.installed) || false,
					enabled: true,
					version: version || undefined,
					description: config.description,
					hasToggle: true
				};
			});
	});

	let databaseSDKs = $derived(() => {
		return sdkConfigs
			.filter((config) => config.category === 'database')
			.map((config) => {
				// Get version, preferring SDK version
				const rawVersion = config.sdk_version || null;
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : null;
				
				return {
					id: config.id,
					name: config.name,
					displayName: config.display_name,
					icon: config.icon,
					category: 'database',
					installed: config.sdk_installed || false,
					enabled: true,
					version: version || undefined,
					description: config.description,
					hasToggle: true,
					port: config.service_port ?? null,
					isRunning: config.service_running ?? false
				} as SDKItem;
			});
	});

	let webServerSDKs = $derived(() => {
		return sdkConfigs
			.filter((config) => config.category === 'server')
			.map((config) => {
				// Get version, preferring SDK version
				const rawVersion = config.sdk_version || null;
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : null;
				
				return {
					id: config.id,
					name: config.name,
					displayName: config.display_name,
					icon: config.icon,
					category: 'server',
					installed: config.sdk_installed || false,
					enabled: true,
					version: version || undefined,
					description: config.description,
					hasToggle: true
				} as SDKItem;
			});
	});

	let containerSDKs = $derived(() => {
		return sdkConfigs
			.filter((config) => config.category === 'container')
			.map((config) => {
				// Get version, preferring SDK version
				const rawVersion = config.sdk_version || null;
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : null;
				
				return {
					id: config.id,
					name: config.name,
					displayName: config.display_name,
					icon: config.icon,
					category: 'container',
					installed: config.sdk_installed || false,
					enabled: true,
					version: version || undefined,
					description: config.description,
					hasToggle: true
				} as SDKItem;
			});
	});

	let aiSDKs = $derived(() => {
		return sdkConfigs
			.filter((config) => config.category === 'ai')
			.map((config) => {
				// Get version, preferring SDK version
				const rawVersion = config.sdk_version || null;
				const version = rawVersion ? rawVersion.trim().replace(/^v/, '') : null;
				
				return {
					id: config.id,
					name: config.name,
					displayName: config.display_name,
					icon: config.icon,
					category: 'ai',
					installed: config.sdk_installed || false,
					enabled: true,
					version: version || undefined,
					description: config.description,
					hasToggle: true
				} as SDKItem;
			});
	});

	// Reactive state
	let allSDKs = $derived(() => [
		...languageSDKs(),
		...databaseSDKs(),
		...webServerSDKs(),
		...containerSDKs(),
		...aiSDKs()
	]);

	let selectedItem = $state<string | null>(null);

	$effect(() => {
		selectedItem = selectedSDK || null;
	});

	// Category icons
	const categoryIcons = {
		language: Code,
		database: Database,
		web: Globe,
		container: Container,
		package: Package
	};

	// Map SDK types to their route paths - use dynamic route
	function getSDKRoute(sdkId: string, category: string): string {
		// Normalize SDK ID for routing
		const normalizedId = sdkId.toLowerCase().trim();
		
		// SDK Managers go to /sdk/manager/[name] (singular, matches route structure)
		if (category === 'manager') {
			return `/sdk/manager/${normalizedId}`;
		}
		
		// Use the new dynamic route for all SDKs (language, database, ai, server, container, etc.)
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

	function getSDKIconColor(sdkId: string): string {
		const colorMap: Record<string, string> = {
			// Language & Runtime - colored icons
			'java': 'text-orange-600',
			'node': 'text-green-600',
			'nodejs': 'text-green-600',
			'python': 'text-blue-600',
			'rust': 'text-orange-600',
			'go': 'text-blue-500',
			'php': 'text-purple-600',
			'ruby': 'text-red-600',
			'bun': 'text-yellow-600',
			'deno': 'text-gray-800',
			'gradle': 'text-blue-500',
			'kotlin': 'text-purple-600',
			'scala': 'text-red-600',
			'erlang': 'text-red-500',
			'perl': 'text-blue-700',
			
			// Database
			'mysql': 'text-blue-600',
			'postgresql': 'text-blue-700',
			'mongodb': 'text-green-600',
			'mariadb': 'text-blue-500',
			
			// Web Server
			'nginx': 'text-green-600',
			'apache': 'text-red-600',
			'caddy': 'text-blue-600',
			
			// Container
			'docker': 'text-blue-500',
			'kubernetes': 'text-blue-600',
			'podman': 'text-blue-600',
			
			// Package Managers
			'npm': 'text-red-600',
			'yarn': 'text-blue-600',
			'pip': 'text-blue-600',
			'cargo': 'text-orange-600',
			'composer': 'text-gray-700',
			'gem': 'text-red-600',
			
			// SDK Managers
			'nvm': 'text-green-600',
			'pyenv': 'text-blue-600',
			'rustup': 'text-orange-600',
			'sdkman': 'text-blue-600',
			'goenv': 'text-blue-500',
			'rbenv': 'text-red-600',
			'phpenv': 'text-purple-600'
		};
		
		return colorMap[sdkId.toLowerCase()] || '';
	}

</script>

<div class="flex flex-col h-full min-h-0 overflow-hidden">
	{#if sidebar.state === 'collapsed'}
		<!-- Header (icon-only) -->
		<div class="p-2 border-b border-border flex-shrink-0">
			<SidebarMenuButton
				size="sm"
				tooltipContent="Back to Portal Desktop"
				onclick={() => goto('/')}
			>
				<ArrowLeft class="h-4 w-4" />
			</SidebarMenuButton>
		</div>

		<!-- Sidebar Content -->
		<ScrollArea class="flex-1 min-h-0 overflow-hidden">
			<div class="p-2 space-y-4">
				<!-- Navigation Section (icon-only) -->
				<div class="space-y-1">
					{#each navigationItems as item}
						<SidebarMenuButton
							size="default"
							isActive={currentPath === item.id}
							tooltipContent={item.name}
							onclick={() => goto(item.id)}
						>
							<div class="h-4 w-4 flex items-center justify-center">
								{#if item.icon === 'lucide:layout-dashboard'}
									<Settings class="h-4 w-4" />
								{:else if item.icon === 'lucide:settings'}
									<Settings class="h-4 w-4" />
								{:else if item.icon === 'lucide:download'}
									<Download class="h-4 w-4" />
								{:else if item.icon === 'lucide:package'}
									<Package class="h-4 w-4" />
								{/if}
							</div>
						</SidebarMenuButton>
					{/each}
				</div>

				<Separator />

				{#if loading}
					<div class="text-center text-muted-foreground py-8">
						<div class="text-2xl mb-2">⏳</div>
						<h3 class="font-medium">Loading SDKs...</h3>
					</div>
				{:else if languageSDKs().length === 0 && databaseSDKs().length === 0 && webServerSDKs().length === 0 && containerSDKs().length === 0 && aiSDKs().length === 0}
					<div class="text-center text-muted-foreground py-8">
						<div class="text-2xl mb-2">🔍</div>
						<h3 class="font-medium mb-2">No SDKs Detected</h3>
						<p class="text-sm">
							Install SDK managers like NVM, Pyenv, or SDKMAN to get started.
						</p>
					</div>
				{:else}
					<!-- Language & Runtime -->
					{#if languageSDKs().length > 0}
						<div class="space-y-1">
							<SidebarMenuButton
								size="sm"
								tooltipContent={getCategoryName('language')}
								onclick={() => {}}
							>
								{@const LanguageIcon = getCategoryIcon('language')}
								<LanguageIcon class="h-4 w-4" />
							</SidebarMenuButton>
							<div class="space-y-1 pl-1">
								{#each languageSDKs() as sdk}
									{@const iconColor = getSDKIconColor(sdk.id)}
									<SidebarMenuButton
										size="default"
										isActive={selectedItem === sdk.id}
										tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
										onclick={() => handleSDKClick(sdk)}
									>
										<Devicon icon={sdk.icon} size="sm" class={iconColor} />
										{#if sdk.installed}
											<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
										{:else}
											<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
										{/if}
									</SidebarMenuButton>
								{/each}
							</div>
						</div>
					{/if}

					{#if databaseSDKs().length > 0}
						<Separator />
						<div class="space-y-1">
							<SidebarMenuButton
								size="sm"
								tooltipContent={getCategoryName('database')}
								onclick={() => {}}
							>
								{@const DatabaseIcon = getCategoryIcon('database')}
								<DatabaseIcon class="h-4 w-4" />
							</SidebarMenuButton>
							<div class="space-y-1 pl-1">
								{#each databaseSDKs() as sdk}
									{@const iconColor = getSDKIconColor(sdk.id)}
									<SidebarMenuButton
										size="default"
										isActive={selectedItem === sdk.id}
										tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
										onclick={() => handleSDKClick(sdk)}
									>
										<Devicon icon={sdk.icon} size="sm" class={iconColor} />
										{#if sdk.installed}
											<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
										{:else}
											<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
										{/if}
										{#if sdk.installed && sdk.isRunning}
											<div class="h-2 w-2 rounded-full bg-green-500" title="Running"></div>
										{:else if sdk.installed && !sdk.isRunning}
											<div class="h-2 w-2 rounded-full bg-yellow-500" title="Installed but not running"></div>
										{/if}
									</SidebarMenuButton>
								{/each}
							</div>
						</div>
					{/if}

					{#if webServerSDKs().length > 0}
						<Separator />
						<div class="space-y-1">
							<SidebarMenuButton
								size="sm"
								tooltipContent={getCategoryName('web')}
								onclick={() => {}}
							>
								{@const WebIcon = getCategoryIcon('web')}
								<WebIcon class="h-4 w-4" />
							</SidebarMenuButton>
							<div class="space-y-1 pl-1">
								{#each webServerSDKs() as sdk}
									{@const iconColor = getSDKIconColor(sdk.id)}
									<SidebarMenuButton
										size="default"
										isActive={selectedItem === sdk.id}
										tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
										onclick={() => handleSDKClick(sdk)}
									>
										<Devicon icon={sdk.icon} size="sm" class={iconColor} />
										{#if sdk.installed}
											<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
										{:else}
											<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
										{/if}
									</SidebarMenuButton>
								{/each}
							</div>
						</div>
					{/if}

					{#if containerSDKs().length > 0}
						<Separator />
						<div class="space-y-1">
							<SidebarMenuButton
								size="sm"
								tooltipContent={getCategoryName('container')}
								onclick={() => {}}
							>
								{@const ContainerIcon = getCategoryIcon('container')}
								<ContainerIcon class="h-4 w-4" />
							</SidebarMenuButton>
							<div class="space-y-1 pl-1">
								{#each containerSDKs() as sdk}
									{@const iconColor = getSDKIconColor(sdk.id)}
									<SidebarMenuButton
										size="default"
										isActive={selectedItem === sdk.id}
										tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
										onclick={() => handleSDKClick(sdk)}
									>
										<Devicon icon={sdk.icon} size="sm" class={iconColor} />
										{#if sdk.installed}
											<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
										{:else}
											<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
										{/if}
									</SidebarMenuButton>
								{/each}
							</div>
						</div>
					{/if}

					{#if aiSDKs().length > 0}
						<Separator />
						<div class="space-y-1">
							<SidebarMenuButton
								size="sm"
								tooltipContent="AI SDKs"
								onclick={() => {}}
							>
								{@const AIIcon = getCategoryIcon('ai')}
								<AIIcon class="h-4 w-4" />
							</SidebarMenuButton>
							<div class="space-y-1 pl-1">
								{#each aiSDKs() as sdk}
									{@const iconColor = getSDKIconColor(sdk.id)}
									<SidebarMenuButton
										size="default"
										isActive={selectedItem === sdk.id}
										tooltipContent={sdk.version ? `${sdk.displayName} (${sdk.version})` : sdk.displayName}
										onclick={() => handleSDKClick(sdk)}
									>
										<Devicon icon={sdk.icon} size="sm" class={iconColor} />
										{#if sdk.installed}
											<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
										{:else}
											<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
										{/if}
									</SidebarMenuButton>
								{/each}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		</ScrollArea>
	{:else}
	<!-- Header -->
	<div class="p-4 border-b border-border bg-background flex-shrink-0">
		<Button 
			variant="ghost" 
			size="sm" 
			class="w-full justify-start mb-3"
			onclick={() => goto('/')}
		>
			<ArrowLeft class="h-4 w-4 mr-2" />
			Back to Portal Desktop
		</Button>
		<h2 class="text-lg font-semibold">SDK Manager</h2>
		<p class="text-sm text-muted-foreground">Manage your development environment</p>
	</div>


	<!-- Sidebar Content -->
	<ScrollArea class="flex-1 min-h-0 overflow-hidden">
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
								{:else if item.icon === 'lucide:package'}
									<Package class="h-4 w-4" />
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
							{@const iconColor = getSDKIconColor(sdk.id)}
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
							<Devicon icon={sdk.icon} size="sm" class={iconColor} />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{:else if !sdk.installed}
									<div class="text-xs text-muted-foreground">Not installed</div>
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).port}
									<div class="text-xs text-muted-foreground">Port: {(sdk as SDKItem).port}</div>
								{/if}
							</div>
							<!-- Installation and Running Status Icons -->
							<div class="flex items-center gap-1 flex-shrink-0">
								{#if sdk.installed}
									<CheckCircle class="h-4 w-4 text-green-500" />
								{:else}
									<XCircle class="h-4 w-4 text-red-500" />
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-green-500" title="Running"></div>
								{:else if sdk.category === 'database' && sdk.installed && !(sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-yellow-500" title="Installed but not running"></div>
								{/if}
							</div>
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
						{@const iconColor = getSDKIconColor(sdk.id)}
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
							<Devicon icon={sdk.icon} size="sm" class={iconColor} />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{/if}
							</div>
							<!-- Installation Status Icon -->
							{#if sdk.installed}
								<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
							{:else}
								<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
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
						{@const iconColor = getSDKIconColor(sdk.id)}
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
							<Devicon icon={sdk.icon} size="sm" class={iconColor} />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{:else if !sdk.installed}
									<div class="text-xs text-muted-foreground">Not installed</div>
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).port}
									<div class="text-xs text-muted-foreground">Port: {(sdk as SDKItem).port}</div>
								{/if}
							</div>
							<!-- Installation and Running Status Icons -->
							<div class="flex items-center gap-1 flex-shrink-0">
								{#if sdk.installed}
									<CheckCircle class="h-4 w-4 text-green-500" />
								{:else}
									<XCircle class="h-4 w-4 text-red-500" />
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-green-500" title="Running"></div>
								{:else if sdk.category === 'database' && sdk.installed && !(sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-yellow-500" title="Installed but not running"></div>
								{/if}
							</div>
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
						{@const iconColor = getSDKIconColor(sdk.id)}
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
							<Devicon icon={sdk.icon} size="sm" class={iconColor} />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{sdk.displayName}</div>
								{#if sdk.version}
									<div class="text-xs text-muted-foreground">{sdk.version}</div>
								{:else if !sdk.installed}
									<div class="text-xs text-muted-foreground">Not installed</div>
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).port}
									<div class="text-xs text-muted-foreground">Port: {(sdk as SDKItem).port}</div>
								{/if}
							</div>
							<!-- Installation and Running Status Icons -->
							<div class="flex items-center gap-1 flex-shrink-0">
								{#if sdk.installed}
									<CheckCircle class="h-4 w-4 text-green-500" />
								{:else}
									<XCircle class="h-4 w-4 text-red-500" />
								{/if}
								{#if sdk.category === 'database' && (sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-green-500" title="Running"></div>
								{:else if sdk.category === 'database' && sdk.installed && !(sdk as SDKItem).isRunning}
									<div class="h-2 w-2 rounded-full bg-yellow-500" title="Installed but not running"></div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- AI SDKs Section -->
			{#if aiSDKs().length > 0}
				<div class="space-y-3">
					<div class="flex items-center gap-2">
						{#if true}
							{@const AIIcon = getCategoryIcon('ai')}
							<AIIcon class="h-4 w-4" />
						{/if}
						<h3 class="font-medium text-sm">AI SDKs</h3>
					</div>
					<div class="space-y-1">
						{#each aiSDKs() as sdk}
							{@const iconColor = getSDKIconColor(sdk.id)}
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
							<Devicon icon={sdk.icon} size="sm" class={iconColor} />
								<div class="flex-1 min-w-0">
									<div class="text-sm font-medium truncate">{sdk.displayName}</div>
									{#if sdk.version}
										<div class="text-xs text-muted-foreground">{sdk.version}</div>
									{/if}
								</div>
							<!-- Installation Status Icon -->
							{#if sdk.installed}
								<CheckCircle class="h-4 w-4 text-green-500 flex-shrink-0" />
							{:else}
								<XCircle class="h-4 w-4 text-red-500 flex-shrink-0" />
							{/if}
							</div>
						{/each}
					</div>
				</div>
				<Separator />
			{/if}

			<!-- No Data Fallback -->
			{#if loading}
				<div class="text-center text-muted-foreground py-8">
					<div class="text-4xl mb-4">⏳</div>
					<h3 class="font-medium mb-2">Loading SDKs...</h3>
				</div>
			{:else if languageSDKs().length === 0 && databaseSDKs().length === 0 && webServerSDKs().length === 0 && containerSDKs().length === 0 && aiSDKs().length === 0}
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
				{allSDKs().filter(sdk => sdk.installed).length} of {allSDKs().length} SDKs
			</div>
		</div>
	</div>
	{/if}
</div>
