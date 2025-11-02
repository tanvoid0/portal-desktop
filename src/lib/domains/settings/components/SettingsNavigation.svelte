<!--
	Settings Navigation - Sidebar navigation for settings sections
-->

<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Settings, Code, Terminal, Palette, Laptop } from 'lucide-svelte';

	type SettingsSectionType = 'general' | 'editor' | 'terminal' | 'theme' | 'ides';

	interface Props {
		currentSection?: SettingsSectionType | 'framework-ides';
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
			id: 'theme' as const,
			label: 'Theme',
			description: 'Appearance & colors',
			icon: Palette,
			path: '/settings/theme'
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
		if (['general', 'editor', 'terminal', 'theme', 'ides'].includes(normalizedSection)) {
			return normalizedSection as SettingsSectionType;
		}
		return 'general';
	});

	function handleSectionClick(sectionId: string, path: string) {
		goto(path);
	}
</script>

<nav class="space-y-1 {className}">
	{#each sections as section}
		{@const isActive = activeSection() === section.id}
		<Button
			type="button"
			variant="ghost"
			size="sm"
			onclick={() => handleSectionClick(section.id, section.path)}
			class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors justify-start {isActive
				? 'bg-accent text-accent-foreground' 
				: 'text-muted-foreground hover:text-foreground hover:bg-accent/50'}"
		>
			{@const Icon = section.icon}
			<Icon class="h-4 w-4 mr-3" />
			<div class="flex-1 text-left">
				<div class="font-medium">{section.label}</div>
				<p class="text-xs text-muted-foreground mt-0.5">
					{section.description}
				</p>
			</div>
		</Button>
	{/each}
</nav>

