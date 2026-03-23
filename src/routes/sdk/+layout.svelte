<!--
	SDK Layout - Sidebar + Main Container
	Provides consistent layout for all SDK pages
-->

<script lang="ts">
	import type { Snippet } from 'svelte';
	import SDKSidebar from '$lib/domains/sdk/components/SDKSidebar.svelte';
	import { Sidebar as SidebarRoot } from '$lib/components/ui/sidebar';
	import { page } from '$app/stores';

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();
	
	// Get current path for sidebar selection
	let currentPath = $derived($page.url.pathname);
</script>

<div class="flex h-full w-full min-h-0 overflow-hidden">
	<!-- Sidebar -->
	<SidebarRoot collapsible="icon">
		<div class="h-full min-h-0 flex flex-col">
			<SDKSidebar selectedSDK={currentPath.split('/').pop() || undefined} />
		</div>
	</SidebarRoot>
	
	<!-- Main Content Container -->
	<main class="flex-1 overflow-y-auto min-w-0 min-h-0 bg-background">
		{@render children()}
	</main>
</div>
