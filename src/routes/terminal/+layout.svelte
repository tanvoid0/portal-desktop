<!--
	Terminal Layout - Provides tab navigation for all terminal sub-pages
-->

<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import type { Snippet } from 'svelte';

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();

	// Get current tab from URL
	let currentTab = $derived(page.url.pathname === '/terminal' ? 'global' : page.url.pathname.split('/').pop() || 'global');

	function handleTabChange(tab: string) {
		if (tab === 'global') {
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
	<Tabs value={currentTab} onValueChange={handleTabChange} class="h-full">
		<TabsList class="grid w-full grid-cols-3">
			<TabsTrigger value="global">Global Terminal</TabsTrigger>
			<TabsTrigger value="project">Project Terminals</TabsTrigger>
			<TabsTrigger value="containerized">Containerized</TabsTrigger>
		</TabsList>

		<TabsContent value="global" class="h-full">
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
