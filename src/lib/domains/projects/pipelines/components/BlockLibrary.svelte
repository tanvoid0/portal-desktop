<!--
	Block Library - Block browser and selector
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import type { Block } from '../types';
	import { blockLibraryService, blockLibraryStore, filteredBlocks } from '../index';

	interface Props {
		onSelect?: (blockId: string) => void;
		onClose?: () => void;
	}

	let { onSelect, onClose }: Props = $props();

	let searchQuery = $state('');
	let selectedCategory = $state<Block['category'] | null>(null);
	let blocks = $state<Block[]>([]);
	let loading = $state(false);

	$effect(() => {
		blockLibraryStore.setSearchQuery(searchQuery);
		blockLibraryStore.setSelectedCategory(selectedCategory);
	});

	onMount(async () => {
		loading = true;
		await blockLibraryStore.loadBlocks();
		loading = false;
		
		// Subscribe to filtered blocks
		const unsubscribe = filteredBlocks.subscribe((filtered) => {
			blocks = filtered;
		});
		
		return () => unsubscribe();
	});
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
	<Card class="w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
		<CardHeader>
			<div class="flex items-center justify-between">
				<CardTitle>Block Library</CardTitle>
				<Button variant="ghost" onclick={onClose}>Close</Button>
			</div>
		</CardHeader>
		<CardContent class="flex-1 overflow-hidden flex flex-col">
			<!-- Search and Filters -->
			<div class="space-y-2 mb-4">
				<Input
					bind:value={searchQuery}
					placeholder="Search blocks..."
					class="w-full"
				/>
				<div class="flex gap-2 flex-wrap">
					<Button
						variant={selectedCategory === null ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = null)}
					>
						All
					</Button>
					<Button
						variant={selectedCategory === 'build' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'build')}
					>
						Build
					</Button>
					<Button
						variant={selectedCategory === 'test' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'test')}
					>
						Test
					</Button>
					<Button
						variant={selectedCategory === 'deploy' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'deploy')}
					>
						Deploy
					</Button>
					<Button
						variant={selectedCategory === 'utility' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (selectedCategory = 'utility')}
					>
						Utility
					</Button>
				</div>
			</div>

			<!-- Block List -->
			<div class="flex-1 overflow-y-auto space-y-2">
				{#if loading}
					<p class="text-center text-muted-foreground py-8">Loading blocks...</p>
				{:else if blocks.length === 0}
					<p class="text-center text-muted-foreground py-8">No blocks found</p>
				{:else}
					{#each blocks as block (block.id)}
						<div
							class="p-4 border rounded-md hover:bg-accent cursor-pointer transition-colors"
							onclick={() => onSelect?.(block.id)}
						>
							<div class="flex items-start justify-between">
								<div class="flex-1">
									<h3 class="font-medium">{block.name}</h3>
									<p class="text-sm text-muted-foreground mt-1">
										{block.description}
									</p>
									<div class="flex gap-2 mt-2">
										<span
											class="px-2 py-1 text-xs rounded bg-primary/10 text-primary"
										>
											{block.category}
										</span>
										{#each block.tags.slice(0, 3) as tag}
											<span
												class="px-2 py-1 text-xs rounded bg-muted text-muted-foreground"
											>
												{tag}
											</span>
										{/each}
									</div>
								</div>
								<Button size="sm" onclick={() => onSelect?.(block.id)}>
									Add
								</Button>
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</CardContent>
	</Card>
</div>

