<!-- AI Layout - Handles sidebar navigation for all AI subpages -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Sidebar as SidebarRoot } from '$lib/components/ui/sidebar';
	import AINavigation from '$lib/domains/ai/components/navigation/AINavigation.svelte';

	let { children }: { children: Snippet<[]> } = $props();
</script>

<div class="flex flex-col h-full w-full min-h-0 overflow-hidden">
	<!-- Main Content Area with Sidebar -->
	<div class="flex flex-1 min-h-0 overflow-hidden w-full">
		<SidebarRoot collapsible="icon">
			<div class="flex h-full flex-col min-h-0 overflow-hidden">
				<div class="p-4 flex-1 overflow-y-auto min-h-0">
				<AINavigation />
			</div>
			</div>
		</SidebarRoot>

		<!-- Page Content -->
		<main class="flex-1 overflow-hidden min-w-0 min-h-0">
			{@render children()}
		</main>
	</div>
</div>

