<script lang="ts">
	import { Textarea } from '$lib/components/ui/textarea';
	import { Button } from '$lib/components/ui/button';
	import { Send } from 'lucide-svelte';

	interface Props {
		value: string;
		onValueChange?: (value: string) => void;
		onSend: () => void;
		placeholder?: string;
		disabled?: boolean;
	}

	let {
		value = $bindable(''),
		onValueChange,
		onSend,
		placeholder = 'Type your message...',
		disabled = false
	}: Props = $props();

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			onSend();
		}
	}

	$effect(() => {
		if (onValueChange) {
			onValueChange(value);
		}
	});
</script>

<div class="border-t p-4">
	<div class="flex gap-2">
		<Textarea
			bind:value={value}
			placeholder={placeholder}
			rows={3}
			class="resize-none"
			onkeydown={handleKeydown}
			disabled={disabled}
		/>
		<Button
			onclick={onSend}
			disabled={!value.trim() || disabled}
			class="self-end"
			size="sm"
		>
			<Send class="h-4 w-4" />
		</Button>
	</div>
	<p class="text-xs text-muted-foreground mt-2">
		Press Enter to send, Shift+Enter for new line
	</p>
</div>

