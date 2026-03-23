<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { MenuButton as SidebarMenuButton } from '$lib/components/ui/sidebar';
	import { Sparkles, MessageSquare, Settings, Database, FileText } from 'lucide-svelte';

	const navItems = [
		{
			title: 'Chat',
			url: '/ai/chat',
			icon: MessageSquare,
			description: 'Chat with AI'
		},
		{
			title: 'Providers',
			url: '/ai/providers',
			icon: Settings,
			description: 'Configure AI providers'
		},
		{
			title: 'Training Data',
			url: '/ai/training',
			icon: Database,
			description: 'Manage training data'
		},
		{
			title: 'Logs',
			url: '/ai/logs',
			icon: FileText,
			description: 'View AI interaction logs'
		}
	];

	function isActive(url: string): boolean {
		return $page.url.pathname === url || $page.url.pathname.startsWith(url + '/');
	}
</script>

<nav class="space-y-1">
	<div class="px-3 py-2 mb-3 group-data-[collapsible=icon]:hidden">
		<h2 class="text-lg font-semibold flex items-center gap-2 text-foreground">
			<Sparkles class="h-5 w-5 text-primary" />
			AI
		</h2>
	</div>
	{#each navItems as item}
		<SidebarMenuButton
			size="default"
			isActive={isActive(item.url)}
			tooltipContent={item.title}
			onclick={() => goto(item.url)}
			class="items-start"
		>
			<svelte:component this={item.icon} class="h-4 w-4 flex-shrink-0" />
			<div class="flex flex-col items-start gap-0.5 min-w-0 group-data-[collapsible=icon]:hidden">
				<span class="text-sm font-medium">{item.title}</span>
				<span class="text-xs opacity-70">{item.description}</span>
			</div>
		</SidebarMenuButton>
	{/each}
</nav>

