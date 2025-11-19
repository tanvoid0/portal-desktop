<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';
	import ConversationList from '$lib/domains/ai/components/conversations/ConversationList.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { aiConversationService, aiChatService, aiProviderService } from '$lib/domains/ai';
	import type { Conversation, ChatMessage, ConversationMessage, ProviderType } from '$lib/domains/ai/types/index.js';
	import { MessageSquare } from 'lucide-svelte';

	let messages = $state<ChatMessage[]>([]);
	let isLoading = $state(false);
	let conversations = $state<Conversation[]>([]);
	let selectedConversation = $state<Conversation | null>(null);
	let selectedProvider = $state<ProviderType | null>(null);
	let isSending = $state(false);
	let conversationId = $state<string | undefined>(undefined);

	onMount(async () => {
		const defaultProvider = await aiProviderService.getDefaultProvider();
		selectedProvider = defaultProvider || 'Ollama';
		
		// Load conversations first
		await loadConversations();
		
		// Check for conversation ID in URL query params after conversations are loaded
		const urlConversationId = $page.url.searchParams.get('id');
		if (urlConversationId) {
			const conversation = conversations.find(c => c.id === urlConversationId);
			if (conversation) {
				await handleConversationClick(conversation);
			} else {
				// Try loading the conversation directly if not in list
				try {
					const result = await aiConversationService.loadConversation(urlConversationId);
					selectedConversation = result.conversation;
					messages = result.messages.map((msg: ConversationMessage) => ({
						role: msg.role,
						content: msg.content,
						timestamp: new Date(msg.timestamp)
					}));
					conversationId = urlConversationId;
					await loadConversations(); // Reload to include it in the list
				} catch (error) {
					toastActions.error('Failed to load conversation from URL', error);
				}
			}
		}
	});

	async function loadConversations() {
		try {
			conversations = await aiConversationService.listConversations();
		} catch (error) {
			toastActions.error('Failed to load conversations', error);
		}
	}

	async function handleConversationClick(conversation: Conversation) {
		if (selectedConversation?.id === conversation.id) return;
		
		isLoading = true;
		try {
			const result = await aiConversationService.loadConversation(conversation.id);
			selectedConversation = result.conversation;
			messages = result.messages.map((msg: ConversationMessage) => ({
				role: msg.role,
				content: msg.content,
				timestamp: new Date(msg.timestamp)
			}));
			conversationId = conversation.id;
			
			// Update URL without navigation
			const url = new URL(window.location.href);
			url.searchParams.set('id', conversation.id);
			window.history.replaceState({}, '', url.toString());
		} catch (error) {
			toastActions.error('Failed to load conversation', error);
		} finally {
			isLoading = false;
		}
	}

	async function handleDeleteConversation(conversation: Conversation) {
		if (!confirm(`Delete conversation "${conversation.title}"?`)) return;

		try {
			await aiConversationService.deleteConversation(conversation.id);
			toastActions.success('Conversation deleted');
			
			// If deleted conversation was selected, clear selection
			if (selectedConversation?.id === conversation.id) {
				handleNewConversation();
			}
			
			await loadConversations();
		} catch (error) {
			toastActions.error('Failed to delete conversation', error);
		}
	}

	async function handleDeleteAllConversations() {
		if (!confirm(`Delete all ${conversations.length} conversations? This cannot be undone.`)) return;

		try {
			// Delete all conversations one by one
			const deletePromises = conversations.map(conv => 
				aiConversationService.deleteConversation(conv.id)
			);
			await Promise.all(deletePromises);
			
			toastActions.success(`Deleted ${conversations.length} conversations`);
			
			// Clear selection
			handleNewConversation();
			await loadConversations();
		} catch (error) {
			toastActions.error('Failed to delete all conversations', error);
		}
	}

	function handleNewConversation() {
		selectedConversation = null;
		messages = [];
		conversationId = undefined;
		
		// Clear URL parameter
		const url = new URL(window.location.href);
		url.searchParams.delete('id');
		window.history.replaceState({}, '', url.toString());
	}

	async function handleSendMessage(message: string, history: ChatMessage[]) {
		if (!message.trim() || isSending) return;

		const userMessage: ChatMessage = {
			role: 'user',
			content: message.trim(),
			timestamp: new Date()
		};

		messages = [...messages, userMessage];
		isSending = true;

		// Create assistant message placeholder for streaming
		const assistantMessageIndex = messages.length;
		messages = [...messages, {
			role: 'assistant',
			content: '',
			timestamp: new Date()
		}];

		try {
			// If no conversation is selected, create a new one with auto-generated title from first message
			if (!selectedConversation && selectedProvider) {
				// Auto-generate title from first message (truncate to 50 chars)
				const title = message.trim().slice(0, 50) || 'New Conversation';
				const conversation = await aiConversationService.createConversation(title, selectedProvider);
				selectedConversation = conversation;
				conversationId = conversation.id;
				await loadConversations();
			}

			// Use streaming API
			const fullResponse = await aiChatService.streamMessage(message, history, {
				provider: selectedConversation?.provider || selectedProvider || undefined,
				conversation_id: conversationId,
				model: undefined,
				onChunk: (chunk: string) => {
					// Update the assistant message content incrementally
					messages[assistantMessageIndex].content += chunk;
					// Trigger reactivity by creating a new array reference
					messages = [...messages];
				},
				onComplete: (fullMessage: string) => {
					messages[assistantMessageIndex].content = fullMessage;
					messages = [...messages];
				}
			});

			// Ensure final message is set
			messages[assistantMessageIndex].content = fullResponse;
			messages = [...messages];

			// Save conversation if we have one
			if (selectedConversation) {
				await aiConversationService.saveConversation(selectedConversation.id, [
					...messages.map((msg, idx) => ({
						id: `${selectedConversation!.id}-${idx}`,
						conversation_id: selectedConversation!.id,
						role: msg.role,
						content: msg.content,
						timestamp: msg.timestamp?.toISOString() || new Date().toISOString(),
						sequence: idx
					}))
				]);
				await loadConversations();
				
				// Update URL if not already set
				if (!conversationId) {
					const url = new URL(window.location.href);
					url.searchParams.set('id', selectedConversation.id);
					window.history.replaceState({}, '', url.toString());
				}
			}
		} catch (error) {
			toastActions.error('Failed to send message', error);
			// Remove both user and assistant messages on error
			messages = messages.slice(0, -2);
		} finally {
			isSending = false;
		}
	}
</script>

<div class="h-full w-full flex overflow-hidden">
	<!-- Conversation History Sidebar -->
	<aside class="w-64 border-r bg-background flex-shrink-0 flex flex-col overflow-hidden">
		<div class="p-2.5 border-b">
			<h2 class="text-sm font-semibold flex items-center gap-1.5">
				<MessageSquare class="h-4 w-4" />
				Conversations
			</h2>
		</div>
		<ConversationList
			bind:conversations
			onConversationClick={handleConversationClick}
			onCreateNew={handleNewConversation}
			onDeleteConversation={handleDeleteConversation}
			onDeleteAll={handleDeleteAllConversations}
			selectedConversationId={selectedConversation?.id}
		/>
	</aside>

	<!-- Main Chat Area -->
	<main class="flex-1 flex flex-col min-w-0 overflow-hidden">
		<div class="h-full w-full p-6 flex flex-col min-h-0">
			<AIChatPanel
				bind:messages
				isLoading={isSending || isLoading}
				title={selectedConversation?.title || 'AI Chat'}
				placeholder="Ask me anything..."
				class="h-full flex flex-col"
				{conversationId}
				onSendMessageWithHistory={handleSendMessage}
			/>
		</div>
	</main>
</div>

