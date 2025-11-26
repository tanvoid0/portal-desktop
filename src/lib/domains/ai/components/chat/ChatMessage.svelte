<script lang="ts">
	import { Card } from '$lib/components/ui/card';
	import { Bot, User } from 'lucide-svelte';
	import { marked } from 'marked';
	import type { ChatMessage as ChatMessageType } from '../../types/index.js';

	interface Props {
		message: ChatMessageType;
	}

	let { message }: Props = $props();

	// Configure marked options
	marked.setOptions({
		breaks: true, // Convert line breaks to <br>
		gfm: true // Enable GitHub Flavored Markdown
	});

	// Convert markdown to HTML
	function renderMarkdown(content: string): string {
		return marked.parse(content) as string;
	}

	const renderedContent = $derived(renderMarkdown(message.content));
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
			<div
				class="flex-1 text-sm prose prose-sm max-w-none dark:prose-invert {message.role === 'user'
					? 'prose-invert' : ''} 
					prose-headings:mt-2 prose-headings:mb-1 prose-p:my-1 prose-ul:my-1 prose-ol:my-1 
					prose-code:text-xs prose-pre:my-2 prose-pre:p-2 prose-pre:rounded prose-pre:overflow-x-auto
					prose-a:underline prose-strong:font-semibold prose-em:italic"
			>
				{@html renderedContent}
			</div>
		</div>
	</Card>
	{#if message.timestamp}
		<span class="text-xs text-muted-foreground mt-1 px-1">
			{new Date(message.timestamp).toLocaleTimeString()}
		</span>
	{/if}
</div>

