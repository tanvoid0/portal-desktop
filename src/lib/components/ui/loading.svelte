<!--
	Production-ready Loading Component
	Provides consistent loading states across the application
-->

<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		size?: 'sm' | 'md' | 'lg' | 'xl';
		variant?: 'spinner' | 'dots' | 'pulse' | 'skeleton';
		text?: string;
		overlay?: boolean;
		class?: string;
	}

	let {
		size = 'md',
		variant = 'spinner',
		text = 'Loading...',
		overlay = false,
		class: className = ''
	}: Props = $props();

	// Size configurations
	const sizeConfig = {
		sm: { size: 'h-4 w-4', text: 'text-sm' },
		md: { size: 'h-6 w-6', text: 'text-base' },
		lg: { size: 'h-8 w-8', text: 'text-lg' },
		xl: { size: 'h-12 w-12', text: 'text-xl' }
	};

	const currentSize = sizeConfig[size];
</script>

{#if overlay}
	<div class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
		<div class={cn("flex flex-col items-center gap-3", className)}>
			{#if variant === 'spinner'}
				<svg class={cn("animate-spin text-primary", currentSize.size)} fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
			{:else if variant === 'dots'}
				<div class="flex space-x-1">
					<div class="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.3s]"></div>
					<div class="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.15s]"></div>
					<div class="w-2 h-2 bg-primary rounded-full animate-bounce"></div>
				</div>
			{:else if variant === 'pulse'}
				<div class={cn("bg-primary rounded-full animate-pulse", currentSize.size)}></div>
			{:else if variant === 'skeleton'}
				<div class="space-y-2">
					<div class="h-4 bg-muted rounded animate-pulse"></div>
					<div class="h-4 bg-muted rounded animate-pulse w-3/4"></div>
					<div class="h-4 bg-muted rounded animate-pulse w-1/2"></div>
				</div>
			{/if}
			
			{#if text}
				<p class={cn("text-muted-foreground", currentSize.text)}>{text}</p>
			{/if}
		</div>
	</div>
{:else}
	<div class={cn("flex flex-col items-center gap-3", className)}>
		{#if variant === 'spinner'}
			<svg class={cn("animate-spin text-primary", currentSize.size)} fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
			</svg>
		{:else if variant === 'dots'}
			<div class="flex space-x-1">
				<div class="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.3s]"></div>
				<div class="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.15s]"></div>
				<div class="w-2 h-2 bg-primary rounded-full animate-bounce"></div>
			</div>
		{:else if variant === 'pulse'}
			<div class={cn("bg-primary rounded-full animate-pulse", currentSize.size)}></div>
		{:else if variant === 'skeleton'}
			<div class="space-y-2 w-full">
				<div class="h-4 bg-muted rounded animate-pulse"></div>
				<div class="h-4 bg-muted rounded animate-pulse w-3/4"></div>
				<div class="h-4 bg-muted rounded animate-pulse w-1/2"></div>
			</div>
		{/if}
		
		{#if text}
			<p class={cn("text-muted-foreground", currentSize.text)}>{text}</p>
		{/if}
	</div>
{/if}
