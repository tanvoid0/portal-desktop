<!--
	Terminal Layout - Simplified with AI Terminal as primary interface
-->

<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import type { Snippet } from 'svelte';

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();

	// Get current tab from URL
	let currentTab = $derived(() => {
		const path = page.url.pathname;
		if (path === '/terminal' || path === '/terminal/ai') return 'ai';
		if (path === '/terminal/project') return 'project';
		if (path === '/terminal/containerized') return 'containerized';
		return 'ai';
	});

	function handleTabChange(tab: string) {
		if (tab === 'ai') {
			goto('/terminal');
		} else {
			goto(`/terminal/${tab}`);
		}
	}
</script>

<svelte:head>
	<title>Terminal - Portal Desktop</title>
</svelte:head>

<div class="h-full w-full">
	<Tabs value={currentTab()} onValueChange={handleTabChange} class="h-full">
		<TabsList class="grid w-full grid-cols-3">
			<TabsTrigger value="ai">Terminal</TabsTrigger>
			<TabsTrigger value="project">Project Terminals</TabsTrigger>
			<TabsTrigger value="containerized">Containerized</TabsTrigger>
		</TabsList>

		<TabsContent value="ai" class="h-full">
			{@render children()}
		</TabsContent>

		<TabsContent value="project" class="h-full">
			{@render children()}
		</TabsContent>

		<TabsContent value="containerized" class="h-full">
			{@render children()}
		</TabsContent>
	</Tabs>
</div>
