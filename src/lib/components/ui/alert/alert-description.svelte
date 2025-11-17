<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLAttributes } from 'svelte/elements';
	
	interface AlertDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {}
	
	let {
		class: className,
		children,
		...restProps
	}: AlertDescriptionProps = $props();
</script>

<div
	class={cn('text-sm [&_p]:leading-relaxed', className)}
	{...restProps}
>
	{@render children()}
</div>
