<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="card"
	class={cn(
		'relative flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground',
		'shadow-lg transition-shadow duration-200',
		'hover:shadow-xl',
		'backdrop-blur-sm',
		'before:absolute before:inset-0 before:rounded-xl before:pointer-events-none',
		'before:bg-gradient-to-br before:from-white/5 before:via-transparent before:to-transparent',
		'dark:before:from-white/[0.02] dark:before:via-transparent dark:before:to-transparent',
		className
	)}
	{...restProps}
>
	<div class="relative z-10">
		{@render children?.()}
	</div>
</div>
