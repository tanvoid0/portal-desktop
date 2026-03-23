<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import Card from "./card.svelte";
	import CardHeader from "./card-header.svelte";
	import CardTitle from "./card-title.svelte";
	import CardDescription from "./card-description.svelte";
	import CardContent from "./card-content.svelte";

	interface Props extends Omit<HTMLAttributes<HTMLDivElement>, 'onclick'> {
		ref?: HTMLDivElement | null;
		class?: string;
		children?: any;
		title?: string;
		description?: string;
		value?: string | number;
		icon?: any;
		trend?: 'up' | 'down' | 'neutral';
		trendValue?: string;
		gradient?: boolean;
		onclick?: () => void;
		onkeydown?: (e: KeyboardEvent) => void;
	}

	let {
		ref = $bindable(null),
		class: className,
		children,
		title,
		description,
		value,
		icon,
		trend,
		trendValue,
		gradient = true,
		onclick,
		onkeydown,
		...restProps
	}: Props = $props();

	const trendConfig = {
		up: {
			color: 'text-success-600 dark:text-success-400',
			icon: '↑'
		},
		down: {
			color: 'text-error-600 dark:text-error-400',
			icon: '↓'
		},
		neutral: {
			color: 'text-muted-foreground',
			icon: '→'
		}
	};

	const trendStyles = trend ? trendConfig[trend] : null;
</script>

<Card
	bind:ref={ref}
	variant="surface"
	elevation="elevated"
	{gradient}
	glass={true}
	onclick={onclick}
	onkeydown={onkeydown}
	class={cn(
		"relative overflow-hidden group",
		className
	)}
	{...restProps}
>
	<!-- Modern gradient overlay -->
	<div class="absolute inset-0 gradient-modern opacity-50 pointer-events-none"></div>
	<div class="absolute inset-0 bg-gradient-to-br from-primary/8 via-primary/4 to-transparent pointer-events-none"></div>
	
	<div class="relative z-10">
		{#if title || description || icon}
			<CardHeader class="pb-3">
				<div class="flex items-start justify-between">
					<div class="flex-1">
						{#if title}
							<CardTitle class="text-lg">{title}</CardTitle>
						{/if}
						{#if description}
							<CardDescription class="mt-1">{description}</CardDescription>
						{/if}
					</div>
					{#if icon}
						{@const Icon = icon}
						<div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
							<Icon class="w-6 h-6 text-primary" />
						</div>
					{/if}
				</div>
			</CardHeader>
		{/if}
		
		<CardContent>
					{#if value !== undefined}
				<div class="space-y-2">
					<div class="flex items-baseline gap-2">
						<span class="text-4xl font-extrabold bg-gradient-to-r from-primary via-primary/85 to-primary/70 bg-clip-text text-transparent">
							{value}
						</span>
						{#if trend && trendValue}
							<span class={cn("text-sm font-medium", trendStyles?.color)}>
								{trendStyles?.icon} {trendValue}
							</span>
						{/if}
					</div>
				</div>
			{/if}
			
			{#if children}
				<div class="mt-4">
					{@render children?.()}
				</div>
			{/if}
		</CardContent>
	</div>
</Card>

