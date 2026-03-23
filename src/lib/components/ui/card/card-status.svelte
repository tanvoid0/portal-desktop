<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import Card from "./card.svelte";
	import CardHeader from "./card-header.svelte";
	import CardTitle from "./card-title.svelte";
	import CardDescription from "./card-description.svelte";
	import CardContent from "./card-content.svelte";
	import CardFooter from "./card-footer.svelte";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		ref?: HTMLDivElement | null;
		class?: string;
		children?: any;
		status: 'success' | 'warning' | 'error' | 'info';
		icon?: any;
		title?: string;
		description?: string;
	}

	let {
		ref = $bindable(null),
		class: className,
		children,
		status,
		icon,
		title,
		description,
		...restProps
	}: Props = $props();

	const statusConfig = {
		success: {
			border: 'border-l-success-500',
			bg: 'bg-success-50 dark:bg-success-950/20',
			text: 'text-success-700 dark:text-success-300',
			iconBg: 'bg-success-100 dark:bg-success-900/30',
			iconColor: 'text-success-600 dark:text-success-400'
		},
		warning: {
			border: 'border-l-warning-500',
			bg: 'bg-warning-50 dark:bg-warning-950/20',
			text: 'text-warning-700 dark:text-warning-300',
			iconBg: 'bg-warning-100 dark:bg-warning-900/30',
			iconColor: 'text-warning-600 dark:text-warning-400'
		},
		error: {
			border: 'border-l-error-500',
			bg: 'bg-error-50 dark:bg-error-950/20',
			text: 'text-error-700 dark:text-error-300',
			iconBg: 'bg-error-100 dark:bg-error-900/30',
			iconColor: 'text-error-600 dark:text-error-400'
		},
		info: {
			border: 'border-l-info-500',
			bg: 'bg-info-50 dark:bg-info-950/20',
			text: 'text-info-700 dark:text-info-300',
			iconBg: 'bg-info-100 dark:bg-info-900/30',
			iconColor: 'text-info-600 dark:text-info-400'
		}
	};

	const config = statusConfig[status];
</script>

<Card
	bind:ref={ref}
	borderAccent="left"
	class={cn(
		"border-l-4",
		config.border,
		config.bg,
		className
	)}
	{...restProps}
>
	{#if title || description || icon}
		<CardHeader>
			<div class="flex items-start gap-3">
				{#if icon}
					{@const Icon = icon}
					<div class={cn("w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0", config.iconBg)}>
						<Icon class={cn("w-5 h-5", config.iconColor)} />
					</div>
				{/if}
				<div class="flex-1 min-w-0">
					{#if title}
						<CardTitle class={cn(config.text)}>{title}</CardTitle>
					{/if}
					{#if description}
						<CardDescription class={cn("mt-1", config.text, "opacity-80")}>{description}</CardDescription>
					{/if}
				</div>
			</div>
		</CardHeader>
	{/if}
	
	{#if children}
		<CardContent class={cn(config.text)}>
			{@render children?.()}
		</CardContent>
	{/if}
</Card>

