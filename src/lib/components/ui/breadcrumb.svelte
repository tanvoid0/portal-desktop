<!--
	Breadcrumb component for desktop navigation
	Provides hierarchical navigation with clickable links
-->

<script lang="ts">
	import { goto } from '$app/navigation';
	import { ChevronRight, Home } from '@lucide/svelte';
	import { Button } from './button';
	import { cn } from '$lib/utils';
	import type { BreadcrumbItem } from '@/lib/domains/shared';

	interface Props {
		items: BreadcrumbItem[];
		showHome?: boolean;
		homeIcon?: boolean;
		homeItem?: BreadcrumbItem;
		class?: string;
	}

	let { 
		items = [], 
		showHome = true, 
		homeIcon = true,
		homeItem,
		class: className = ''
	}: Props = $props();

	// Default home item (fallback)
	const defaultHomeItem: BreadcrumbItem = {
		label: 'Home',
		href: '/',
		icon: 'home'
	};

	// Use custom home item if provided, otherwise use default
	const effectiveHomeItem = $derived(homeItem || defaultHomeItem);

	// Combine home item with provided items
	let allItems = $derived(showHome ? [effectiveHomeItem, ...items] : items);

	function handleClick(item: BreadcrumbItem) {
		if (item.href && !item.disabled) {
			goto(item.href);
		}
	}

	function isLastItem(index: number): boolean {
		return index === allItems.length - 1;
	}
</script>

<nav class={cn("flex items-center space-x-1 text-sm", className)} aria-label="Breadcrumb">
	<ol class="flex items-center space-x-1">
		{#each allItems as item, index (index)}
			<li class="flex items-center">
				{#if index > 0}
					<ChevronRight class="h-4 w-4 text-muted-foreground mx-1" />
				{/if}
				
				{#if item.href && !item.disabled && !isLastItem(index)}
					<Button
						variant="ghost"
						size="sm"
						class="h-auto p-1 text-muted-foreground hover:text-foreground"
						onclick={() => handleClick(item)}
					>
						{#if item.icon === 'home' && homeIcon}
							<Home class="h-3 w-3 mr-1" />
						{/if}
						<span class={item.label.startsWith('?') ? 'text-xs' : ''}>{item.label}</span>
					</Button>
				{:else}
					<span 
						class="flex items-center px-1 py-0.5 text-foreground font-medium"
						class:opacity-50={item.disabled}
					>
						{#if item.icon === 'home' && homeIcon}
							<Home class="h-3 w-3 mr-1" />
						{/if}
						<span class={item.label.startsWith('?') ? 'text-xs' : ''}>{item.label}</span>
					</span>
				{/if}
			</li>
		{/each}
	</ol>
</nav>
