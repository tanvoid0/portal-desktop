<!--
	IDE Settings - Configure integrated development environments
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Laptop, Plus, Edit, Trash2, Search, RefreshCw, Loader2, Code2, Code } from '@lucide/svelte';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import { logger } from '$lib/domains/shared';
	import { ideService, type IdeConfig, type FrameworkIdeMapping } from '$lib/domains/ide';
	import { Separator } from '$lib/components/ui/separator';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Link2, ArrowRight, Upload, X } from '@lucide/svelte';
	import Select from '@/lib/components/ui/select.svelte';
	import Icon from '@iconify/svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import type { Framework } from '$lib/domains/ide';

	/**
	 * Normalize Windows paths by removing the extended-length path prefix (\\?\)
	 */
	function normalizePath(path: string): string {
		if (path.startsWith('\\\\?\\')) {
			return path.slice(4);
		}
		return path;
	}

	function getIdeIcon(ideName: string, executable: string): typeof Code2 {
		const name = ideName.toLowerCase();
		const exe = executable.toLowerCase();
		
		if (name.includes('code') || exe.includes('code')) return Code2;
		if (name.includes('idea') || exe.includes('idea')) return Code;
		if (name.includes('pycharm') || exe.includes('pycharm')) return Code;
		if (name.includes('webstorm') || exe.includes('webstorm')) return Code;
		if (name.includes('clion') || exe.includes('clion')) return Code;
		if (name.includes('goland') || exe.includes('goland')) return Code;
		if (name.includes('phpstorm') || exe.includes('phpstorm')) return Code;
		if (name.includes('rubymine') || exe.includes('rubymine')) return Code;
		if (name.includes('rider') || exe.includes('rider')) return Code;
		if (name.includes('studio') || exe.includes('studio')) return Code;
		if (name.includes('sublime') || exe.includes('sublime')) return Code;
		if (name.includes('vim') || exe.includes('vim')) return Code;
		if (name.includes('neovim') || exe.includes('nvim')) return Code;
		if (name.includes('emacs') || exe.includes('emacs')) return Code;
		
		return Laptop;
	}


	let ides = $state<IdeConfig[]>([]);
	let isLoadingIdes = $state(false);
	let isDetectingIdes = $state(false);
	let showModal = $state(false);
	let editingIde = $state<IdeConfig | null>(null);
	let ideName = $state('');
	let ideExecutable = $state('');
	let detectedIdes = $state<string[]>([]);
	
	// Framework IDE Mappings state
	let mappings = $state<FrameworkIdeMapping[]>([]);
	let isLoadingMappings = $state(false);
	let showMappingModal = $state(false);
	let editingMapping = $state<FrameworkIdeMapping | null>(null);
	let mappingFramework = $state('');
	let mappingIdeId = $state<string>('');

	// Framework groups loaded from backend (suggested)
	let frameworkGroups = $state<import('$lib/domains/ide').FrameworkGroup[]>([]);
	// User-defined frameworks from database
	let frameworks = $state<Framework[]>([]);
	let isLoadingFrameworks = $state(false);
	let showFrameworkModal = $state(false);
	let editingFramework = $state<Framework | null>(null);
	let frameworkName = $state('');
	let frameworkIcon = $state('');
	let frameworkIconType = $state<'devicon' | 'file'>('devicon');
	let frameworkCategory = $state('Custom');
	let iconSearchQuery = $state('');
	let iconSearchResults = $state<string[]>([]);

	function quickAddFramework(framework: string) {
		mappingFramework = framework;
		mappingIdeId = '';
		editingMapping = null;
		showMappingModal = true;
	}

	onMount(async () => {
		await Promise.all([loadIdes(), loadMappings(), loadSuggestedFrameworks(), loadFrameworks()]);
	});

	async function loadSuggestedFrameworks() {
		try {
			isLoadingFrameworks = true;
			frameworkGroups = await ideService.getSuggestedFrameworks();
			logger.info('Suggested frameworks loaded', { context: 'IdeSettings', count: frameworkGroups.length });
		} catch (error) {
			logger.error('Failed to load suggested frameworks', { context: 'IdeSettings', error });
			// Don't show error toast as this is not critical
		} finally {
			isLoadingFrameworks = false;
		}
	}

	async function loadFrameworks() {
		try {
			frameworks = await ideService.getAllFrameworks();
			logger.info('Frameworks loaded', { context: 'IdeSettings', count: frameworks.length });
		} catch (error) {
			toast.error('Failed to load frameworks', error);
		}
	}

	function startAddingFramework() {
		frameworkName = '';
		frameworkIcon = '';
		frameworkIconType = 'devicon';
		frameworkCategory = 'Custom';
		iconSearchQuery = '';
		iconSearchResults = [];
		editingFramework = null;
		showFrameworkModal = true;
	}

	function startEditingFramework(framework: Framework) {
		frameworkName = framework.name;
		frameworkIcon = framework.icon;
		frameworkIconType = framework.icon_type;
		frameworkCategory = framework.category;
		iconSearchQuery = '';
		iconSearchResults = [];
		editingFramework = framework;
		showFrameworkModal = true;
	}

	function closeFrameworkModal() {
		showFrameworkModal = false;
		frameworkName = '';
		frameworkIcon = '';
		frameworkIconType = 'devicon';
		frameworkCategory = 'Custom';
		iconSearchQuery = '';
		iconSearchResults = [];
		editingFramework = null;
	}

	async function searchIcons() {
		if (!iconSearchQuery.trim()) {
			iconSearchResults = [];
			return;
		}

		// Simple search - you can enhance this with actual DevIcon API or icon set
		const commonIcons = [
			'logos:react',
			'logos:vue',
			'logos:angular-icon',
			'logos:svelte-icon',
			'logos:nextjs-icon',
			'logos:nodejs-icon',
			'logos:python',
			'logos:java',
			'logos:go',
			'logos:rust',
			'logos:php',
			'logos:ruby',
			'logos:django-icon',
			'logos:flask',
			'logos:laravel',
			'logos:rails',
			'logos:spring-icon',
			'logos:express',
			'logos:fastify-icon',
			'logos:nestjs',
			'logos:nuxt-icon',
			'logos:remix-icon',
			'logos:gatsby',
			'logos:quarkus-icon',
			'logos:phoenix',
			'logos:elixir',
			'logos:sinatra',
			'logos:symfony',
			'logos:fastapi-icon'
		];

		const query = iconSearchQuery.toLowerCase();
		iconSearchResults = commonIcons.filter(icon => 
			icon.includes(query) || icon.replace('logos:', '').includes(query)
		).slice(0, 12);
	}

	async function selectIconFile() {
		try {
			const file = await open({
				multiple: false,
				filters: [{
					name: 'Images',
					extensions: ['png', 'jpg', 'jpeg', 'svg', 'ico', 'webp']
				}]
			});

			if (file && typeof file === 'string') {
				frameworkIcon = file;
				frameworkIconType = 'file';
			}
		} catch (error) {
			toast.error('Failed to select icon file', error);
		}
	}

	async function saveFramework() {
		if (!frameworkName.trim()) {
			toast.error('Framework name is required');
			return;
		}

		if (!frameworkIcon.trim()) {
			toast.error('Framework icon is required');
			return;
		}

		try {
			if (editingFramework) {
				await ideService.updateFramework(
					editingFramework.id,
					frameworkName,
					frameworkIcon,
					frameworkIconType,
					frameworkCategory
				);
				toast.success('Framework updated successfully');
			} else {
				await ideService.createFramework(
					frameworkName,
					frameworkIcon,
					frameworkIconType,
					frameworkCategory
				);
				toast.success('Framework created successfully');
			}

			await loadFrameworks();
			closeFrameworkModal();
		} catch (error: any) {
			toast.error('Failed to save framework', error);
		}
	}

	async function deleteFramework(framework: Framework) {
		if (!confirm(`Are you sure you want to delete "${framework.name}"?`)) {
			return;
		}

		try {
			await ideService.deleteFramework(framework.id);
			toast.success('Framework deleted successfully');
			await loadFrameworks();
		} catch (error: any) {
			toast.error('Failed to delete framework', error);
		}
	}

	function getAllFrameworks(): Array<{name: string; icon: string; category: string; isCustom: boolean}> {
		const suggested: Array<{name: string; icon: string; category: string; isCustom: boolean}> = [];
		
		frameworkGroups.forEach(group => {
			group.frameworks.forEach(fw => {
				suggested.push({ ...fw, isCustom: false });
			});
		});

		const userFrameworks = frameworks.map(fw => ({
			name: fw.name,
			icon: fw.icon,
			category: fw.category,
			isCustom: true
		}));

		return [...suggested, ...userFrameworks];
	}

	async function loadIdes() {
		try {
			isLoadingIdes = true;
			ides = await ideService.getAllIdes();
			logger.info('IDEs loaded', { context: 'IdeSettings', count: ides.length });
		} catch (error) {
			toast.error('Failed to load IDEs', error);
		} finally {
			isLoadingIdes = false;
		}
	}

	async function detectIdes() {
		try {
			isDetectingIdes = true;
			detectedIdes = await ideService.detectInstalledIdes();
			logger.info('IDEs detected', { context: 'IdeSettings', count: detectedIdes.length });
			
			if (detectedIdes.length === 0) {
				toast.info('No IDEs detected automatically');
			}
		} catch (error) {
			toast.error('Failed to detect IDEs', error);
		} finally {
			isDetectingIdes = false;
		}
	}

	function startAddingIde() {
		ideName = '';
		ideExecutable = '';
		editingIde = null;
		showModal = true;
	}

	function startEditingIde(ide: IdeConfig) {
		ideName = ide.name;
		ideExecutable = normalizePath(ide.executable);
		editingIde = ide;
		showModal = true;
	}

	function closeModal() {
		showModal = false;
		ideName = '';
		ideExecutable = '';
		editingIde = null;
	}

	async function saveIde() {
		if (!ideName.trim() || !ideExecutable.trim()) {
			toast.error('IDE name and executable path are required');
			return;
		}

		try {
			if (editingIde && editingIde.id) {
				await ideService.updateIde(editingIde.id, ideName, ideExecutable);
				toast.success('IDE updated successfully');
			} else {
				await ideService.addIde(ideName, ideExecutable);
				toast.success('IDE added successfully');
			}

			await loadIdes();
			closeModal();
		} catch (error: any) {
			toast.error('Failed to save IDE', error);
		}
	}

	async function deleteIde(ide: IdeConfig) {
		if (!confirm(`Are you sure you want to delete "${ide.name}"?`)) {
			return;
		}

		try {
			if (ide.id) {
				await ideService.deleteIde(ide.id);
				toast.success('IDE deleted successfully');
				await loadIdes();
				await loadMappings(); // Reload mappings in case any were deleted
			}
		} catch (error: any) {
			toast.error('Failed to delete IDE', error);
		}
	}

	async function setDefaultIde(ide: IdeConfig) {
		try {
			if (ide.id) {
				await ideService.setDefaultIde(ide.id);
				toast.success('Default IDE updated');
				await loadIdes();
			}
		} catch (error: any) {
			toast.error('Failed to set default IDE', error);
		}
	}

	async function loadMappings() {
		try {
			isLoadingMappings = true;
			mappings = await ideService.getAllFrameworkIdeMappings();
			logger.info('Framework IDE mappings loaded', { context: 'IdeSettings', count: mappings.length });
		} catch (error) {
			toast.error('Failed to load framework IDE mappings', error);
		} finally {
			isLoadingMappings = false;
		}
	}

	function startAddingMapping() {
		mappingFramework = '';
		mappingIdeId = '';
		editingMapping = null;
		showMappingModal = true;
	}

	function startEditingMapping(mapping: FrameworkIdeMapping) {
		mappingFramework = mapping.framework;
		mappingIdeId = String(mapping.ide_id);
		editingMapping = mapping;
		showMappingModal = true;
	}

	function closeMappingModal() {
		showMappingModal = false;
		mappingFramework = '';
		mappingIdeId = '';
		editingMapping = null;
	}

	async function saveMapping() {
		if (!mappingFramework.trim() || !mappingIdeId) {
			toast.error('Framework and IDE are required');
			return;
		}

		try {
			const ideId = parseInt(mappingIdeId);
			if (editingMapping) {
				// For updates, we need to delete and recreate since framework is the key
				await ideService.deleteFrameworkIdeMapping(editingMapping.framework);
			}
			await ideService.setFrameworkIdeMapping(mappingFramework, ideId);
			toast.success(editingMapping ? 'Mapping updated successfully' : 'Mapping created successfully');
			await loadMappings();
			closeMappingModal();
		} catch (error: any) {
			toast.error('Failed to save mapping', error);
		}
	}

	async function deleteMapping(mapping: FrameworkIdeMapping) {
		if (!confirm(`Are you sure you want to delete the mapping for "${mapping.framework}"?`)) {
			return;
		}

		try {
			await ideService.deleteFrameworkIdeMapping(mapping.framework);
			toast.success('Mapping deleted successfully');
			await loadMappings();
		} catch (error: any) {
			toast.error('Failed to delete mapping', error);
		}
	}

	function getIdeName(ideId: number): string {
		const ide = ides.find(i => i.id === ideId);
		return ide ? ide.name : `IDE #${ideId}`;
	}
</script>

<div class="space-y-6">
	<!-- IDE Configuration Section -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-xl font-bold tracking-tight flex items-center gap-2">
				<Code2 class="h-5 w-5" />
				IDE Configuration
			</h2>
			<p class="text-sm text-muted-foreground">
				Configure integrated development environments for project opening
			</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={detectIdes} disabled={isDetectingIdes}>
				{#if isDetectingIdes}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{:else}
					<Search class="h-4 w-4 mr-2" />
				{/if}
				Detect IDEs
			</Button>
			<Button variant="outline" size="sm" onclick={loadIdes} disabled={isLoadingIdes}>
				<RefreshCw class="h-4 w-4 mr-2" />
				Refresh
			</Button>
			<Button variant="outline" size="sm" onclick={startAddingIde}>
				<Plus class="h-4 w-4 mr-2" />
				Add IDE
			</Button>
		</div>
	</div>

	<!-- IDEs List -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Code2 class="h-5 w-5" />
				Configured IDEs
				<Badge variant="outline">{ides.length}</Badge>
			</CardTitle>
			<CardDescription>
				Manage your integrated development environments
			</CardDescription>
		</CardHeader>
		<CardContent>
			{#if isLoadingIdes}
				<div class="flex items-center justify-center py-8">
					<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
					<span class="ml-2 text-muted-foreground">Loading IDEs...</span>
				</div>
			{:else if ides.length === 0}
				<div class="text-center py-8">
					<Code2 class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
					<p class="text-muted-foreground mb-4">No IDEs configured</p>
					<div class="flex gap-2 justify-center">
						<Button onclick={detectIdes} disabled={isDetectingIdes}>
							{#if isDetectingIdes}
								<Loader2 class="h-4 w-4 mr-2 animate-spin" />
							{:else}
								<Search class="h-4 w-4 mr-2" />
							{/if}
							Detect IDEs
						</Button>
						<Button onclick={startAddingIde} variant="outline">
							<Plus class="h-4 w-4 mr-2" />
							Add Manually
						</Button>
					</div>
				</div>
			{:else}
				<div class="space-y-3">
					{#each ides as ide}
						{@const IdeIcon = getIdeIcon(ide.name, ide.executable)}
						<div class="flex items-center justify-between p-4 rounded-md border hover:bg-accent transition-colors">
							<div class="flex items-center gap-4">
								<IdeIcon class="h-5 w-5 text-muted-foreground" />
								<div class="flex-1">
									<div class="flex items-center gap-2">
										<span class="font-medium">{ide.name}</span>
										{#if ide.is_default}
											<Badge variant="default" class="text-xs">Default</Badge>
										{/if}
									</div>
									<p class="text-sm text-muted-foreground truncate">{normalizePath(ide.executable)}</p>
								</div>
							</div>

							<div class="flex items-center gap-2">
								{#if !ide.is_default}
									<Button variant="ghost" size="sm" onclick={() => setDefaultIde(ide)}>
										Set Default
									</Button>
								{/if}
								<Button variant="ghost" size="sm" onclick={() => startEditingIde(ide)}>
									<Edit class="h-4 w-4" />
								</Button>
								<Button variant="ghost" size="sm" onclick={() => deleteIde(ide)} class="text-destructive hover:text-destructive">
									<Trash2 class="h-4 w-4" />
								</Button>
							</div>
						</div>
					{/each}
				</div>
			{/if}

			{#if detectedIdes.length > 0}
				<Separator class="my-4" />
				<div>
					<p class="text-sm font-medium mb-2">Detected IDEs</p>
					<div class="flex flex-wrap gap-2">
						{#each detectedIdes as executablePath}
							{@const executableName = executablePath.split(/[/\\]/).pop() || executablePath}
							{@const normalizedPath = normalizePath(executablePath)}
							{@const alreadyAdded = ides.some(ide => normalizePath(ide.executable) === normalizedPath)}
							{#if !alreadyAdded}
								<Button
									variant="outline"
									size="sm"
									onclick={async () => {
										const name = executableName.replace(/\.exe$/i, '').replace(/\.app$/i, '');
										ideName = name;
										ideExecutable = normalizedPath;
										showModal = true;
									}}
								>
									<Plus class="h-3 w-3 mr-1" />
									{executableName}
								</Button>
							{/if}
						{/each}
					</div>
				</div>
			{/if}
		</CardContent>
	</Card>

	<Separator class="my-6" />

	<!-- Framework IDE Mappings Section -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-xl font-bold tracking-tight flex items-center gap-2">
				<Link2 class="h-5 w-5" />
				Framework IDE Mappings
			</h2>
			<p class="text-sm text-muted-foreground">
				Map frameworks to their preferred IDEs for automatic project opening
			</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={startAddingFramework}>
				<Plus class="h-4 w-4 mr-2" />
				Create Framework
			</Button>
			<Button variant="outline" size="sm" onclick={loadMappings} disabled={isLoadingMappings}>
				<RefreshCw class="h-4 w-4 mr-2" />
				Refresh
			</Button>
			<Button variant="outline" size="sm" onclick={startAddingMapping} disabled={ides.length === 0}>
				<Plus class="h-4 w-4 mr-2" />
				Add Mapping
			</Button>
		</div>
	</div>

	<!-- Mappings List -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<Link2 class="h-5 w-5" />
				Framework IDE Mappings
				<Badge variant="outline">{mappings.length}</Badge>
			</CardTitle>
			<CardDescription>
				Configure which IDE should open for each framework
			</CardDescription>
		</CardHeader>
		<CardContent>
			{#if isLoadingMappings}
				<div class="flex items-center justify-center py-8">
					<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
					<span class="ml-2 text-muted-foreground">Loading mappings...</span>
				</div>
			{:else if ides.length === 0}
				<div class="text-center py-8">
					<Link2 class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
					<p class="text-muted-foreground mb-2">No IDEs configured</p>
					<p class="text-sm text-muted-foreground mb-4">Please add an IDE first before creating framework mappings</p>
				</div>
			{:else if mappings.length === 0}
				<div class="space-y-6">
					<div class="text-center py-4">
						<Link2 class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
						<p class="text-muted-foreground mb-2 font-medium">No framework IDE mappings configured</p>
						<p class="text-sm text-muted-foreground mb-4">Quickly add common frameworks below or create a custom mapping</p>
					</div>
					
					<!-- All Frameworks (Suggested + User-defined) -->
					{#if isLoadingFrameworks}
						<div class="text-center py-4">
							<Loader2 class="h-4 w-4 animate-spin text-muted-foreground mx-auto" />
						</div>
					{:else if frameworkGroups.length > 0 || frameworks.length > 0}
						<div class="space-y-4">
							<p class="text-sm font-medium text-muted-foreground">Frameworks</p>
							
							<!-- Frameworks -->
							{#if frameworks.length > 0}
								{@const frameworksByCategory = frameworks.reduce((acc, fw) => {
									const cat = fw.category || 'Other';
									if (!acc[cat]) acc[cat] = [];
									acc[cat].push(fw);
									return acc;
								}, {} as Record<string, typeof frameworks>)}
								{#each Object.entries(frameworksByCategory) as [category, categoryFrameworks]}
									<div>
										<p class="text-xs font-medium text-muted-foreground mb-2 uppercase tracking-wide">{category}</p>
										<div class="flex flex-wrap gap-2">
											{#each categoryFrameworks as framework}
												{@const isMapped = mappings.some(m => m.framework.toLowerCase() === framework.name.toLowerCase())}
												<div class="relative group">
													<Button
														variant="outline"
														size="sm"
														onclick={() => quickAddFramework(framework.name)}
														disabled={isMapped || ides.length === 0}
														class="h-8 pr-8"
													>
														{#if isMapped}
															<svg class="w-3 h-3 mr-1.5" fill="currentColor" viewBox="0 0 20 20">
																<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
															</svg>
														{:else if framework.icon_type === 'file'}
															<img src={framework.icon} alt={framework.name} class="w-3 h-3 mr-1.5" />
														{:else}
															<Icon icon={framework.icon} class="w-3 h-3 mr-1.5" />
														{/if}
														{framework.name}
													</Button>
													<div class="absolute right-1 top-1 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
														<Button variant="ghost" size="sm" class="h-6 w-6 p-0" onclick={() => startEditingFramework(framework)}>
															<Edit class="h-3 w-3" />
														</Button>
														<Button variant="ghost" size="sm" class="h-6 w-6 p-0 text-destructive hover:text-destructive" onclick={() => deleteFramework(framework)}>
															<X class="h-3 w-3" />
														</Button>
													</div>
												</div>
											{/each}
										</div>
									</div>
								{/each}
							{/if}

							<!-- Suggested Frameworks -->
							{#each frameworkGroups as group}
								{@const unmappedInGroup = group.frameworks.filter(fw => 
									!mappings.some(m => m.framework.toLowerCase() === fw.name.toLowerCase())
								)}
								{#if unmappedInGroup.length > 0}
									<div>
										<p class="text-xs font-medium text-muted-foreground mb-2 uppercase tracking-wide">{group.category}</p>
										<div class="flex flex-wrap gap-2">
											{#each group.frameworks as framework}
												{@const isMapped = mappings.some(m => m.framework.toLowerCase() === framework.name.toLowerCase())}
												<Button
													variant="outline"
													size="sm"
													onclick={() => quickAddFramework(framework.name)}
													disabled={isMapped || ides.length === 0}
													class="h-8"
												>
													{#if isMapped}
														<svg class="w-3 h-3 mr-1.5" fill="currentColor" viewBox="0 0 20 20">
															<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
														</svg>
													{:else}
														<Icon icon={framework.icon} class="w-3 h-3 mr-1.5" />
													{/if}
													{framework.name}
												</Button>
											{/each}
										</div>
									</div>
								{/if}
							{/each}
							
							{#if ides.length === 0}
								<p class="text-xs text-muted-foreground mt-2">Please add an IDE first before creating framework mappings</p>
							{/if}
						</div>
					{/if}
				</div>
			{:else}
				<div class="space-y-4">
					<!-- Existing Mappings -->
					<div class="space-y-3">
						{#each mappings as mapping}
							{@const allFrameworks = getAllFrameworks()}
							{@const frameworkInfo = allFrameworks.find(f => f.name.toLowerCase() === mapping.framework.toLowerCase())}
							<div class="flex items-center justify-between p-4 rounded-md border hover:bg-accent transition-colors">
								<div class="flex items-center gap-4">
									{#if frameworkInfo}
										{#if frameworkInfo.isCustom && frameworkInfo.icon}
											{#if frameworkInfo.icon.startsWith('logos:') || frameworkInfo.icon.includes(':')}
												<Icon icon={frameworkInfo.icon} class="w-5 h-5" />
											{:else}
												<img src={frameworkInfo.icon} alt={mapping.framework} class="w-5 h-5" />
											{/if}
										{:else if frameworkInfo.icon}
											<Icon icon={frameworkInfo.icon} class="w-5 h-5" />
										{/if}
									{/if}
									<Badge variant="secondary" class="font-medium">{mapping.framework}</Badge>
									<ArrowRight class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm font-medium">{getIdeName(mapping.ide_id)}</span>
								</div>

								<div class="flex items-center gap-2">
									<Button variant="ghost" size="sm" onclick={() => startEditingMapping(mapping)}>
										<Edit class="h-4 w-4" />
									</Button>
									<Button variant="ghost" size="sm" onclick={() => deleteMapping(mapping)} class="text-destructive hover:text-destructive">
										<Trash2 class="h-4 w-4" />
									</Button>
								</div>
							</div>
						{/each}
					</div>

					<!-- Additional Frameworks (show unmapped ones) -->
					{#if (() => {
						const allFrameworksList = getAllFrameworks();
						const unmappedFrameworks = allFrameworksList.filter(fw => 
							!mappings.some(m => m.framework.toLowerCase() === fw.name.toLowerCase())
						);
						return unmappedFrameworks.length > 0;
					})()}
						{@const allFrameworksList = getAllFrameworks()}
						{@const unmappedFrameworks = allFrameworksList.filter(fw => 
							!mappings.some(m => m.framework.toLowerCase() === fw.name.toLowerCase())
						)}
						{@const groupedByCategory = unmappedFrameworks.reduce((acc, fw) => {
							if (!acc[fw.category]) acc[fw.category] = [];
							acc[fw.category].push(fw);
							return acc;
						}, {} as Record<string, typeof unmappedFrameworks>)}
						<div class="pt-4 border-t space-y-4">
							<p class="text-sm font-medium text-muted-foreground mb-3">Add More Frameworks</p>
							{#each Object.entries(groupedByCategory) as [category, frameworks]}
								<div>
									<p class="text-xs font-medium text-muted-foreground mb-2 uppercase tracking-wide">{category}</p>
									<div class="flex flex-wrap gap-2">
										{#each frameworks as framework}
											<Button
												variant="outline"
												size="sm"
												onclick={() => quickAddFramework(framework.name)}
												class="h-8"
											>
												{#if framework.icon && !framework.icon.startsWith('http') && !framework.icon.startsWith('/') && !framework.icon.includes(':')}
													<img src={framework.icon} alt={framework.name} class="w-3 h-3 mr-1.5" />
												{:else if framework.icon}
													<Icon icon={framework.icon} class="w-3 h-3 mr-1.5" />
												{/if}
												{framework.name}
											</Button>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

<!-- Add/Edit Modal -->
<Dialog.Root bind:open={showModal}>
	<Dialog.Content class="max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editingIde ? 'Edit IDE' : 'Add IDE'}</Dialog.Title>
		</Dialog.Header>
		
		<div class="space-y-4">
			<div class="space-y-2">
				<Label for="ide-name">IDE Name</Label>
				<Input
					id="ide-name"
					bind:value={ideName}
					placeholder="e.g., Visual Studio Code"
					required
				/>
			</div>

			<div class="space-y-2">
				<Label for="ide-executable">Executable Path</Label>
				<Input
					id="ide-executable"
					bind:value={ideExecutable}
					placeholder="e.g., /usr/bin/code or C:\Program Files\Code\code.exe"
					required
				/>
				<p class="text-xs text-muted-foreground">
					Full path to the IDE executable
				</p>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={closeModal}>Cancel</Button>
				<Button onclick={saveIde}>{editingIde ? 'Update' : 'Add'} IDE</Button>
			</Dialog.Footer>
		</div>
	</Dialog.Content>
</Dialog.Root>

<!-- Framework IDE Mapping Modal -->
<Dialog.Root bind:open={showMappingModal}>
	<Dialog.Content class="max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editingMapping ? 'Edit Framework IDE Mapping' : 'Add Framework IDE Mapping'}</Dialog.Title>
		</Dialog.Header>
		
		<div class="space-y-4">
			<div class="space-y-2">
				<Label for="mapping-framework">Framework</Label>
				<Input
					id="mapping-framework"
					bind:value={mappingFramework}
					placeholder="e.g., React, Vue, Angular, Node.js"
					required
				/>
				<p class="text-xs text-muted-foreground">
					The framework name that will trigger this IDE
				</p>
			</div>

			<div class="space-y-2">
				<Label for="mapping-ide">IDE</Label>
				<Select
					defaultValue={mappingIdeId || ''}
					options={ides.map(ide => ({ value: String(ide.id || ''), label: ide.name }))}
					onSelect={(value) => mappingIdeId = value || ''}
					placeholder="Select an IDE..."
				/>
				<p class="text-xs text-muted-foreground">
					The IDE that will open for this framework
				</p>
			</div>

			<Dialog.Footer>
				<Button variant="outline" onclick={closeMappingModal}>Cancel</Button>
				<Button onclick={saveMapping}>{editingMapping ? 'Update' : 'Create'} Mapping</Button>
			</Dialog.Footer>
		</div>
	</Dialog.Content>
</Dialog.Root>

<!-- Framework Modal -->
<Dialog.Root bind:open={showFrameworkModal}>
	<Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
		<Dialog.Header>
			<Dialog.Title>{editingFramework ? 'Edit Framework' : 'Create Framework'}</Dialog.Title>
		</Dialog.Header>
		
		<div class="space-y-4">
				<div class="space-y-2">
					<Label for="framework-name">Framework Name</Label>
					<Input
						id="framework-name"
						bind:value={frameworkName}
						placeholder="e.g., MyFramework"
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="framework-category">Category</Label>
					<Input
						id="framework-category"
						bind:value={frameworkCategory}
						placeholder="e.g., Custom, Frontend, Backend"
					/>
				</div>

				<div class="space-y-2">
					<Label>Icon</Label>
					<div class="flex gap-2 mb-2">
						<Button
							type="button"
							variant={frameworkIconType === 'devicon' ? 'default' : 'outline'}
							size="sm"
							onclick={() => { frameworkIconType = 'devicon'; iconSearchQuery = ''; iconSearchResults = []; }}
						>
							DevIcon
						</Button>
						<Button
							type="button"
							variant={frameworkIconType === 'file' ? 'default' : 'outline'}
							size="sm"
							onclick={() => { frameworkIconType = 'file'; iconSearchQuery = ''; iconSearchResults = []; }}
						>
							<Upload class="h-4 w-4 mr-2" />
							File
						</Button>
					</div>

					{#if frameworkIconType === 'devicon'}
						<div class="space-y-2">
							<div class="flex gap-2">
								<Input
									bind:value={iconSearchQuery}
									placeholder="Search for icon (e.g., react, vue, python)"
									oninput={searchIcons}
								/>
								<Button type="button" variant="outline" size="sm" onclick={searchIcons}>
									<Search class="h-4 w-4" />
								</Button>
							</div>
							
							{#if iconSearchResults.length > 0}
								<div class="grid grid-cols-4 gap-2 p-2 border rounded-md max-h-48 overflow-y-auto">
									{#each iconSearchResults as iconName}
										<Button
											type="button"
											variant={frameworkIcon === iconName ? 'default' : 'outline'}
											size="sm"
											onclick={() => { frameworkIcon = iconName; iconSearchQuery = ''; iconSearchResults = []; }}
											class="p-2 h-auto flex flex-col items-center gap-1"
										>
											<Icon icon={iconName} class="w-6 h-6" />
											<span class="text-xs truncate w-full text-center">{iconName.replace('logos:', '')}</span>
										</Button>
									{/each}
								</div>
							{/if}

							{#if frameworkIcon}
								<div class="p-3 border rounded-md flex items-center gap-3">
									<Icon icon={frameworkIcon} class="w-8 h-8" />
									<div class="flex-1">
										<p class="text-sm font-medium">{frameworkIcon}</p>
										<Button variant="ghost" size="sm" class="mt-1 h-6" onclick={() => frameworkIcon = ''}>
											<X class="h-3 w-3 mr-1" />
											Remove
										</Button>
									</div>
								</div>
							{:else}
								<p class="text-xs text-muted-foreground">Search for an icon or enter icon name (e.g., logos:react, logos:vue)</p>
								<Input
									bind:value={frameworkIcon}
									placeholder="e.g., logos:react, logos:vue"
								/>
							{/if}
						</div>
					{:else}
						<div class="space-y-2">
							<Button type="button" variant="outline" onclick={selectIconFile}>
								<Upload class="h-4 w-4 mr-2" />
								Select Icon File
							</Button>
							{#if frameworkIcon}
								<div class="p-3 border rounded-md flex items-center gap-3">
									<img src={frameworkIcon} alt="Framework icon" class="w-8 h-8" />
									<div class="flex-1">
										<p class="text-sm font-medium truncate">{frameworkIcon}</p>
										<Button variant="ghost" size="sm" class="mt-1 h-6" onclick={() => frameworkIcon = ''}>
											<X class="h-3 w-3 mr-1" />
											Remove
										</Button>
									</div>
								</div>
							{/if}
						</div>
					{/if}
				</div>

				<Dialog.Footer>
					<Button variant="outline" onclick={closeFrameworkModal}>Cancel</Button>
					<Button onclick={saveFramework}>{editingFramework ? 'Update' : 'Create'} Framework</Button>
				</Dialog.Footer>
			</div>
	</Dialog.Content>
</Dialog.Root>
