<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { MessageSquare, MessageCircle } from 'lucide-svelte';
	import ChatMessage from './ChatMessage.svelte';
	import ChatInput from './ChatInput.svelte';
	import ChatProviderSelector from './ChatProviderSelector.svelte';
	import ChatModelSelector from './ChatModelSelector.svelte';
	import type { ChatMessage as ChatMessageType, ProviderType } from '../../types/index.js';
	import { aiChatService } from '../../services/aiChatService.js';

	interface Props {
		messages?: ChatMessageType[];
		onSendMessage?: (message: string) => void | Promise<void>;
		onSendMessageWithHistory?: (message: string, history: ChatMessageType[]) => void | Promise<void>;
		isLoading?: boolean;
		placeholder?: string;
		title?: string;
		class?: string;
		conversationId?: string;
	}

	let {
		messages = $bindable<ChatMessageType[]>([]),
		onSendMessage,
		onSendMessageWithHistory,
		isLoading = $bindable(false),
		placeholder = 'Type your message...',
		title = 'Chat',
		class: className = '',
		conversationId
	}: Props = $props();

	let messageInput = $state('');
	let messagesContainer: HTMLElement | null = $state(null);
	let scrollViewport: HTMLElement | null = $state(null);
	let selectedProvider = $state<ProviderType | null>(null);
	let selectedModel = $state<string | null>(null);

	async function handleSend() {
		if (!messageInput.trim() || isLoading) return;

		const currentMessage = messageInput.trim();
		messageInput = '';

		if (onSendMessageWithHistory) {
			// Don't add message here - parent will handle it
			isLoading = true;
			try {
				await onSendMessageWithHistory(currentMessage, messages);
			} finally {
				isLoading = false;
			}
		} else if (onSendMessage) {
			// Don't add message here - parent will handle it
			isLoading = true;
			try {
				await onSendMessage(currentMessage);
			} finally {
				isLoading = false;
			}
		} else {
			// Use default AI chat service - only add message here if no callback provided
			const userMessage: ChatMessageType = {
				role: 'user',
				content: currentMessage,
				timestamp: new Date()
			};
			messages = [...messages, userMessage];
			
			isLoading = true;
			try {
				const response = await aiChatService.sendMessage(currentMessage, messages, {
					provider: selectedProvider || undefined,
					conversation_id: conversationId,
					model: selectedModel || undefined
				});
				const assistantMessage: ChatMessageType = {
					role: 'assistant',
					content: response,
					timestamp: new Date()
				};
				messages = [...messages, assistantMessage];
			} catch (error) {
				console.error('Failed to send message:', error);
				// Remove user message on error
				messages = messages.slice(0, -1);
			} finally {
				isLoading = false;
			}
		}
	}

	$effect(() => {
		// Scroll to bottom when messages change
		if (scrollViewport && messages.length > 0) {
			// Use requestAnimationFrame for smoother scrolling
			requestAnimationFrame(() => {
				if (scrollViewport) {
					scrollViewport.scrollTop = scrollViewport.scrollHeight;
				}
			});
		}
	});
</script>

<Card class="h-full flex flex-col {className}">
	<CardHeader class="pb-3">
		<div class="flex items-center justify-between">
			<CardTitle class="text-lg flex items-center gap-2">
				<MessageSquare class="h-5 w-5" />
				{title}
			</CardTitle>
			<div class="flex items-center gap-2">
				<ChatProviderSelector bind:selectedProvider />
				{#if selectedProvider}
					<ChatModelSelector bind:selectedProvider bind:selectedModel />
				{/if}
			</div>
		</div>
	</CardHeader>
	<CardContent class="flex-1 flex flex-col p-0 min-h-0 overflow-hidden">
		<ScrollArea class="flex-1 min-h-0" bind:viewportRef={scrollViewport}>
			<div class="px-4 py-4 space-y-4" bind:this={messagesContainer}>
				{#if messages.length === 0}
					<div class="text-center text-muted-foreground py-8">
						<MessageCircle class="h-12 w-12 mx-auto mb-2 opacity-50" />
						<p class="text-sm">Start a conversation</p>
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
		<div class="flex-shrink-0 border-t bg-background">
			<ChatInput
				bind:value={messageInput}
				onSend={handleSend}
				{placeholder}
				disabled={isLoading}
			/>
		</div>
	</CardContent>
</Card>

