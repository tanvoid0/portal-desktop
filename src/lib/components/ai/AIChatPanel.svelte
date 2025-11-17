<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import Icon from '@iconify/svelte';

	export interface ChatMessage {
		role: 'user' | 'assistant';
		content: string;
		timestamp?: Date;
	}

	interface Props {
		messages?: ChatMessage[];
		onSendMessage?: (message: string) => void | Promise<void>;
		onSendMessageWithHistory?: (message: string, history: ChatMessage[]) => void | Promise<void>;
		isLoading?: boolean;
		placeholder?: string;
		title?: string;
		class?: string;
	}

	let {
		messages = $bindable<ChatMessage[]>([]),
		onSendMessage,
		onSendMessageWithHistory,
		isLoading = $bindable(false),
		placeholder = 'Type your message...',
		title = 'Chat',
		class: className = ''
	}: Props = $props();

	let messageInput = $state('');
	let messagesContainer: HTMLDivElement;

	async function handleSend() {
		if (!messageInput.trim() || isLoading) return;

		const userMessage: ChatMessage = {
			role: 'user',
			content: messageInput.trim(),
			timestamp: new Date()
		};

		// Add user message immediately
		messages = [...messages, userMessage];
		const currentMessage = messageInput.trim();
		messageInput = '';

		// Call the appropriate handler
		if (onSendMessageWithHistory) {
			isLoading = true;
			try {
				await onSendMessageWithHistory(currentMessage, messages);
			} finally {
				isLoading = false;
			}
		} else if (onSendMessage) {
			isLoading = true;
			try {
				await onSendMessage(currentMessage);
			} finally {
				isLoading = false;
			}
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			handleSend();
		}
	}

	// Auto-scroll when new messages arrive
	$effect(() => {
		if (messagesContainer && messages.length > 0) {
			setTimeout(() => {
				messagesContainer.scrollTop = messagesContainer.scrollHeight;
			}, 100);
		}
	});
</script>

<Card class="h-full flex flex-col {className}">
	<CardHeader class="pb-3">
		<CardTitle class="text-lg flex items-center gap-2">
			<Icon icon="lucide:message-square" class="h-5 w-5" />
			{title}
		</CardTitle>
	</CardHeader>
	<CardContent class="flex-1 flex flex-col p-0 min-h-0">
		<ScrollArea class="flex-1 px-4" bind:this={messagesContainer}>
			<div class="space-y-4 py-4">
				{#if messages.length === 0}
					<div class="text-center text-muted-foreground py-8">
						<Icon icon="lucide:message-circle" class="h-12 w-12 mx-auto mb-2 opacity-50" />
						<p class="text-sm">Start a conversation</p>
					</div>
				{:else}
					{#each messages as message}
						<div class="flex flex-col {message.role === 'user' ? 'items-end' : 'items-start'}">
							<div
								class="max-w-[85%] px-4 py-2 rounded-lg text-sm {message.role === 'user'
									? 'bg-primary text-primary-foreground'
									: 'bg-muted text-foreground'}"
							>
								{message.content}
							</div>
							{#if message.timestamp}
								<span class="text-xs text-muted-foreground mt-1 px-1">
									{new Date(message.timestamp).toLocaleTimeString()}
								</span>
							{/if}
						</div>
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
		<div class="border-t p-4">
			<div class="flex gap-2">
				<Textarea
					bind:value={messageInput}
					placeholder={placeholder}
					rows={3}
					class="resize-none"
					onkeydown={handleKeydown}
					disabled={isLoading}
				/>
				<Button
					onclick={handleSend}
					disabled={!messageInput.trim() || isLoading}
					class="self-end"
					size="sm"
				>
					<Icon icon="lucide:send" class="h-4 w-4" />
				</Button>
			</div>
			<p class="text-xs text-muted-foreground mt-2">
				Press Enter to send, Shift+Enter for new line
			</p>
		</div>
	</CardContent>
</Card>

