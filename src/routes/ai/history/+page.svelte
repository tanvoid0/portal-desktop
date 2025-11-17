<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import ConversationList from '$lib/domains/ai/components/conversations/ConversationList.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import Select from '$lib/components/ui/select.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { aiConversationService, aiProviderService } from '$lib/domains/ai';
	import type { Conversation, ProviderType } from '$lib/domains/ai/types/index.js';
	import Icon from '@iconify/svelte';

	let conversations = $state<Conversation[]>([]);
	let isLoading = $state(false);
	let showCreateDialog = $state(false);
	let newConversationTitle = $state('');
	let selectedProvider = $state<ProviderType | null>(null);
	
	const providerOptions = [
		{ value: 'Ollama', label: 'Ollama' },
		{ value: 'Gemini', label: 'Gemini' }
	];

	onMount(async () => {
		await loadConversations();
		const defaultProvider = await aiProviderService.getDefaultProvider();
		selectedProvider = defaultProvider || 'Ollama';
	});

	async function loadConversations() {
		isLoading = true;
		try {
			conversations = await aiConversationService.listConversations();
		} catch (error) {
			console.error('Failed to load conversations:', error);
			toastActions.error('Failed to load conversations', error);
		} finally {
			isLoading = false;
		}
	}

	async function handleCreateConversation() {
		if (!newConversationTitle.trim() || !selectedProvider) return;

		try {
			const conversation = await aiConversationService.createConversation(
				newConversationTitle.trim(),
				selectedProvider
			);
			toastActions.success('Conversation created');
			showCreateDialog = false;
			newConversationTitle = '';
			await loadConversations();
			goto(`/ai/history/${conversation.id}`);
		} catch (error) {
			console.error('Failed to create conversation:', error);
			toastActions.error('Failed to create conversation', error);
		}
	}

	function handleConversationClick(conversation: Conversation) {
		goto(`/ai/history/${conversation.id}`);
	}
</script>

<div class="h-full w-full p-6">
	<ConversationList
		bind:conversations
		onConversationClick={handleConversationClick}
		onCreateNew={() => (showCreateDialog = true)}
	/>
</div>

<Dialog.Root bind:open={showCreateDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Create New Conversation</Dialog.Title>
			<Dialog.Description>
				Create a new conversation thread to start chatting with AI.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="title">Title</Label>
				<Input
					id="title"
					bind:value={newConversationTitle}
					placeholder="Enter conversation title..."
				/>
			</div>
			<div class="space-y-2">
				<Label for="provider">Provider</Label>
				<Select
					options={providerOptions}
					value={selectedProvider || undefined}
					onSelect={(value) => (selectedProvider = value as ProviderType)}
					placeholder="Select provider"
				/>
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showCreateDialog = false)}>
				Cancel
			</Button>
			<Button
				onclick={handleCreateConversation}
				disabled={!newConversationTitle.trim() || !selectedProvider}
			>
				<Icon icon="lucide:plus" class="h-4 w-4 mr-2" />
				Create
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

