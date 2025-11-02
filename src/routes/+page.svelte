<!--
	Home page - Modern Dashboard
	Comprehensive overview with quick actions and statistics
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Badge } from '@/lib/components/ui/badge';
	import { Separator } from '@/lib/components/ui/separator';
	import { Progress } from '@/lib/components/ui/progress';
	import { 
		Terminal, 
		Code, 
		Folder, 
		CheckSquare, 
		Lock, 
		Rocket, 
		FileText, 
		Settings,
		Plus,
		Activity,
		Clock,
		Star,
		GitBranch,
		Zap,
		Database,
		Server,
		Monitor
	} from 'lucide-svelte';
	import { projectStore } from '@/lib/domains/projects';
	import { taskStats } from '@/lib/domains/tasks';
	import { terminalStore } from '@/lib/domains/terminal';

	// Reactive data with null safety
	$: projects = $projectStore.projects || [];
	$: recentProjects = projects.slice(0, 3);
	$: starredProjects = projects.filter(p => p.starred);
	$: activeTabs = $terminalStore.tabs?.length || 0;
	$: taskStatsData = $taskStats || { total: 0, completed: 0, inProgress: 0, pending: 0 };

	// Quick actions
	const quickActions = [
		{
			title: 'New Project',
			description: 'Create a new project',
			icon: Plus,
			href: '/projects',
			variant: 'default' as const,
			action: 'create'
		},
		{
			title: 'Open Terminal',
			description: 'Launch integrated terminal',
			icon: Terminal,
			href: '/terminal',
			variant: 'secondary' as const,
			action: 'terminal'
		},
		{
			title: 'SDK Manager',
			description: 'Manage SDK versions',
			icon: Code,
			href: '/sdk',
			variant: 'outline' as const,
			action: 'sdk'
		},
		{
			title: 'Deployments',
			description: 'Manage Docker deployments',
			icon: Rocket,
			href: '/deployments',
			variant: 'outline' as const,
			action: 'deploy'
		}
	];

	// Navigation sections with null safety
	$: navigationSections = [
		{
			title: 'Development',
			items: [
				{ title: 'Projects', href: '/projects', icon: Folder, count: projects.length },
				{ title: 'Terminal', href: '/terminal', icon: Terminal, count: activeTabs },
				{ title: 'SDK Manager', href: '/sdk', icon: Code, count: null }
			]
		},
		{
			title: 'Management',
			items: [
				{ title: 'Tasks', href: '/tasks', icon: CheckSquare, count: taskStatsData.total },
				{ title: 'Credentials', href: '/credentials', icon: Lock, count: null },
				{ title: 'Deployments', href: '/deployments', icon: Rocket, count: null }
			]
		},
		{
			title: 'System',
			items: [
				{ title: 'Documents', href: '/documents', icon: FileText, count: null },
				{ title: 'Settings', href: '/settings', icon: Settings, count: null }
			]
		}
	];

	// System stats with null safety
	$: systemStats = [
		{
			title: 'Active Projects',
			value: projects.length,
			change: '+2 this week',
			icon: Folder,
			color: 'text-blue-600'
		},
		{
			title: 'Terminal Sessions',
			value: activeTabs,
			change: '3 running',
			icon: Terminal,
			color: 'text-green-600'
		},
		{
			title: 'Tasks Completed',
			value: taskStatsData.completed,
			change: `${Math.round((taskStatsData.completed / Math.max(taskStatsData.total, 1)) * 100)}% done`,
			icon: CheckSquare,
			color: 'text-purple-600'
		},
		{
			title: 'Starred Projects',
			value: starredProjects.length,
			change: 'Favorites',
			icon: Star,
			color: 'text-yellow-600'
		}
	];

	function handleQuickAction(action: string) {
		switch (action) {
			case 'create':
				goto('/projects');
				break;
			case 'terminal':
				goto('/terminal');
				break;
			case 'sdk':
				goto('/sdk');
				break;
			case 'deploy':
				goto('/deployments');
				break;
		}
	}

	function formatDate(date: Date): string {
		return new Intl.RelativeTimeFormat('en', { numeric: 'auto' }).format(
			Math.ceil((date.getTime() - Date.now()) / (1000 * 60 * 60 * 24)),
			'day'
		);
	}
</script>

<svelte:head>
	<title>Portal Desktop - Dashboard</title>
</svelte:head>

<main class="min-h-screen bg-background">
	<!-- Header -->
	<div class="border-b bg-card">
		<div class="container mx-auto px-6 py-8">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold tracking-tight">Welcome back!</h1>
					<p class="text-muted-foreground mt-2">
						Here's what's happening with your development environment.
					</p>
				</div>
				<div class="flex items-center space-x-2">
					<Badge variant="outline" class="flex items-center gap-1">
						<Activity class="h-3 w-3" />
						All systems operational
					</Badge>
				</div>
			</div>
		</div>
	</div>

	<!-- Main Content -->
	<div class="container mx-auto px-6 py-8">
		<div class="grid gap-8">
			<!-- System Stats -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
				{#each systemStats as stat (stat.title)}
					<Card>
						<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle class="text-sm font-medium">{stat.title}</CardTitle>
							<svelte:component this={stat.icon} class="h-4 w-4 {stat.color}" />
						</CardHeader>
						<CardContent>
							<div class="text-2xl font-bold">{stat.value}</div>
							<p class="text-xs text-muted-foreground">{stat.change}</p>
						</CardContent>
					</Card>
				{/each}
			</div>

			<!-- Quick Actions -->
			<Card>
				<CardHeader>
					<CardTitle>Quick Actions</CardTitle>
					<CardDescription>
						Get started with common tasks
					</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
						{#each quickActions as action (action.title)}
							<Button
								variant={action.variant}
								class="h-auto p-6 flex flex-col items-center space-y-2"
								onclick={() => handleQuickAction(action.action)}
							>
								<svelte:component this={action.icon} class="h-6 w-6" />
								<div class="text-center">
									<div class="font-medium">{action.title}</div>
									<div class="text-xs opacity-70">{action.description}</div>
								</div>
							</Button>
						{/each}
					</div>
				</CardContent>
			</Card>

			<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
				<!-- Recent Projects -->
				<Card>
					<CardHeader>
						<CardTitle>Recent Projects</CardTitle>
						<CardDescription>
							Your most recently accessed projects
						</CardDescription>
					</CardHeader>
					<CardContent>
						{#if recentProjects.length > 0}
							<div class="space-y-4">
								{#each recentProjects as project (project.id)}
									<div class="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors">
										<div class="flex items-center space-x-3">
											<div class="p-2 bg-primary/10 rounded-lg">
												<Folder class="h-4 w-4 text-primary" />
											</div>
											<div>
												<div class="font-medium">{project.name}</div>
												<div class="text-sm text-muted-foreground">
													{project.framework || 'No framework'}
												</div>
											</div>
										</div>
										<div class="flex items-center space-x-2">
											{#if project.starred}
												<Star class="h-4 w-4 text-yellow-500 fill-current" />
											{/if}
											<Button variant="ghost" size="sm" onclick={() => goto(`/projects/${project.id}`)}>
												Open
											</Button>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-8">
								<Folder class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
								<p class="text-muted-foreground">No projects yet</p>
								<Button variant="outline" class="mt-4" onclick={() => goto('/projects')}>
									<Plus class="h-4 w-4 mr-2" />
									Create your first project
								</Button>
							</div>
						{/if}
					</CardContent>
				</Card>

				<!-- Navigation -->
				<Card>
					<CardHeader>
						<CardTitle>Navigation</CardTitle>
						<CardDescription>
							Access all application features
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="space-y-6">
							{#each navigationSections as section (section.title)}
								<div>
									<h4 class="text-sm font-medium text-muted-foreground mb-3">{section.title}</h4>
									<div class="space-y-2">
										{#each section.items as item (item.title)}
											<Button
												variant="ghost"
												class="w-full justify-start h-auto p-3"
												onclick={() => goto(item.href)}
											>
												<svelte:component this={item.icon} class="h-4 w-4 mr-3" />
												<div class="flex-1 text-left">
													<div class="font-medium">{item.title}</div>
													{#if item.count !== null}
														<div class="text-xs text-muted-foreground">
															{item.count} {item.count === 1 ? 'item' : 'items'}
														</div>
													{/if}
												</div>
												{#if item.count !== null}
													<Badge variant="secondary" class="ml-2">
														{item.count}
													</Badge>
												{/if}
											</Button>
										{/each}
									</div>
								</div>
								{#if section !== navigationSections[navigationSections.length - 1]}
									<Separator />
								{/if}
							{/each}
						</div>
					</CardContent>
				</Card>
			</div>

			<!-- Task Progress -->
			{#if taskStatsData.total > 0}
				<Card>
					<CardHeader>
						<CardTitle>Task Progress</CardTitle>
						<CardDescription>
							Your current task completion status
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div class="space-y-4">
							<div class="flex items-center justify-between">
								<span class="text-sm font-medium">Overall Progress</span>
								<span class="text-sm text-muted-foreground">
									{taskStatsData.completed} of {taskStatsData.total} tasks
								</span>
							</div>
							<Progress 
								value={Math.round((taskStatsData.completed / Math.max(taskStatsData.total, 1)) * 100)} 
								class="h-2"
							/>
							<div class="grid grid-cols-3 gap-4 text-center">
								<div>
									<div class="text-2xl font-bold text-green-600">{taskStatsData.completed}</div>
									<div class="text-xs text-muted-foreground">Completed</div>
								</div>
								<div>
									<div class="text-2xl font-bold text-yellow-600">{taskStatsData.inProgress}</div>
									<div class="text-xs text-muted-foreground">In Progress</div>
								</div>
								<div>
									<div class="text-2xl font-bold text-gray-600">{taskStatsData.pending}</div>
									<div class="text-xs text-muted-foreground">Pending</div>
								</div>
							</div>
						</div>
					</CardContent>
				</Card>
			{/if}

			<!-- System Status -->
			<Card>
				<CardHeader>
					<CardTitle>System Status</CardTitle>
					<CardDescription>
						Current system health and performance
					</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
						<div class="flex items-center space-x-3">
							<div class="p-2 bg-green-100 dark:bg-green-900/20 rounded-lg">
								<Server class="h-4 w-4 text-green-600" />
							</div>
							<div>
								<div class="font-medium">Backend Services</div>
								<div class="text-sm text-green-600">Operational</div>
							</div>
						</div>
						<div class="flex items-center space-x-3">
							<div class="p-2 bg-blue-100 dark:bg-blue-900/20 rounded-lg">
								<Database class="h-4 w-4 text-blue-600" />
							</div>
							<div>
								<div class="font-medium">Database</div>
								<div class="text-sm text-blue-600">Connected</div>
							</div>
						</div>
						<div class="flex items-center space-x-3">
							<div class="p-2 bg-purple-100 dark:bg-purple-900/20 rounded-lg">
								<Monitor class="h-4 w-4 text-purple-600" />
							</div>
							<div>
								<div class="font-medium">Terminal</div>
								<div class="text-sm text-purple-600">{activeTabs} sessions</div>
							</div>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	</div>
</main>