<!-- Small badge showing keyboard shortcut hint -->
<script lang="ts">
	import { formatShortcut, parseShortcut } from '../utils/shortcutParser';
	
	interface Props {
		shortcut: string;
		showOnHover?: boolean;
		variant?: 'default' | 'muted' | 'subtle';
	}
	
	let { shortcut, showOnHover = false, variant = 'muted' }: Props = $props();
	
	let show = $state(!showOnHover);
	
	const formatted = $derived(formatShortcut(parseShortcut(shortcut)));
	
	const variantClasses = {
		default: 'bg-background border',
		muted: 'bg-muted',
		subtle: 'bg-muted/50'
	};
</script>

{#if show || !showOnHover}
	<kbd
		class="px-1.5 py-0.5 text-xs font-mono font-semibold rounded border {variantClasses[variant]} {showOnHover
			? 'opacity-0 group-hover:opacity-100 transition-opacity'
			: ''}"
		onmouseenter={() => showOnHover && (show = true)}
		onmouseleave={() => showOnHover && (show = false)}
	>
		{formatted}
	</kbd>
{/if}

