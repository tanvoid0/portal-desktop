<!--
	BaseCard - Shared card component for consistent styling across domains
	Uses Svelte 5 runes and Tailwind CSS
-->

<script lang="ts">
	import { cn } from '@/lib/utils';

	interface Props {
		class?: string;
		children?: any;
		onclick?: () => void;
		role?: string;
		tabindex?: number | null;
		onkeydown?: (e: KeyboardEvent) => void;
	}

	let { 
		class: className = '',
		children,
		onclick,
		role = 'button',
		tabindex = 0,
		onkeydown = () => {}
	}: Props = $props();

	const isClickable = $derived(!!onclick);
</script>

<div 
	class={cn(
		'bg-card text-card-foreground rounded-lg border shadow-sm transition-all duration-200',
		isClickable && 'cursor-pointer hover:shadow-md hover:border-border/80',
		className
	)}
	{onclick}
	{role}
	{tabindex}
	{onkeydown}
>
	{@render children?.()}
</div>
