<!-- AI Layout - Handles sidebar navigation for all AI subpages -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import AINavigation from '$lib/domains/ai/components/navigation/AINavigation.svelte';
	import { Card } from '$lib/components/ui/card';
	
	let { children }: { children: Snippet<[]> } = $props();
</script>

<div class="flex flex-col h-full w-full min-h-0 overflow-hidden">
	<!-- Main Content Area with Sidebar -->
	<div class="flex flex-1 min-h-0 overflow-hidden w-full">
		<!-- Sidebar Navigation -->
		<aside class="w-64 border-r bg-background flex-shrink-0 overflow-y-auto min-w-0">
			<div class="p-4">
				<Card class="p-3">
					<AINavigation />
				</Card>
			</div>
		</aside>

		<!-- Page Content -->
		<main class="flex-1 overflow-y-auto min-w-0 min-h-0">
			{@render children()}
		</main>
	</div>
</div>

