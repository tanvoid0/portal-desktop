<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLAttributes } from 'svelte/elements';
	
	interface AlertDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {}
	
	let {
		class: className,
		children,
		...restProps
	}: AlertDescriptionProps & {
		children?: import('svelte').Snippet;
	} = $props();
</script>

<div
	class={cn('text-sm [&_p]:leading-relaxed', className)}
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</div>
