<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import ConversationCard from './ConversationCard.svelte';
	import Icon from '@iconify/svelte';
	import type { Conversation } from '../../types/index.js';

	interface Props {
		conversations: Conversation[];
		onConversationClick?: (conversation: Conversation) => void;
		onCreateNew?: () => void;
	}

	let {
		conversations = $bindable<Conversation[]>([]),
		onConversationClick,
		onCreateNew
	}: Props = $props();

	let searchQuery = $state('');
	let filteredConversations = $derived(
		conversations.filter((conv) =>
			conv.title.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);
</script>

<div class="flex flex-col h-full">
	<div class="flex items-center gap-2 p-4 border-b">
		<Input
			placeholder="Search conversations..."
			bind:value={searchQuery}
			class="flex-1"
		/>
		<Button onclick={onCreateNew}>
			<Icon icon="lucide:plus" class="h-4 w-4 mr-2" />
			New Conversation
		</Button>
	</div>
	<ScrollArea class="flex-1">
		<div class="p-4 space-y-2">
			{#if filteredConversations.length === 0}
				<div class="text-center text-muted-foreground py-8">
					<Icon icon="lucide:message-circle" class="h-12 w-12 mx-auto mb-2 opacity-50" />
					<p class="text-sm">
						{searchQuery ? 'No conversations found' : 'No conversations yet'}
					</p>
				</div>
			{:else}
				{#each filteredConversations as conversation}
					<ConversationCard
						{conversation}
						onClick={() => onConversationClick?.(conversation)}
					/>
				{/each}
			{/if}
		</div>
	</ScrollArea>
</div>

