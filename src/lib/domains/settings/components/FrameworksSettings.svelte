<!--
	Frameworks Settings - Manage frameworks with intelligent recommendations
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
	import { 
		Package, 
		Plus, 
		Edit, 
		Trash2, 
		Search, 
		RefreshCw, 
		Loader2, 
		Sparkles,
		X,
		TrendingUp,
		Star
	} from '@lucide/svelte';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import { logger } from '$lib/domains/shared';
	import { ideService, type Framework, type FrameworkGroup, type SuggestedFramework } from '$lib/domains/ide';
	import { suggestionEngine, learningService } from '@/lib/domains/learning';
	import Icon from '@iconify/svelte';

	const log = logger.createScoped('FrameworksSettings');

	// State
	let frameworks = $state<Framework[]>([]);
	let suggestedFrameworkGroups = $state<FrameworkGroup[]>([]);
	let recommendedFrameworks = $state<string[]>([]); // Learned recommendations
	let isLoading = $state(false);
	let searchQuery = $state('');
	let showAddDialog = $state(false);
	let editingFramework = $state<Framework | null>(null);
	
	// Form state
	let frameworkName = $state('');
	let frameworkIcon = $state('');
	let frameworkIconType = $state<'devicon' | 'file'>('devicon');
	let frameworkCategory = $state('Custom');

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		isLoading = true;
		try {
			await Promise.all([
				loadFrameworks(),
				loadSuggestedFrameworks(),
				loadRecommendedFrameworks()
			]);
		} catch (error) {
			log.error('Failed to load frameworks data', error);
			toast.error('Failed to load frameworks');
		} finally {
			isLoading = false;
		}
	}

	async function loadFrameworks() {
		try {
			frameworks = await ideService.getAllFrameworks();
			log.info('Frameworks loaded', { count: frameworks.length });
		} catch (error) {
			log.error('Failed to load frameworks', error);
			toast.error('Failed to load frameworks');
		}
	}

	async function loadSuggestedFrameworks() {
		try {
			suggestedFrameworkGroups = await ideService.getSuggestedFrameworks();
			log.info('Suggested frameworks loaded', { count: suggestedFrameworkGroups.length });
		} catch (error) {
			log.error('Failed to load suggested frameworks', error);
		}
	}

	async function loadRecommendedFrameworks() {
		try {
			// Get learned framework patterns from the learning system
			const suggestions = await suggestionEngine.getContextualSuggestions('framework');
			
			// Extract unique framework names from suggestions (sorted by frequency * success_rate)
			const frameworkScores = new Map<string, number>();
			for (const suggestion of suggestions) {
				if (suggestion.pattern_data && typeof suggestion.pattern_data === 'object') {
					const data = suggestion.pattern_data as Record<string, unknown>;
					if (data.framework && typeof data.framework === 'string') {
						const score = suggestion.frequency * suggestion.success_rate;
						const currentScore = frameworkScores.get(data.framework) || 0;
						frameworkScores.set(data.framework, currentScore + score);
					}
				}
			}
			
			// Sort by score and get top frameworks
			const sorted = Array.from(frameworkScores.entries())
				.sort((a, b) => b[1] - a[1])
				.map(([name]) => name)
				.slice(0, 15); // Top 15 recommendations
			
			recommendedFrameworks = sorted;
			log.info('Recommended frameworks loaded', { count: recommendedFrameworks.length });
		} catch (error) {
			log.error('Failed to load recommended frameworks', error);
		}
	}

	function startAddingFramework(suggestedName?: string) {
		frameworkName = suggestedName || '';
		frameworkIcon = '';
		frameworkIconType = 'devicon';
		frameworkCategory = 'Custom';
		editingFramework = null;
		showAddDialog = true;
	}

	function startEditingFramework(framework: Framework) {
		frameworkName = framework.name;
		frameworkIcon = framework.icon;
		frameworkIconType = framework.icon_type;
		frameworkCategory = framework.category;
		editingFramework = framework;
		showAddDialog = true;
	}

	async function saveFramework() {
		if (!frameworkName.trim()) {
			toast.error('Framework name is required');
			return;
		}

		try {
			if (editingFramework) {
				await ideService.updateFramework(
					editingFramework.id,
					frameworkName.trim(),
					frameworkIcon.trim() || 'logos:code',
					frameworkIconType,
					frameworkCategory.trim() || 'Custom'
				);
				toast.success('Framework updated successfully');
			} else {
				await ideService.createFramework(
					frameworkName.trim(),
					frameworkIcon.trim() || 'logos:code',
					frameworkIconType,
					frameworkCategory.trim() || 'Custom'
				);
				toast.success('Framework added successfully');
				
				// Learn from adding framework
				try {
					await learningService.learnPattern({
						pattern_type: 'framework',
						pattern_data: {
							framework: frameworkName.trim(),
							category: frameworkCategory.trim()
						},
						context: `category_${frameworkCategory.toLowerCase().replace(/\s+/g, '_')}`
					});
				} catch (error) {
					log.warn('Failed to learn framework pattern', error);
				}
			}
			
			showAddDialog = false;
			await loadData();
		} catch (error) {
			log.error('Failed to save framework', error);
			toast.error(error instanceof Error ? error.message : 'Failed to save framework');
		}
	}

	async function deleteFramework(id: number, name: string) {
		if (!confirm(`Are you sure you want to delete the framework "${name}"?`)) {
			return;
		}

		try {
			await ideService.deleteFramework(id);
			toast.success('Framework deleted successfully');
			await loadFrameworks();
		} catch (error) {
			log.error('Failed to delete framework', error);
			toast.error('Failed to delete framework');
		}
	}

	function isFrameworkInList(frameworkName: string): boolean {
		return frameworks.some(f => f.name.toLowerCase() === frameworkName.toLowerCase());
	}

	function getFrameworkFrequency(frameworkName: string): number {
		// This would ideally come from the learning system, but for now return 0
		// In a full implementation, we'd query the learned patterns
		return 0;
	}

	const filteredFrameworks = $derived.by(() => {
		if (!searchQuery.trim()) return frameworks;
		
		const query = searchQuery.toLowerCase();
		return frameworks.filter(f => 
			f.name.toLowerCase().includes(query) ||
			f.category.toLowerCase().includes(query) ||
			f.icon.toLowerCase().includes(query)
		);
	});

	const allSuggestedFrameworks = $derived.by(() => {
		const all: SuggestedFramework[] = [];
		for (const group of suggestedFrameworkGroups) {
			all.push(...group.frameworks);
		}
		return all;
	});

	const suggestedFrameworksNotAdded = $derived.by(() => {
		const all = allSuggestedFrameworks;
		return all.filter((sf: SuggestedFramework) => 
			!frameworks.some(f => f.name.toLowerCase() === sf.name.toLowerCase())
		);
	});
</script>

<div class="space-y-6">
	<!-- Header Actions -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Input
				placeholder="Search frameworks..."
				bind:value={searchQuery}
				class="w-64"
			>
				<Search class="w-4 h-4" />
			</Input>
		</div>
		<Button onclick={() => startAddingFramework()}>
			<Plus class="w-4 h-4 mr-2" />
			Add Framework
		</Button>
	</div>

	{#if isLoading}
		<div class="flex items-center justify-center py-12">
			<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			<span class="ml-2 text-muted-foreground">Loading frameworks...</span>
		</div>
	{:else}
		<Tabs value="your-frameworks" class="space-y-4">
			<TabsList>
				<TabsTrigger value="your-frameworks">
					Your Frameworks ({frameworks.length})
				</TabsTrigger>
			{@const allSuggestedCount = allSuggestedFrameworks.length}
			<TabsTrigger value="suggested">
				Suggested ({allSuggestedCount})
			</TabsTrigger>
				<TabsTrigger value="recommended">
					Recommended ({recommendedFrameworks.length})
					{#if recommendedFrameworks.length > 0}
						<Sparkles class="w-3 h-3 ml-1 text-primary" />
					{/if}
				</TabsTrigger>
			</TabsList>

			<!-- Your Frameworks Tab -->
			<TabsContent value="your-frameworks" class="space-y-4">
				{@const filtered = filteredFrameworks}
			{#if filtered.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-12">
							<Package class="w-12 h-12 text-muted-foreground mb-4" />
							<p class="text-muted-foreground mb-2">
								{#if searchQuery}
									No frameworks match your search
								{:else}
									No frameworks added yet
								{/if}
							</p>
							<Button variant="outline" onclick={() => startAddingFramework()}>
								<Plus class="w-4 h-4 mr-2" />
								Add Your First Framework
							</Button>
						</CardContent>
					</Card>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{#each filtered as framework}
							<Card class="relative">
								<CardHeader>
									<div class="flex items-start justify-between">
										<div class="flex items-center gap-3">
											{#if framework.icon_type === 'devicon'}
												<Icon icon={framework.icon} class="w-8 h-8" />
											{:else}
												<img src={framework.icon} alt={framework.name} class="w-8 h-8" />
											{/if}
											<div>
												<CardTitle class="text-lg">{framework.name}</CardTitle>
												<CardDescription>{framework.category}</CardDescription>
											</div>
										</div>
									</div>
								</CardHeader>
								<CardContent>
									<div class="flex items-center justify-end gap-2 mt-2">
										<Button
											variant="outline"
											size="sm"
											onclick={() => startEditingFramework(framework)}
										>
											<Edit class="w-3 h-3 mr-1" />
											Edit
										</Button>
										<Button
											variant="outline"
											size="sm"
											onclick={() => deleteFramework(framework.id, framework.name)}
											class="text-red-500 hover:text-red-700"
										>
											<Trash2 class="w-3 h-3 mr-1" />
											Delete
										</Button>
									</div>
								</CardContent>
							</Card>
						{/each}
					</div>
				{/if}
			</TabsContent>

			<!-- Suggested Frameworks Tab -->
			<TabsContent value="suggested" class="space-y-4">
				{#if suggestedFrameworkGroups.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-12">
							<Package class="w-12 h-12 text-muted-foreground mb-4" />
							<p class="text-muted-foreground">No suggested frameworks available</p>
						</CardContent>
					</Card>
				{:else}
					{#each suggestedFrameworkGroups as group}
						<Card>
							<CardHeader>
								<CardTitle>{group.category}</CardTitle>
								<CardDescription>Popular frameworks in this category</CardDescription>
							</CardHeader>
							<CardContent>
								<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
									{#each group.frameworks as framework}
										{@const isAdded = isFrameworkInList(framework.name)}
										<div class="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors">
											<div class="flex items-center gap-3 flex-1 min-w-0">
												<Icon icon={framework.icon} class="w-6 h-6 flex-shrink-0" />
												<div class="min-w-0 flex-1">
													<div class="font-medium truncate">{framework.name}</div>
													<div class="text-xs text-muted-foreground">{framework.category}</div>
												</div>
											</div>
											{#if isAdded}
												<Badge variant="secondary" class="ml-2">Added</Badge>
											{:else}
												<Button
													variant="ghost"
													size="sm"
													onclick={() => startAddingFramework(framework.name)}
												>
													<Plus class="w-4 h-4" />
												</Button>
											{/if}
										</div>
									{/each}
								</div>
							</CardContent>
						</Card>
					{/each}
				{/if}
			</TabsContent>

			<!-- Recommended Frameworks Tab (Based on Learning) -->
			<TabsContent value="recommended" class="space-y-4">
				{#if recommendedFrameworks.length === 0 && suggestedFrameworkGroups.length > 0 && suggestedFrameworkGroups[0]?.category.includes('Recommended')}
					{@const recommendedGroup = suggestedFrameworkGroups.find(g => g.category.includes('Recommended'))}
					{#if recommendedGroup}
						<Card>
							<CardHeader>
								<div class="flex items-center gap-2">
									<Sparkles class="w-5 h-5 text-primary" />
									<CardTitle>Recommended Based on Your Usage</CardTitle>
								</div>
								<CardDescription>
									These frameworks are suggested based on patterns learned from your projects
								</CardDescription>
							</CardHeader>
							<CardContent>
								<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
									{#each recommendedGroup.frameworks as framework}
										{@const isAdded = isFrameworkInList(framework.name)}
										<div class="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors {isAdded ? 'opacity-60' : ''}">
											<div class="flex items-center gap-3 flex-1 min-w-0">
												<Icon icon={framework.icon} class="w-6 h-6 flex-shrink-0" />
												<div class="min-w-0 flex-1">
													<div class="font-medium truncate flex items-center gap-1">
														{framework.name}
														<Badge variant="outline" class="text-xs border-primary text-primary bg-primary/5">
															<TrendingUp class="w-3 h-3 mr-1" />
															Recommended
														</Badge>
													</div>
													<div class="text-xs text-muted-foreground">{framework.category}</div>
												</div>
											</div>
											{#if isAdded}
												<Badge variant="secondary" class="ml-2">Added</Badge>
											{:else}
												<Button
													variant="ghost"
													size="sm"
													onclick={() => startAddingFramework(framework.name)}
												>
													<Plus class="w-4 h-4" />
												</Button>
											{/if}
										</div>
									{/each}
								</div>
							</CardContent>
						</Card>
					{/if}
				{:else if recommendedFrameworks.length === 0}
					<Card>
						<CardContent class="flex flex-col items-center justify-center py-12">
							<Sparkles class="w-12 h-12 text-muted-foreground mb-4" />
							<p class="text-muted-foreground mb-2">No recommendations yet</p>
							<p class="text-sm text-muted-foreground text-center">
								As you create projects, the system will learn your framework preferences and suggest them here
							</p>
						</CardContent>
					</Card>
				{:else}
					<Card>
						<CardHeader>
							<div class="flex items-center gap-2">
								<Sparkles class="w-5 h-5 text-primary" />
								<CardTitle>Recommended Based on Your Usage</CardTitle>
							</div>
							<CardDescription>
								These frameworks are suggested based on patterns learned from your projects
							</CardDescription>
						</CardHeader>
						<CardContent>
							<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
								{#each recommendedFrameworks as frameworkName}
									{@const isAdded = isFrameworkInList(frameworkName)}
									{@const suggestedFramework = allSuggestedFrameworks.find((sf: SuggestedFramework) => 
										sf.name.toLowerCase() === frameworkName.toLowerCase()
									)}
									<div class="flex items-center justify-between p-3 border rounded-lg hover:bg-accent/50 transition-colors {isAdded ? 'opacity-60' : ''}">
										<div class="flex items-center gap-3 flex-1 min-w-0">
											{#if suggestedFramework}
												<Icon icon={suggestedFramework.icon} class="w-6 h-6 flex-shrink-0" />
											{:else}
												<Package class="w-6 h-6 flex-shrink-0 text-muted-foreground" />
											{/if}
											<div class="min-w-0 flex-1">
												<div class="font-medium truncate flex items-center gap-1">
													{frameworkName}
													<Badge variant="outline" class="text-xs border-primary text-primary bg-primary/5">
														<TrendingUp class="w-3 h-3 mr-1" />
														Recommended
													</Badge>
												</div>
												{#if suggestedFramework}
													<div class="text-xs text-muted-foreground">{suggestedFramework.category}</div>
												{/if}
											</div>
										</div>
										{#if isAdded}
											<Badge variant="secondary" class="ml-2">Added</Badge>
										{:else}
											<Button
												variant="ghost"
												size="sm"
												onclick={() => startAddingFramework(frameworkName)}
											>
												<Plus class="w-4 h-4" />
											</Button>
										{/if}
									</div>
								{/each}
							</div>
						</CardContent>
					</Card>
				{/if}
			</TabsContent>
		</Tabs>
	{/if}
</div>

<!-- Add/Edit Framework Dialog -->
<Dialog bind:open={showAddDialog}>
	<DialogContent class="sm:max-w-[500px]">
		<DialogHeader>
			<DialogTitle>
				{editingFramework ? 'Edit Framework' : 'Add Framework'}
			</DialogTitle>
			<DialogDescription>
				{editingFramework 
					? 'Update framework details'
					: 'Add a new framework to your collection'}
			</DialogDescription>
		</DialogHeader>

		<div class="grid gap-4 py-4">
			<div class="space-y-2">
				<Label for="framework-name">Framework Name *</Label>
				<Input
					id="framework-name"
					bind:value={frameworkName}
					placeholder="React, Vue, Express, etc."
				/>
			</div>

			<div class="space-y-2">
				<Label for="framework-category">Category</Label>
				<Input
					id="framework-category"
					bind:value={frameworkCategory}
					placeholder="Frontend, Backend, Full Stack, etc."
				/>
			</div>

			<div class="space-y-2">
				<Label for="framework-icon">Icon</Label>
				<Input
					id="framework-icon"
					bind:value={frameworkIcon}
					placeholder="logos:react (for Devicon) or image URL"
				/>
				<p class="text-xs text-muted-foreground">
					Enter a Devicon name (e.g., "logos:react") or an image URL
				</p>
			</div>

			<div class="space-y-2">
				<Label for="framework-icon-type">Icon Type</Label>
				<select
					id="framework-icon-type"
					bind:value={frameworkIconType}
					class="w-full px-3 py-2 border rounded-md"
				>
					<option value="devicon">Devicon</option>
					<option value="file">Image URL</option>
				</select>
			</div>
		</div>

		<DialogFooter>
			<Button variant="outline" onclick={() => showAddDialog = false}>
				Cancel
			</Button>
			<Button onclick={saveFramework}>
				{editingFramework ? 'Update' : 'Add'} Framework
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<style>
	select {
		background-color: hsl(var(--background));
		color: hsl(var(--foreground));
		border-color: hsl(var(--border));
	}
</style>

