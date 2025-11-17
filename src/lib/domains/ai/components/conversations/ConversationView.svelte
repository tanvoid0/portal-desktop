<script lang="ts">
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import ChatMessage from '../chat/ChatMessage.svelte';
	import ChatInput from '../chat/ChatInput.svelte';
	import ConversationHeader from './ConversationHeader.svelte';
	import type { Conversation, ChatMessage as ChatMessageType } from '../../types/index.js';

	interface Props {
		conversation: Conversation;
		messages: ChatMessageType[];
		isLoading?: boolean;
		selectedModel?: string | null;
		onTitleChange?: (title: string) => void;
		onDelete?: () => void;
		onBack?: () => void;
		onSendMessage?: (message: string) => void | Promise<void>;
		onModelChange?: (model: string) => void;
	}

	let {
		conversation,
		messages = $bindable<ChatMessageType[]>([]),
		isLoading = false,
		selectedModel = $bindable<string | null>(null),
		onTitleChange,
		onDelete,
		onBack,
		onSendMessage,
		onModelChange
	}: Props = $props();

	let messageInput = $state('');
	let messagesContainer: HTMLDivElement;

	async function handleSend() {
		if (!messageInput.trim() || isLoading) return;
		const currentMessage = messageInput.trim();
		messageInput = '';
		if (onSendMessage) {
			await onSendMessage(currentMessage);
		}
	}

	$effect(() => {
		if (messagesContainer && messages.length > 0) {
			setTimeout(() => {
				messagesContainer.scrollTop = messagesContainer.scrollHeight;
			}, 100);
		}
	});
</script>

<div class="flex flex-col h-full">
	<ConversationHeader
		{conversation}
		bind:selectedModel
		{onTitleChange}
		{onDelete}
		{onBack}
		{onModelChange}
	/>
	<ScrollArea class="flex-1 px-4" bind:this={messagesContainer}>
		<div class="space-y-4 py-4">
			{#if messages.length === 0}
				<div class="text-center text-muted-foreground py-8">
					<p class="text-sm">No messages yet. Start the conversation!</p>
				</div>
			{:else}
				{#each messages as message}
					<ChatMessage {message} />
				{/each}
			{/if}
			{#if isLoading}
				<div class="flex items-start">
					<div class="bg-muted px-4 py-2 rounded-lg">
						<div class="flex gap-1">
							<span class="w-2 h-2 rounded-full bg-foreground/50 animate-pulse"></span>
							<span class="w-2 h-2 rounded-full bg-foreground/50 animate-pulse" style="animation-delay: 0.2s"></span>
							<span class="w-2 h-2 rounded-full bg-foreground/50 animate-pulse" style="animation-delay: 0.4s"></span>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</ScrollArea>
	<ChatInput
		bind:value={messageInput}
		onSend={handleSend}
		placeholder="Continue the conversation..."
		disabled={isLoading}
	/>
</div>

