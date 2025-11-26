<!--
	Settings Navigation - Sidebar navigation for settings sections
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '$lib/components/ui/collapsible';
	import { Settings, Code, Terminal, Palette, Laptop, Brain, Package, Bot, Download, Box, FileCode, ChevronDown, Languages } from '@lucide/svelte';

	type SettingsSectionType = 'general' | 'editor' | 'terminal' | 'theme' | 'ides' | 'frameworks-languages' | 'frameworks' | 'package-managers' | 'languages' | 'learning' | 'autonomy' | 'updates';

	interface Props {
		currentSection?: SettingsSectionType | 'framework-ides' | 'updates';
		className?: string;
	}

	let {
		currentSection,
		className = ''
	}: Props = $props();

	// Check if we're in the frameworks-languages section or any of its sub-sections
	const isFrameworksLanguagesSection = $derived(() => {
		const path = $page.url.pathname;
		return path.startsWith('/settings/frameworks-languages') || 
		       path.startsWith('/settings/frameworks') || 
		       path.startsWith('/settings/package-managers') || 
		       path.startsWith('/settings/languages');
	});
	
	// State for collapsible open/close
	let collapsibleOpen = $state(false);
	
	$effect(() => {
		collapsibleOpen = isFrameworksLanguagesSection();
	});

	const sections = [
		{
			id: 'general' as const,
			label: 'General',
			description: 'Application preferences',
			icon: Settings,
			path: '/settings/general'
		},
		{
			id: 'updates' as const,
			label: 'Updates',
			description: 'Check for updates',
			icon: Download,
			path: '/settings/updates'
		},
		{
			id: 'editor' as const,
			label: 'Editor',
			description: 'Code editor settings',
			icon: Code,
			path: '/settings/editor'
		},
		{
			id: 'terminal' as const,
			label: 'Terminal',
			description: 'Terminal configuration',
			icon: Terminal,
			path: '/settings/terminal'
		},
		{
			id: 'ides' as const,
			label: 'IDEs',
			description: 'Configure IDEs & framework mappings',
			icon: Laptop,
			path: '/settings/ides'
		},
		{
			id: 'frameworks-languages' as const,
			label: 'Frameworks & Languages',
			description: 'Languages, frameworks & package managers',
			icon: Languages,
			path: '/settings/frameworks-languages',
			subSections: [
				{
					id: 'languages' as const,
					label: 'Languages',
					path: '/settings/languages'
				},
				{
					id: 'package-managers' as const,
					label: 'Package Managers',
					path: '/settings/package-managers'
				},
				{
					id: 'frameworks' as const,
					label: 'Frameworks',
					path: '/settings/frameworks'
				}
			]
		},
		{
			id: 'theme' as const,
			label: 'Theme',
			description: 'Appearance & colors',
			icon: Palette,
			path: '/settings/theme'
		},
		{
			id: 'learning' as const,
			label: 'Learning',
			description: 'ML learning & AI settings',
			icon: Brain,
			path: '/settings/learning'
		},
		{
			id: 'autonomy' as const,
			label: 'Autonomy',
			description: 'Autonomous action settings',
			icon: Bot,
			path: '/settings/autonomy'
		}
	];

	// Derive current section from URL if not provided
	const activeSection = $derived((): SettingsSectionType => {
		if (currentSection) {
			// Map framework-ides to ides
			return currentSection === 'framework-ides' ? 'ides' : currentSection;
		}
		const path = $page.url.pathname;
		if (path === '/settings' || path === '/settings/') return 'general';
		const section = path.replace('/settings/', '').replace(/\/$/, '');
		// Redirect framework-ides to ides
		if (section === 'framework-ides') return 'ides';
		// Map old paths to new structure
		if (section === 'frameworks' || section === 'package-managers' || section === 'languages') {
			return 'frameworks-languages';
		}
		const normalizedSection = section || 'general';
		// Type guard to ensure we return a valid section
		if (['general', 'editor', 'terminal', 'theme', 'ides', 'frameworks-languages', 'frameworks', 'package-managers', 'languages', 'learning', 'autonomy', 'updates'].includes(normalizedSection)) {
			return normalizedSection as SettingsSectionType;
		}
		return 'general';
	});

	// Get active sub-section for frameworks-languages
	const activeSubSection = $derived(() => {
		if (!isFrameworksLanguagesSection()) return null;
		const path = $page.url.pathname;
		if (path.includes('/languages')) return 'languages';
		if (path.includes('/package-managers')) return 'package-managers';
		if (path.includes('/frameworks') && !path.includes('/frameworks-languages')) return 'frameworks';
		// If on the main frameworks-languages page, check query param
		if (path.includes('/frameworks-languages')) {
			const tab = $page.url.searchParams.get('tab');
			if (tab) return tab;
		}
		return null;
	});

	function handleSectionClick(sectionId: string, path: string) {
		goto(path);
	}

	function handleSubSectionClick(path: string) {
		goto(path);
	}
</script>

<nav class="space-y-2 {className}">
	{#each sections as section}
		{@const isActive = activeSection() === section.id}
		{#if section.subSections}
			<Collapsible bind:open={collapsibleOpen} class="w-full">
				<CollapsibleTrigger
					type="button"
					onclick={() => handleSectionClick(section.id, section.path)}
					class="w-full flex items-start px-4 py-3 text-base font-medium rounded-lg transition-colors justify-start h-auto whitespace-normal {isActive
						? 'bg-accent text-accent-foreground' 
						: 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}"
				>
					{@const Icon = section.icon}
					<Icon class="h-5 w-5 mr-3 mt-0.5 flex-shrink-0" />
					<div class="flex-1 text-left min-w-0">
						<div class="font-semibold text-base leading-tight">{section.label}</div>
						<p class="text-sm text-muted-foreground mt-1 leading-relaxed break-words">
							{section.description}
						</p>
					</div>
					<ChevronDown class="h-4 w-4 ml-2 mt-0.5 transition-transform {collapsibleOpen ? 'rotate-180' : ''}" />
				</CollapsibleTrigger>
				<CollapsibleContent>
					<div class="ml-4 mt-1 space-y-1 border-l-2 border-border pl-4">
						{#each section.subSections as subSection}
							{@const isSubActive = activeSubSection() === subSection.id}
							<Button
								type="button"
								variant="ghost"
								onclick={() => handleSubSectionClick(subSection.path)}
								class="w-full flex items-center px-3 py-2 text-sm rounded-md transition-colors justify-start h-auto {isSubActive
									? 'bg-accent text-accent-foreground font-medium' 
									: 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}"
							>
								<span class="text-sm">{subSection.label}</span>
							</Button>
						{/each}
					</div>
				</CollapsibleContent>
			</Collapsible>
		{:else}
			<Button
				type="button"
				variant="ghost"
				onclick={() => handleSectionClick(section.id, section.path)}
				class="w-full flex items-start px-4 py-3 text-base font-medium rounded-lg transition-colors justify-start h-auto whitespace-normal {isActive
					? 'bg-accent text-accent-foreground' 
					: 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}"
			>
				{@const Icon = section.icon}
				<Icon class="h-5 w-5 mr-3 mt-0.5 flex-shrink-0" />
				<div class="flex-1 text-left min-w-0">
					<div class="font-semibold text-base leading-tight">{section.label}</div>
					<p class="text-sm text-muted-foreground mt-1 leading-relaxed break-words">
						{section.description}
					</p>
				</div>
			</Button>
		{/if}
	{/each}
</nav>

