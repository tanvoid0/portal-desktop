<script lang="ts">
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { projectStore, recentProjects } from '@/lib/domains/projects';
	import { taskStats } from '@/lib/domains/tasks';
	import { invoke } from '@tauri-apps/api/core';

	// Get current time for greeting
	const currentHour = new Date().getHours();
	const greeting = currentHour < 12 ? 'Good morning' : currentHour < 18 ? 'Good afternoon' : 'Good evening';

	// Quick actions configuration
	const quickActions = [
		{
			title: 'New Project',
			description: 'Create a new project',
			icon: 'folder-plus',
			url: '/projects/create',
			color: 'text-blue-600 dark:text-blue-400'
		},
		{
			title: 'New Task',
			description: 'Add a new task',
			icon: 'check-square',
			url: '/tasks/create',
			color: 'text-green-600 dark:text-green-400'
		},
		{
			title: 'Terminal',
			description: 'Open terminal',
			icon: 'terminal',
			url: '/terminal',
			color: 'text-purple-600 dark:text-purple-400'
		},
		{
			title: 'SDK Manager',
			description: 'Manage SDK versions',
			icon: 'code',
			url: '/sdk',
			color: 'text-orange-600 dark:text-orange-400'
		},
		{
			title: 'Cloud Resources',
			description: 'Manage cloud resources',
			icon: 'cloud',
			url: '/cloud',
			color: 'text-cyan-600 dark:text-cyan-400'
		},
		{
			title: 'Credentials',
			description: 'Manage credentials',
			icon: 'lock',
			url: '/credentials',
			color: 'text-red-600 dark:text-red-400'
		},
		{
			title: 'Pipeline Blocks',
			description: 'Manage reusable blocks',
			icon: 'blocks',
			url: '/blocks',
			color: 'text-indigo-600 dark:text-indigo-400'
		}
	];

	// Main navigation items
	const mainNavItems = [
		{
			title: 'Projects',
			description: 'Manage your projects',
			url: '/projects',
			icon: 'folder',
			badge: $projectStore.projects.length,
			color: 'bg-blue-500/10 text-blue-600 dark:text-blue-400'
		},
		{
			title: 'Tasks',
			description: 'View and manage tasks',
			url: '/tasks',
			icon: 'check-square',
			badge: $taskStats.total,
			color: 'bg-green-500/10 text-green-600 dark:text-green-400'
		},
		{
			title: 'Terminal',
			description: 'Integrated terminal',
			url: '/terminal',
			icon: 'terminal',
			color: 'bg-purple-500/10 text-purple-600 dark:text-purple-400'
		},
		{
			title: 'Cloud',
			description: 'Cloud resources',
			url: '/cloud',
			icon: 'cloud',
			color: 'bg-cyan-500/10 text-cyan-600 dark:text-cyan-400'
		}
	];

	let runningServicesCount = $state(0);

	// Load running services count
	async function loadRunningServicesCount() {
		try {
			const result = await invoke('get_running_services_count') as number;
			runningServicesCount = result || 0;
		} catch (err) {
			console.error('Failed to load running services count:', err);
			runningServicesCount = 0;
		}
	}

	loadRunningServicesCount();

	// Icon component helper
	function getIcon(iconName: string) {
		const icons: Record<string, string> = {
			'folder-plus': 'M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
			'check-square': 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
			'terminal': 'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
			'code': 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
			'blocks': 'M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z',
			'cloud': 'M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z',
			'lock': 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
			'folder': 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
			'settings': 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
			'container': 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4'
		};
		return icons[iconName] || icons['folder'];
	}
</script>

<div class="container mx-auto p-6 space-y-6 max-w-7xl">
	<!-- Welcome Header -->
	<div class="mb-8">
		<h1 class="text-4xl font-bold mb-2">{greeting}!</h1>
		<p class="text-muted-foreground text-lg">
			Welcome to Portal Desktop. Here's what's happening with your workspace.
		</p>
	</div>

	<!-- Stats Overview -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
		<Card>
			<CardContent class="p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground mb-1">Total Tasks</p>
						<p class="text-3xl font-bold">{$taskStats.total}</p>
						{#if $taskStats.completed > 0}
							<p class="text-xs text-green-600 dark:text-green-400 mt-1">
								{$taskStats.completed} completed
							</p>
						{/if}
					</div>
					<div class="w-12 h-12 rounded-full bg-green-500/10 flex items-center justify-center">
						<svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
						</svg>
					</div>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardContent class="p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground mb-1">Projects</p>
						<p class="text-3xl font-bold">{$projectStore.projects.length}</p>
						{#if $recentProjects.length > 0}
							<p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
								{$recentProjects.length} recent
							</p>
						{/if}
					</div>
					<div class="w-12 h-12 rounded-full bg-blue-500/10 flex items-center justify-center">
						<svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
						</svg>
					</div>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardContent class="p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground mb-1">Running Services</p>
						<p class="text-3xl font-bold">{runningServicesCount}</p>
						<p class="text-xs text-muted-foreground mt-1">
							SDK services
						</p>
					</div>
					<div class="w-12 h-12 rounded-full bg-purple-500/10 flex items-center justify-center">
						<svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
						</svg>
					</div>
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardContent class="p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground mb-1">Completion Rate</p>
						<p class="text-3xl font-bold">
							{$taskStats.total > 0 ? Math.round(($taskStats.completed / $taskStats.total) * 100) : 0}%
						</p>
						<p class="text-xs text-muted-foreground mt-1">
							Tasks completed
						</p>
					</div>
					<div class="w-12 h-12 rounded-full bg-orange-500/10 flex items-center justify-center">
						<svg class="w-6 h-6 text-orange-600 dark:text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
						</svg>
					</div>
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Quick Actions -->
	<Card>
		<CardHeader>
			<CardTitle>Quick Actions</CardTitle>
			<CardDescription>Common tasks and shortcuts</CardDescription>
		</CardHeader>
		<CardContent>
			<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
				{#each quickActions as action}
					<Button
						variant="outline"
						class="h-24 flex flex-col gap-2 hover:bg-accent transition-colors"
						onclick={() => goto(action.url)}
					>
						<svg class="w-6 h-6 {action.color}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(action.icon)} />
						</svg>
						<span class="text-sm font-medium">{action.title}</span>
					</Button>
				{/each}
			</div>
		</CardContent>
	</Card>

	<!-- Main Navigation -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<Card class="cursor-pointer hover:shadow-lg transition-shadow" onclick={() => goto('/projects')}>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
							<svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
							</svg>
						</div>
						Projects
					</CardTitle>
					{#if $projectStore.projects.length > 0}
						<Badge variant="secondary">{$projectStore.projects.length}</Badge>
					{/if}
				</div>
				<CardDescription>
					Manage your projects and workspaces
				</CardDescription>
			</CardHeader>
			<CardContent>
				<p class="text-sm text-muted-foreground">
					{#if $projectStore.projects.length === 0}
						No projects yet. Create your first project to get started.
					{:else if $recentProjects.length > 0}
						Recent: {$recentProjects[0].name}
					{:else}
						{$projectStore.projects.length} project{$projectStore.projects.length !== 1 ? 's' : ''} available
					{/if}
				</p>
			</CardContent>
		</Card>

		<Card class="cursor-pointer hover:shadow-lg transition-shadow" onclick={() => goto('/tasks')}>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center">
							<svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
							</svg>
						</div>
						Tasks
					</CardTitle>
					{#if $taskStats.total > 0}
						<Badge variant="secondary">{$taskStats.total}</Badge>
					{/if}
				</div>
				<CardDescription>
					View and manage your tasks
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="space-y-2">
					<div class="flex items-center justify-between text-sm">
						<span class="text-muted-foreground">Total</span>
						<span class="font-medium">{$taskStats.total}</span>
					</div>
					<div class="flex items-center justify-between text-sm">
						<span class="text-muted-foreground">Completed</span>
						<span class="font-medium text-green-600 dark:text-green-400">{$taskStats.completed}</span>
					</div>
					{#if $taskStats.total > 0}
						<div class="w-full bg-secondary rounded-full h-2 mt-2">
							<div 
								class="bg-green-600 h-2 rounded-full transition-all"
								style="width: {($taskStats.completed / $taskStats.total) * 100}%"
							></div>
						</div>
					{/if}
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Additional Quick Links -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
		{#each mainNavItems.slice(2) as item}
			<Card class="cursor-pointer hover:shadow-lg transition-shadow" onclick={() => goto(item.url)}>
				<CardContent class="p-6">
					<div class="flex items-center gap-4">
						<div class="w-12 h-12 rounded-lg {item.color} flex items-center justify-center">
							<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getIcon(item.icon)} />
							</svg>
						</div>
						<div class="flex-1">
							<h3 class="font-semibold mb-1">{item.title}</h3>
							<p class="text-sm text-muted-foreground">{item.description}</p>
						</div>
						{#if item.badge}
							<Badge variant="secondary">{item.badge}</Badge>
						{/if}
					</div>
				</CardContent>
			</Card>
		{/each}
	</div>
</div>

