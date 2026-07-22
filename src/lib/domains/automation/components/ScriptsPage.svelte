<!--
	Scripts Management Page
	Dedicated page for viewing and managing reusable scripts
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Plus,
		Search,
		Upload,
		Download,
		Edit,
		Trash2,
		FileCode,
		Terminal,
		Copy,
		Eye,
	} from '@lucide/svelte';
	import type { Block } from '$lib/domains/projects/pipelines';
	import { blockLibraryService, blockLibraryStore, blocks as blocksStore } from '$lib/domains/projects/pipelines';
	import { toast } from '$lib/utils/toast';
	import { confirmAction } from '$lib/utils/confirm';
	import { setBreadcrumbs } from '$lib/domains/shared/stores/breadcrumbStore';
	import { PageHeader, PageLoading, PageError, PageEmpty } from '$lib/components/shell';

	setBreadcrumbs([{ label: 'Automation', href: '/automation' }, { label: 'Scripts', href: '/automation/scripts' }]);

	let scripts = $state<Block[]>([]);
	let loading = $state(false);
	let loadError = $state<string | null>(null);
	let searchQuery = $state('');
	let selectedCategory = $state<Block['category'] | null>(null);

	onMount(async () => {
		await loadScripts();
	});

	async function loadScripts() {
		loading = true;
		loadError = null;
		try {
			await blockLibraryStore.loadBlocks();
		} catch (error) {
			console.error('Failed to load scripts', error);
			loadError = error instanceof Error ? error.message : 'Failed to load scripts';
			toast.error('Failed to load scripts');
		} finally {
			loading = false;
		}
	}

	// Subscribe to blocks store and filter for scripts
	$effect(() => {
		const unsubscribe = blocksStore.subscribe((b) => {
			// Filter to show script-type blocks or utility category
			scripts = b.filter(
				(block) => block.executionType === 'script' || block.category === 'utility'
			);
		});
		return unsubscribe;
	});

	function handleCreateScript() {
		goto('/scripts/new');
	}

	function handleViewScript(script: Block) {
		goto(`/scripts/${script.id}`);
	}

	function handleEditScript(script: Block) {
		goto(`/scripts/${script.id}?edit=true`);
	}

	async function handleDeleteScript(scriptId: string) {
		const confirmed = await confirmAction(
			'Are you sure you want to delete this script?',
			'Delete script',
		);
		if (!confirmed) return;
		try {
			await blockLibraryService.deleteBlock(scriptId);
			toast.success('Script deleted successfully');
			await loadScripts();
		} catch (error) {
			console.error('Failed to delete script', error);
			toast.error('Failed to delete script');
		}
	}

	async function handleExportScript(script: Block) {
		try {
			const dataStr = JSON.stringify(script, null, 2);
			const dataBlob = new Blob([dataStr], { type: 'application/json' });
			const url = URL.createObjectURL(dataBlob);
			const link = document.createElement('a');
			link.href = url;
			link.download = `${script.name.toLowerCase().replace(/\s+/g, '-')}.block.json`;
			link.click();
			URL.revokeObjectURL(url);
			toast.success('Script exported successfully');
		} catch (error) {
			console.error('Failed to export script', error);
			toast.error('Failed to export script');
		}
	}

	function handleCopyCommand(command: string) {
		navigator.clipboard.writeText(command);
		toast.success('Command copied to clipboard');
	}

	function handleImportJson() {
		// Navigate to new script page with import flag
		goto('/scripts/new?import=true');
	}

	const filteredScripts = $derived.by(() => {
		let filtered = scripts;

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(
				(script) =>
					script.name.toLowerCase().includes(query) ||
					script.description.toLowerCase().includes(query) ||
					(script.tags || []).some((tag) => tag.toLowerCase().includes(query))
			);
		}

		if (selectedCategory) {
			filtered = filtered.filter((script) => script.category === selectedCategory);
		}

		return filtered;
	});
</script>

<svelte:head>
	<title>Scripts - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto p-6 space-y-6">
	<PageHeader title="Scripts" description="Create, manage, and run reusable scripts for your workflows">
		{#snippet actions()}
			<Button variant="outline" onclick={handleImportJson}>
				<Upload class="h-4 w-4 mr-2" />
				Import JSON
			</Button>
			<Button onclick={handleCreateScript}>
				<Plus class="h-4 w-4 mr-2" />
				New Script
			</Button>
		{/snippet}
	</PageHeader>

	<!-- Search and Filters -->
	<div class="flex gap-4">
		<div class="flex-1 relative">
			<Search class="h-4 w-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground" />
			<Input bind:value={searchQuery} placeholder="Search scripts..." class="pl-9" />
		</div>
		<div class="flex gap-2">
			<Button
				variant={selectedCategory === null ? 'default' : 'outline'}
				size="sm"
				onclick={() => (selectedCategory = null)}
			>
				All
			</Button>
			<Button
				variant={selectedCategory === 'utility' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (selectedCategory = 'utility')}
			>
				Utility
			</Button>
			<Button
				variant={selectedCategory === 'build' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (selectedCategory = 'build')}
			>
				Build
			</Button>
			<Button
				variant={selectedCategory === 'deploy' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (selectedCategory = 'deploy')}
			>
				Deploy
			</Button>
		</div>
	</div>

	<!-- Scripts Grid -->
	{#if loading}
		<PageLoading message="Loading scripts..." />
	{:else if loadError}
		<PageError title="Failed to load scripts" message={loadError} onRetry={loadScripts} />
	{:else if filteredScripts.length === 0}
		<PageEmpty
			title="No scripts found"
			description="Create your first script to get started."
			filteredDescription="Try adjusting your search or filters."
			isFiltered={Boolean(searchQuery.trim() || selectedCategory)}
			icon={FileCode}
			actionLabel="Create Your First Script"
			onAction={handleCreateScript}
		/>
	{:else}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each filteredScripts as script (script.id)}
				<Card class="hover:border-primary/50 transition-colors cursor-pointer" onclick={() => handleViewScript(script)}>
					<CardHeader class="pb-3">
						<div class="flex items-start justify-between">
							<div class="flex-1 min-w-0">
								<CardTitle class="text-lg truncate">{script.name}</CardTitle>
								<CardDescription class="mt-1 line-clamp-2">{script.description}</CardDescription>
							</div>
							<Badge variant="outline" class="ml-2 shrink-0">{script.category}</Badge>
						</div>
					</CardHeader>
					<CardContent class="space-y-4">
						<!-- Command Preview -->
						<div class="bg-muted rounded-md p-2 relative group" onclick={(e) => e.stopPropagation()}>
							<code class="text-xs font-mono text-muted-foreground line-clamp-2">{script.command}</code>
							<Button
								variant="ghost"
								size="sm"
								class="absolute top-1 right-1 h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
								onclick={(e) => { e.stopPropagation(); handleCopyCommand(script.command); }}
							>
								<Copy class="h-3 w-3" />
							</Button>
						</div>

						<!-- Info -->
						<div class="flex items-center gap-4 text-sm text-muted-foreground">
							<span class="flex items-center gap-1">
								<Terminal class="h-3 w-3" />
								{script.executionType}
							</span>
							{#if script.parameters.length > 0}
								<span>{script.parameters.length} params</span>
							{/if}
							<span>v{script.version}</span>
						</div>

						<!-- Tags -->
						{#if script.tags && script.tags.length > 0}
							<div class="flex flex-wrap gap-1">
								{#each script.tags.slice(0, 4) as tag}
									<Badge variant="secondary" class="text-xs">{tag}</Badge>
								{/each}
								{#if script.tags.length > 4}
									<Badge variant="secondary" class="text-xs">+{script.tags.length - 4}</Badge>
								{/if}
							</div>
						{/if}

						<!-- Actions -->
						<div class="divider-edge-t divider-edge-full flex gap-2 pt-2" onclick={(e) => e.stopPropagation()}>
							<Button size="sm" variant="outline" onclick={() => handleViewScript(script)} class="flex-1">
								<Eye class="h-3 w-3 mr-1" />
								View
							</Button>
							<Button size="sm" variant="outline" onclick={() => handleEditScript(script)}>
								<Edit class="h-3 w-3" />
							</Button>
							<Button size="sm" variant="outline" onclick={() => handleExportScript(script)}>
								<Download class="h-3 w-3" />
							</Button>
							<Button size="sm" variant="destructive" onclick={() => handleDeleteScript(script.id)}>
								<Trash2 class="h-3 w-3" />
							</Button>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>
