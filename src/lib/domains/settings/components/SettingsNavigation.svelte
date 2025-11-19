<!--
	Settings Navigation - Sidebar navigation for settings sections
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Settings, Code, Terminal, Palette, Laptop, Brain, Package, Bot, Download } from '@lucide/svelte';

	type SettingsSectionType = 'general' | 'editor' | 'terminal' | 'theme' | 'ides' | 'frameworks' | 'learning' | 'autonomy' | 'updates';

	interface Props {
		currentSection?: SettingsSectionType | 'framework-ides' | 'updates';
		className?: string;
	}

	let {
		currentSection,
		className = ''
	}: Props = $props();

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
			id: 'frameworks' as const,
			label: 'Frameworks',
			description: 'Manage frameworks & recommendations',
			icon: Package,
			path: '/settings/frameworks'
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
		const normalizedSection = section || 'general';
		// Type guard to ensure we return a valid section
		if (['general', 'editor', 'terminal', 'theme', 'ides', 'frameworks', 'learning', 'autonomy', 'updates'].includes(normalizedSection)) {
			return normalizedSection as SettingsSectionType;
		}
		return 'general';
	});

	function handleSectionClick(sectionId: string, path: string) {
		goto(path);
	}
</script>

<nav class="space-y-2 {className}">
	{#each sections as section}
		{@const isActive = activeSection() === section.id}
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
	{/each}
</nav>

