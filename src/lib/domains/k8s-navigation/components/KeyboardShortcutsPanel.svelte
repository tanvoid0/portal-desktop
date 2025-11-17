<!-- Reusable Keyboard Shortcuts Panel Component -->
<!-- Can be customized per page to show relevant shortcuts -->

<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Keyboard, X } from '@lucide/svelte';
	import KeyboardShortcutHint from './KeyboardShortcutHint.svelte';
	import { formatShortcut } from '../utils/shortcutParser';

	export interface ShortcutGroup {
		title: string;
		shortcuts: Array<{ key: string; description: string; hint?: string }>;
	}

	interface Props {
		shortcuts: Array<{ key: string; description: string; category?: string }> | ShortcutGroup[];
		title?: string;
		variant?: 'panel' | 'modal' | 'inline';
		showTitle?: boolean;
		showCategoryHeaders?: boolean;
		collapsible?: boolean;
		maxHeight?: string;
	}

	let {
		shortcuts,
		title = 'Keyboard Shortcuts',
		variant = 'panel',
		showTitle = true,
		showCategoryHeaders = true,
		collapsible = false,
		maxHeight = '400px'
	}: Props = $props();

	let isExpanded = $state(!collapsible);
	let isOpen = $state(false);

	// Normalize shortcuts to groups
	const shortcutGroups = $derived(() => {
		// If already in group format
		if (shortcuts.length > 0 && 'title' in shortcuts[0] && 'shortcuts' in shortcuts[0]) {
			return shortcuts as ShortcutGroup[];
		}

		// Convert flat array to groups by category
		const flatShortcuts = shortcuts as Array<{ key: string; description: string; category?: string }>;
		const groups = new Map<string, Array<{ key: string; description: string }>>();

		flatShortcuts.forEach(shortcut => {
			const category = shortcut.category || 'Other';
			if (!groups.has(category)) {
				groups.set(category, []);
			}
			groups.get(category)!.push({ key: shortcut.key, description: shortcut.description });
		});

		return Array.from(groups.entries()).map(([title, shortcuts]) => ({
			title,
			shortcuts
		}));
	});

	function toggle() {
		if (collapsible) {
			isExpanded = !isExpanded;
		}
	}

	function openModal() {
		isOpen = true;
	}

	function closeModal() {
		isOpen = false;
	}
</script>

{#if variant === 'inline'}
	<div class="space-y-2">
		{#if showTitle}
			<h3 class="text-sm font-semibold flex items-center gap-2">
				<Keyboard class="h-4 w-4" />
				{title}
			</h3>
		{/if}
		<div class="space-y-3">
			{#each shortcutGroups() as group}
				<div>
					{#if showCategoryHeaders}
						<h4 class="text-xs font-medium text-muted-foreground mb-2 uppercase tracking-wider">
							{group.title}
						</h4>
					{/if}
					<div class="space-y-1.5">
						{#each group.shortcuts as shortcut}
							<div class="flex items-center justify-between text-sm">
								<span class="text-muted-foreground">{shortcut.description}</span>
								<KeyboardShortcutHint shortcut={shortcut.key} />
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</div>
{:else if variant === 'panel'}
	<Card>
		<CardHeader class="pb-3">
			<div class="flex items-center justify-between">
				<CardTitle class="text-base flex items-center gap-2">
					<Keyboard class="h-4 w-4" />
					{title}
				</CardTitle>
				{#if collapsible}
					<Button variant="ghost" size="sm" onclick={toggle}>
						{#if isExpanded}
							<X class="h-4 w-4" />
						{:else}
							<Keyboard class="h-4 w-4" />
						{/if}
					</Button>
				{/if}
			</div>
		</CardHeader>
		{#if isExpanded}
			<CardContent>
				<div class="space-y-4" style="max-height: {maxHeight}; overflow-y: auto;">
					{#each shortcutGroups() as group}
						<div>
							{#if showCategoryHeaders}
								<h4 class="text-xs font-medium text-muted-foreground mb-2 uppercase tracking-wider">
									{group.title}
								</h4>
							{/if}
							<div class="space-y-1.5">
								{#each group.shortcuts as shortcut}
									<div class="flex items-center justify-between text-sm py-1">
										<span class="text-muted-foreground">{shortcut.description}</span>
										<KeyboardShortcutHint shortcut={shortcut.key} />
									</div>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			</CardContent>
		{/if}
	</Card>
{:else if variant === 'modal'}
	<!-- Modal variant would use Dialog component -->
	<Button variant="outline" size="sm" onclick={openModal}>
		<Keyboard class="h-4 w-4 mr-2" />
		Shortcuts
	</Button>

	{#if isOpen}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm"
			onclick={closeModal}
			role="dialog"
			aria-modal="true"
		>
			<Card class="w-full max-w-2xl max-h-[80vh] overflow-hidden" onclick={(e) => e.stopPropagation()}>
				<CardHeader class="pb-3">
					<div class="flex items-center justify-between">
						<CardTitle class="text-lg flex items-center gap-2">
							<Keyboard class="h-5 w-5" />
							{title}
						</CardTitle>
						<Button variant="ghost" size="sm" onclick={closeModal}>
							<X class="h-4 w-4" />
						</Button>
					</div>
				</CardHeader>
				<CardContent>
					<div class="space-y-6" style="max-height: calc(80vh - 120px); overflow-y: auto;">
						{#each shortcutGroups() as group}
							<div>
								{#if showCategoryHeaders}
									<h4 class="text-sm font-semibold mb-3">{group.title}</h4>
								{/if}
								<div class="grid grid-cols-2 gap-x-4 gap-y-2">
									{#each group.shortcuts as shortcut}
										<div class="flex items-center justify-between text-sm">
											<span class="text-muted-foreground">{shortcut.description}</span>
											<KeyboardShortcutHint shortcut={shortcut.key} />
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				</CardContent>
			</Card>
		</div>
	{/if}
{/if}

