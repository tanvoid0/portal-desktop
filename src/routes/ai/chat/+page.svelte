<script lang="ts">
	import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';

	let messages = $state<Array<{ role: 'user' | 'assistant'; content: string; timestamp?: Date }>>([]);
	let isLoading = $state(false);
</script>

<div class="h-full w-full p-6">
	<AIChatPanel
		bind:messages
		bind:isLoading
		title="AI Chat"
		placeholder="Ask me anything..."
		class="h-full"
	/>
</div>

