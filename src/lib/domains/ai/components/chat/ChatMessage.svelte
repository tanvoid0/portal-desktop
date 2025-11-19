<script lang="ts">
	import { Card } from '$lib/components/ui/card';
	import { Bot, User } from 'lucide-svelte';
	import type { ChatMessage as ChatMessageType } from '../../types/index.js';

	interface Props {
		message: ChatMessageType;
	}

	let { message }: Props = $props();
</script>

<div class="flex flex-col {message.role === 'user' ? 'items-end' : 'items-start'}">
	<Card
		class="max-w-[85%] px-4 py-2 {message.role === 'user'
			? 'bg-primary text-primary-foreground'
			: 'bg-muted'}"
	>
		<div class="flex items-start gap-2">
			{#if message.role === 'assistant'}
				<Bot class="h-4 w-4 mt-0.5 shrink-0" />
			{:else}
				<User class="h-4 w-4 mt-0.5 shrink-0" />
			{/if}
			<div class="flex-1 text-sm whitespace-pre-wrap">{message.content}</div>
		</div>
	</Card>
	{#if message.timestamp}
		<span class="text-xs text-muted-foreground mt-1 px-1">
			{new Date(message.timestamp).toLocaleTimeString()}
		</span>
	{/if}
</div>

