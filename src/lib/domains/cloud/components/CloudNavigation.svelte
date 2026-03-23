<!--
	Cloud Navigation - Sidebar navigation for cloud resources with grouped sections
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';
	import NamespaceSelector from '$lib/domains/cloud/components/NamespaceSelector.svelte';
	import { Container, Network, Settings, Globe } from '@lucide/svelte';
	import { useKeyboardShortcuts } from '$lib/domains/k8s-navigation';
	import { RESOURCE_TYPE_SHORTCUTS } from '$lib/domains/k8s-navigation/utils/keyboardConstants';
	import { cloudStore } from '$lib/domains/cloud/stores';

	interface NavItem {
		id: string;
		label: string;
		description: string;
		icon: string;
		route: string;
	}

	interface NavGroup {
		title: string;
		items: NavItem[];
	}

	const navGroups: NavGroup[] = [
		{
			title: 'Overview',
			items: [
				{
					id: 'overview',
					label: 'Overview',
					description: 'Cluster overview and metrics',
					icon: '☸️',
					route: '/cloud/workloads'
				}
			]
		},
		{
			title: 'Workloads',
			items: [
				{
					id: 'pods',
					label: 'Pods',
					description: 'Container instances',
					icon: '📦',
					route: '/cloud/workloads/pods'
				},
				{
					id: 'services',
					label: 'Services',
					description: 'Network services',
					icon: '🔗',
					route: '/cloud/workloads/services'
				},
				{
					id: 'deployments',
					label: 'Deployments',
					description: 'Stateless workloads',
					icon: '🚀',
					route: '/cloud/workloads/deployments'
				},
				{
					id: 'statefulsets',
					label: 'StatefulSets',
					description: 'Stateful workloads',
					icon: '🗄️',
					route: '/cloud/workloads/statefulsets'
				},
				{
					id: 'daemonsets',
					label: 'DaemonSets',
					description: 'Node-level workloads',
					icon: '👹',
					route: '/cloud/workloads/daemonsets'
				},
				{
					id: 'jobs',
					label: 'Jobs',
					description: 'One-time tasks',
					icon: '⚙️',
					route: '/cloud/workloads/jobs'
				},
				{
					id: 'cronjobs',
					label: 'CronJobs',
					description: 'Scheduled tasks',
					icon: '⏰',
					route: '/cloud/workloads/cronjobs'
				}
			]
		},
		{
			title: 'Configuration',
			items: [
				{
					id: 'configmaps',
					label: 'ConfigMaps',
					description: 'Configuration data',
					icon: '⚙️',
					route: '/cloud/configmaps'
				},
				{
					id: 'secrets',
					label: 'Secrets',
					description: 'Sensitive data',
					icon: '🔐',
					route: '/cloud/secrets'
				}
			]
		},
		{
			title: 'Networking',
			items: [
				{
					id: 'ingress',
					label: 'Ingress',
					description: 'External access rules',
					icon: '🌐',
					route: '/cloud/ingress'
				}
			]
		}
	];

	function isActiveTab(tabRoute: string): boolean {
		const currentPath = $page.url.pathname;
		// Special handling for overview tab (exact match for /cloud/workloads)
		if (tabRoute === '/cloud/workloads') {
			return currentPath === '/cloud/workloads';
		}
		// For configmaps, secrets, ingress, check exact match or detail pages
		if (tabRoute === '/cloud/configmaps') {
			return currentPath === '/cloud/configmaps' || currentPath.startsWith('/cloud/configmaps/');
		}
		if (tabRoute === '/cloud/secrets') {
			return currentPath === '/cloud/secrets' || currentPath.startsWith('/cloud/secrets/');
		}
		if (tabRoute === '/cloud/ingress') {
			return currentPath === '/cloud/ingress' || currentPath.startsWith('/cloud/ingress/');
		}
		// For other tabs, check if current path starts with the tab route
		return currentPath.startsWith(tabRoute);
	}

	function handleItemClick(route: string) {
		goto(route);
	}
	
	// Resource type navigation shortcuts
	const resourceTypeShortcuts = useKeyboardShortcuts(
		[
			{
				key: RESOURCE_TYPE_SHORTCUTS.OVERVIEW,
				description: 'Go to Overview',
				action: () => goto('/cloud/workloads')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.PODS,
				description: 'Go to Pods',
				action: () => goto('/cloud/workloads/pods')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.SERVICES,
				description: 'Go to Services',
				action: () => goto('/cloud/workloads/services')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.DEPLOYMENTS,
				description: 'Go to Deployments',
				action: () => goto('/cloud/workloads/deployments')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.STATEFULSETS,
				description: 'Go to StatefulSets',
				action: () => goto('/cloud/workloads/statefulsets')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.DAEMONSETS,
				description: 'Go to DaemonSets',
				action: () => goto('/cloud/workloads/daemonsets')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.JOBS,
				description: 'Go to Jobs',
				action: () => goto('/cloud/workloads/jobs')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.CRONJOBS,
				description: 'Go to CronJobs',
				action: () => goto('/cloud/workloads/cronjobs')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.CONFIGMAPS,
				description: 'Go to ConfigMaps',
				action: () => goto('/cloud/configmaps')
			},
			{
				key: RESOURCE_TYPE_SHORTCUTS.SECRETS,
				description: 'Go to Secrets',
				action: () => goto('/cloud/secrets')
			}
		],
		{ enabled: $cloudStore.connection.isConnected }
	);
	
	function handleKeydown(event: KeyboardEvent) {
		resourceTypeShortcuts.handleKeydown(event);
	}
	
	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="space-y-6">
	<!-- Namespace Selector -->
	<div class="pb-4 border-b border-border group-data-[collapsible=icon]:hidden">
		<NamespaceSelector />
	</div>

	<!-- Navigation Groups -->
	<nav class="space-y-6">
		{#each navGroups as group}
		<div>
			<h3 class="px-3 mb-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider group-data-[collapsible=icon]:hidden">
				{group.title}
			</h3>
			<div class="space-y-1">
				{#each group.items as item}
					{@const isActive = isActiveTab(item.route)}
					<SidebarMenuButton
						size="default"
						isActive={isActive}
						tooltipContent={item.label}
						onclick={() => handleItemClick(item.route)}
						class="items-start"
					>
						<span class="text-lg flex-shrink-0">{item.icon}</span>
						<div class="flex-1 text-left min-w-0 group-data-[collapsible=icon]:hidden">
							<div class="font-medium text-sm leading-tight">{item.label}</div>
							<p class="text-xs opacity-70 mt-0.5 leading-relaxed">{item.description}</p>
						</div>
					</SidebarMenuButton>
				{/each}
			</div>
		</div>
		{#if group !== navGroups[navGroups.length - 1]}
			<div class="border-t border-border group-data-[collapsible=icon]:hidden"></div>
		{/if}
		{/each}
	</nav>
</div>

