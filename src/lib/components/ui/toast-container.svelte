<!--
	Toast Container Component
	Renders all active toast notifications
-->

<script lang="ts">
	import { toastStore } from '$lib/stores/toast';
	import { cn } from '@/lib/utils';
	import Toast from './toast.svelte';

	interface Props {
		position?: 'top-left' | 'top-center' | 'top-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';
		class?: string;
	}

	let {
		position = 'top-right',
		class: className = ''
	}: Props = $props();

	// Position configurations
	const positionConfig = {
		'top-left': 'top-4 left-4',
		'top-center': 'top-4 left-1/2 -translate-x-1/2',
		'top-right': 'top-4 right-4',
		'bottom-left': 'bottom-4 left-4',
		'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2',
		'bottom-right': 'bottom-4 right-4'
	};

	const positionClasses = positionConfig[position];
</script>

<!-- Toast Container -->
<div
	class={cn(
		'fixed z-50 flex flex-col gap-2 max-w-sm w-full',
		positionClasses,
		className
	)}
	aria-live="polite"
	aria-label="Notifications"
>
	{#each $toastStore.toasts as toast (toast.id)}
		<Toast
			{...toast}
			onClose={() => toastStore.remove(toast.id)}
		/>
	{/each}
</div>