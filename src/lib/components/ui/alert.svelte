<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLAttributes } from 'svelte/elements';
	
	const variants = {
		default: 'bg-background text-foreground',
		destructive: 'border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive',
		warning: 'border-yellow-500/50 text-yellow-700 dark:text-yellow-400 [&>svg]:text-yellow-600'
	};
	
	interface AlertProps extends HTMLAttributes<HTMLDivElement> {
		variant?: keyof typeof variants;
	}
	
	let {
		class: className,
		variant = 'default',
		children,
		...restProps
	}: AlertProps = $props();
</script>

<div
	class={cn(
		'relative w-full rounded-lg border p-4 [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground',
		variants[variant],
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>

