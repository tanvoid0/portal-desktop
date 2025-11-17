<!--
	Main application layout
	Integrates all domains with navigation and responsive design
-->

<script lang="ts">
	import '../app.css';
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import { Sidebar, SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider, SidebarTrigger } from '@/lib/components/ui/sidebar';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import ThemeToggle from '@/lib/components/ui/theme-toggle.svelte';
	import Breadcrumb from '@/lib/components/ui/breadcrumb.svelte';
	import { projectStore, projectService } from '@/lib/domains/projects';
	import { taskStats } from '@/lib/domains/tasks';
	import { logger, themeStore, resolvedTheme } from '@/lib/domains/shared';
	import { breadcrumbItems, breadcrumbSettings, homeItem, showHome } from '@/lib/domains/shared/stores/breadcrumbStore';
	import { terminalActions } from '@/lib/domains/terminal/stores/terminalStore';
	import { learningService } from '@/lib/domains/learning';
	import ToastContainer from '@/lib/components/ui/toast-container.svelte';
	import { invoke } from '@tauri-apps/api/core';

	const log = logger.createScoped('AppLayout');

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();

	// Dynamic navigation state
	let currentNavigationSections: any[] = $state([]);
	let navigationLoading = $state(false);
	let navigationError: string | null = $state(null);
	let isSdkPage = $derived($page.url.pathname.startsWith('/sdk'));
	let runningServicesCount = $state(0);
	let sdkSubmenuOpen = $state(false);

	// Main application navigation sections - derived to react to runningServicesCount changes
	const navigationSections = $derived([
		{
			title: 'Navigation',
			items: [
				{
					title: 'Overview',
					url: '/',
					icon: 'home',
					description: 'Portal Desktop home',
					badge: null
				},
				{
					title: 'SDK Manager',
					url: '/sdk',
					icon: 'code',
					description: 'SDK version management',
					badge: runningServicesCount > 0 ? runningServicesCount : null,
					submenu: [
						{
							title: 'Node.js',
							url: '/sdk/nodejs',
							icon: 'node-js',
							description: 'Node.js SDK management'
						},
						{
							title: 'Python',
							url: '/sdk/python',
							icon: 'python',
							description: 'Python SDK management'
						},
						{
							title: 'Java',
							url: '/sdk/java',
							icon: 'coffee',
							description: 'Java SDK management'
						},
						{
							title: 'Rust',
							url: '/sdk/rust',
							icon: 'rust',
							description: 'Rust SDK management'
						},
						{
							title: 'Go',
							url: '/sdk/go',
							icon: 'go',
							description: 'Go SDK management'
						},
						{
							title: 'PHP',
							url: '/sdk/php',
							icon: 'php',
							description: 'PHP SDK management'
						},
						{
							title: 'Ruby',
							url: '/sdk/ruby',
							icon: 'ruby',
							description: 'Ruby SDK management'
						}
					]
				}
			]
		},
		{
			title: 'Tools',
			items: [
				{
					title: 'Terminal',
					url: '/terminal',
					icon: 'terminal',
					description: 'Integrated terminal',
					badge: null
				},
				{
					title: 'Projects',
					url: '/projects',
					icon: 'folder',
					description: 'Project management',
					badge: $projectStore.projects.length
				},
				{
					title: 'Tasks',
					url: '/tasks',
					icon: 'check-square',
					description: 'Task management',
					badge: null
				},
				{
					title: 'Credentials',
					url: '/credentials',
					icon: 'lock',
					description: 'Secure credential vault',
					badge: null
				},
				{
					title: 'Cloud',
					url: '/cloud',
					icon: 'cloud',
					description: 'Cloud resources management',
					badge: null
				},
				{
					title: 'Docker Containers',
					url: '/deployments',
					icon: 'container',
					description: 'Local Docker container management',
					badge: null
				},
				{
					title: 'Documents',
					url: '/documents',
					icon: 'file-text',
					description: 'Workspace documentation',
					badge: null
				},
				{
					title: 'Pipeline Blocks',
					url: '/blocks',
					icon: 'blocks',
					description: 'Manage reusable pipeline blocks',
					badge: null
				},
				{
					title: 'AI',
					url: '/ai',
					icon: 'sparkles',
					description: 'AI chat and management',
					badge: null
				},
				{
					title: 'Utilities',
					url: '/utilities',
					icon: 'wrench',
					description: 'Custom scripts and utilities',
					badge: null
				},
				{
					title: 'Settings',
					url: '/settings',
					icon: 'settings',
					description: 'Application settings',
					badge: null
				}
			]
		}
	]);

	let unsubscribe: (() => void) | undefined;

	onMount(async () => {
		try {
			log.info('Initializing application');
			
			// Initialize theme first (now synchronous)
			themeStore.initialize();
			
			// Initialize learning service (should be early to start collecting patterns)
			await learningService.initialize();
			
			// Initialize project service
			await projectService.initialize();
			
			// Sync terminal theme with global theme
			unsubscribe = resolvedTheme.subscribe((theme) => {
				terminalActions.updateSettings({ theme });
			});
			
			log.info('Application initialized successfully');
		} catch (error) {
			log.error('Failed to initialize application', error);
		}
	});

	// Load domain-specific navigation based on current page
	$effect(() => {
		if (isSdkPage) {
			loadSdkNavigation();
		} else {
			// Reset to main navigation for non-SDK pages
			currentNavigationSections = [...navigationSections];
			navigationLoading = false;
			navigationError = null;
		}
		// Always load running services count
		loadRunningServicesCount();
	});

	async function loadSdkNavigation() {
		try {
			navigationLoading = true;
			navigationError = null;
			
			const response = await invoke('get_sdk_navigation_items') as any;
			currentNavigationSections = response.sections || [];
		} catch (err) {
			console.error('Failed to load SDK navigation:', err);
			navigationError = err instanceof Error ? err.message : 'Failed to load SDK navigation';
			currentNavigationSections = [];
		} finally {
			navigationLoading = false;
		}
	}

	async function loadRunningServicesCount() {
		try {
			const result = await invoke('get_running_services_count') as number;
			runningServicesCount = result || 0;
		} catch (err) {
			console.error('Failed to load running services count:', err);
			runningServicesCount = 0;
		}
	}

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
	});
</script>

<SidebarProvider>
	<Sidebar class="border-r">
		<!-- Sidebar Header -->
		<div class="flex items-center gap-3 px-4 py-4 border-b cursor-pointer hover:bg-sidebar-accent/50 transition-colors" role="button" tabindex="0" onclick={() => goto('/')} onkeydown={(e) => e.key === 'Enter' && goto('/')}>
			<div class="flex items-center justify-center w-8 h-8 rounded-md bg-primary/10 text-primary">
				{#if isSdkPage}
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
					</svg>
				{:else}
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
					</svg>
				{/if}
			</div>
			<div class="flex-1 min-w-0">
				<h1 class="text-sm font-semibold text-sidebar-foreground truncate">
					{isSdkPage ? 'SDK Manager' : 'Portal Desktop'}
				</h1>
				<p class="text-xs text-sidebar-foreground/60">
					{isSdkPage ? 'Development Tools' : 'Development Environment'}
				</p>
			</div>
		</div>

		<SidebarContent>
			{#if isSdkPage}
				<!-- Back to Home Button for SDK pages -->
				<div class="px-4 py-2 border-b">
					<a 
						href="/" 
						class="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
					>
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
						</svg>
						Back to Portal Desktop
					</a>
				</div>
			{/if}

			{#if navigationLoading}
				<div class="flex items-center justify-center p-4">
					<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
					<span class="ml-2 text-sm text-muted-foreground">Loading navigation...</span>
				</div>
			{:else if navigationError}
				<div class="p-4">
					<div class="text-sm text-destructive">
						<p class="font-medium">Failed to load navigation</p>
						<p class="text-xs text-muted-foreground mt-1">{navigationError}</p>
					</div>
				</div>
			{:else}
				{#each currentNavigationSections as section}
				<SidebarGroup>
					<SidebarGroupLabel>{section.title}</SidebarGroupLabel>
					<SidebarMenu>
						{#each section.items as item}
							<SidebarMenuItem>
								<SidebarMenuButton 
									size="lg"
									variant={$page.url.pathname === item.url ? 'default' : 'outline'}
									isActive={$page.url.pathname === item.url}
									class="h-12 px-4 py-3 text-base font-medium hover:bg-sidebar-accent hover:text-sidebar-accent-foreground data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground data-[active=true]:font-medium"
									onclick={() => goto(item.url)}
								>
								{#if item.icon === 'home'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
									</svg>
								{:else if item.icon === 'terminal'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
									</svg>
								{:else if item.icon === 'folder'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
									</svg>
								{:else if item.icon === 'check-square'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
									</svg>
								{:else if item.icon === 'code'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
									</svg>
								{:else if item.icon === 'lock'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
									</svg>
								{:else if item.icon === 'rocket'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
									</svg>
								{:else if item.icon === 'file-text'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
									</svg>
								{:else if item.icon === 'settings'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
									</svg>
								{:else if item.icon === 'database'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"/>
									</svg>
								{:else if item.icon === 'container'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
									</svg>
								{:else if item.icon === 'cloud'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"/>
									</svg>
								{:else if item.icon === 'globe'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9m0-9v9"/>
									</svg>
								{:else if item.icon === 'robot'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"/>
									</svg>
								{:else if item.icon === 'download'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
									</svg>
								{:else if item.icon === 'play'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1m4 0h1m-6 4h1m4 0h1m-6-8h8a2 2 0 012 2v8a2 2 0 01-2 2H8a2 2 0 01-2-2v-8a2 2 0 012-2z"/>
									</svg>
								{:else if item.icon === 'blocks'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"/>
									</svg>
								{:else if item.icon === 'wrench'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.42 15.17L17.25 21A2.652 2.652 0 0021 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655-5.653a2.548 2.548 0 010-3.586l4.94-4.94a2.548 2.548 0 013.586 0l4.94 4.94a2.548 2.548 0 010 3.586l-5.877 5.877"/>
									</svg>
								{:else if item.icon === 'sparkles'}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"/>
									</svg>
								<!-- SDK-specific language icons -->
								{:else if item.icon === 'nodejs'}
									<i class="devicon-nodejs-plain w-5 h-5 text-green-600"></i>
								{:else if item.icon === 'python'}
									<i class="devicon-python-plain w-5 h-5 text-blue-600"></i>
								{:else if item.icon === 'java'}
									<i class="devicon-java-plain w-5 h-5 text-orange-600"></i>
								{:else if item.icon === 'rust'}
									<i class="devicon-rust-plain w-5 h-5 text-orange-600"></i>
								{:else if item.icon === 'go'}
									<i class="devicon-go-plain w-5 h-5 text-blue-600"></i>
								{:else if item.icon === 'php'}
									<i class="devicon-php-plain w-5 h-5 text-purple-600"></i>
								{:else if item.icon === 'ruby'}
									<i class="devicon-ruby-plain w-5 h-5 text-red-600"></i>
								{:else}
									<!-- Fallback icon for unknown icons -->
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
									</svg>
								{/if}
								<span class="text-base">{item.title}</span>
								<div class="ml-auto flex items-center gap-2">
									{#if isSdkPage && (item as any).category !== 'sdk-navigation' && (item as any).installed !== undefined}
										{#if (item as any).installed}
											<!-- Green checkmark for installed -->
											<svg class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
											</svg>
										{:else}
											<!-- Red X for not installed -->
											<svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
											</svg>
										{/if}
									{/if}
									{#if item.badge !== null}
										<Badge variant="secondary" class="text-sm px-2 py-1">
											{item.badge}
										</Badge>
									{/if}
								</div>
							</SidebarMenuButton>
						</SidebarMenuItem>
					{/each}
				</SidebarMenu>
			</SidebarGroup>
		{/each}
			{/if}

			{#if !isSdkPage}
				<Separator class="my-4" />

			<SidebarGroup>
				<SidebarGroupLabel>Quick Stats</SidebarGroupLabel>
				<div class="space-y-2">
					<!-- Total Tasks -->
					<div class="flex items-center justify-between px-2 py-1">
						<span class="text-sm text-sidebar-foreground/70">Total Tasks</span>
						<Badge variant="outline" class="text-xs">
							{$taskStats.total}
						</Badge>
					</div>
					
					<!-- Completed Tasks -->
					<div class="flex items-center justify-between px-2 py-1">
						<span class="text-sm text-sidebar-foreground/70">Completed</span>
						<Badge variant="outline" class="text-xs text-green-600">
							{$taskStats.completed}
						</Badge>
					</div>
					
					<!-- Active Projects -->
					<div class="flex items-center justify-between px-2 py-1">
						<span class="text-sm text-sidebar-foreground/70">Projects</span>
						<Badge variant="outline" class="text-xs">
							{$projectStore.projects.length}
						</Badge>
					</div>
				</div>
			</SidebarGroup>
			{/if}
		</SidebarContent>

		<!-- Sidebar Footer -->
		<div class="border-t p-4">
			<div class="flex items-center justify-between">
				<ThemeToggle />
				<Button variant="ghost" size="sm" onclick={() => goto('/settings')}>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
					</svg>
				</Button>
			</div>
		</div>
	</Sidebar>

	<!-- Main Content Area -->
	<div class="flex-1 flex flex-col min-h-screen">
		<!-- Top Navigation Bar -->
		<header class="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
			<div class="flex h-14 items-center px-4">
				<SidebarTrigger class="mr-2" />
				<Breadcrumb items={$breadcrumbItems} showHome={$showHome} homeItem={$homeItem} class="flex-1" />
			</div>
		</header>

		<!-- Main Content -->
		<main class="flex-1 overflow-auto">
			{@render children()}
		</main>
	</div>
</SidebarProvider>

<!-- Toast Container -->
<ToastContainer />