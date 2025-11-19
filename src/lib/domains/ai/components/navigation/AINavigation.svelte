<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Sparkles, MessageSquare, Settings, Database, FileText } from 'lucide-svelte';
	import { cn } from '$lib/utils';

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
	<div class="px-3 py-2 mb-2">
		<h2 class="text-lg font-semibold flex items-center gap-2">
			<Sparkles class="h-5 w-5" />
			AI
		</h2>
	</div>
	{#each navItems as item}
		<Button
			variant={isActive(item.url) ? 'secondary' : 'ghost'}
			class={cn(
				'w-full justify-start',
				isActive(item.url) && 'bg-secondary'
			)}
			onclick={() => goto(item.url)}
		>
			<svelte:component this={item.icon} class="h-4 w-4 mr-2" />
			<div class="flex flex-col items-start">
				<span>{item.title}</span>
				<span class="text-xs text-muted-foreground">{item.description}</span>
			</div>
		</Button>
	{/each}
</nav>

