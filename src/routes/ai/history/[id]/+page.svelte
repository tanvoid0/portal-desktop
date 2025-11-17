<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import ConversationView from '$lib/domains/ai/components/conversations/ConversationView.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { aiConversationService, aiChatService } from '$lib/domains/ai';
	import type { Conversation, ChatMessage, ConversationMessage } from '$lib/domains/ai/types/index.js';

	let conversation = $state<Conversation | null>(null);
	let messages = $state<ChatMessage[]>([]);
	let isLoading = $state(false);
	let isSending = $state(false);
	let selectedModel = $state<string | null>(null);

	$effect(() => {
		const conversationId = $page.params.id;
		if (conversationId) {
			loadConversation(conversationId);
		}
	});

	async function loadConversation(id: string) {
		isLoading = true;
		try {
			const result = await aiConversationService.loadConversation(id);
			conversation = result.conversation;
			messages = result.messages.map((msg: ConversationMessage) => ({
				role: msg.role,
				content: msg.content,
				timestamp: new Date(msg.timestamp)
			}));
		} catch (error) {
			console.error('Failed to load conversation:', error);
			toastActions.error('Failed to load conversation', error);
			goto('/ai/history');
		} finally {
			isLoading = false;
		}
	}

	async function handleSendMessage(message: string) {
		if (!conversation || !message.trim()) return;

		isSending = true;
		const userMessage: ChatMessage = {
			role: 'user',
			content: message.trim(),
			timestamp: new Date()
		};

		messages = [...messages, userMessage];

		try {
			const response = await aiChatService.sendMessage(message, messages, {
				provider: conversation.provider,
				conversation_id: conversation.id,
				model: selectedModel || undefined
			});

			const assistantMessage: ChatMessage = {
				role: 'assistant',
				content: response,
				timestamp: new Date()
			};

			messages = [...messages, assistantMessage];

			// Save conversation
			await aiConversationService.saveConversation(conversation.id, [
				...messages.map((msg, idx) => ({
					id: `${conversation!.id}-${idx}`,
					conversation_id: conversation!.id,
					role: msg.role,
					content: msg.content,
					timestamp: msg.timestamp?.toISOString() || new Date().toISOString(),
					sequence: idx
				}))
			]);
		} catch (error) {
			console.error('Failed to send message:', error);
			toastActions.error('Failed to send message', error);
			messages = messages.slice(0, -1); // Remove user message on error
		} finally {
			isSending = false;
		}
	}

	async function handleTitleChange(title: string) {
		if (!conversation) return;
		try {
			await aiConversationService.updateConversationTitle(conversation.id, title);
			conversation = { ...conversation, title };
			toastActions.success('Title updated');
		} catch (error) {
			console.error('Failed to update title:', error);
			toastActions.error('Failed to update title', error);
		}
	}

	async function handleDelete() {
		if (!conversation) return;
		try {
			await aiConversationService.deleteConversation(conversation.id);
			toastActions.success('Conversation deleted');
			goto('/ai/history');
		} catch (error) {
			console.error('Failed to delete conversation:', error);
			toastActions.error('Failed to delete conversation', error);
		}
	}

	function handleBack() {
		goto('/ai/history');
	}
</script>

{#if isLoading}
	<div class="h-full w-full p-6 flex items-center justify-center">
		<p class="text-muted-foreground">Loading conversation...</p>
	</div>
{:else if conversation}
	<div class="h-full w-full p-6">
		<ConversationView
			{conversation}
			bind:messages
			bind:selectedModel
			isLoading={isSending}
			onTitleChange={handleTitleChange}
			onDelete={handleDelete}
			onBack={handleBack}
			onSendMessage={handleSendMessage}
		/>
	</div>
{/if}

