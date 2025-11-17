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
		role,
		tabindex,
		onkeydown
	}: Props = $props();

	const isClickable = $derived(!!onclick);
	const cardRole = $derived(role ?? (isClickable ? 'button' : undefined));
	const cardTabindex = $derived(tabindex ?? (isClickable ? 0 : undefined));
</script>

<div 
	class={cn(
		'bg-card text-card-foreground rounded-lg border shadow-sm transition-all duration-200',
		isClickable && 'cursor-pointer hover:shadow-md hover:border-border/80',
		className
	)}
	{onclick}
	role={cardRole}
	tabindex={cardTabindex}
	onkeydown={onkeydown}
>
	{@render children?.()}
</div>
