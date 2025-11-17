<script lang="ts">
	import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '$lib/components/ui/collapsible';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';

	interface Props {
		content: string;
		/** Character limit before showing expand/collapse (default: 500) */
		truncateAt?: number;
		/** Whether to start expanded (default: false) */
		defaultExpanded?: boolean;
		/** Custom class for the markdown content */
		class?: string;
	}

	let { content, truncateAt = 500, defaultExpanded = false, class: className = '' }: Props = $props();

	let isExpanded = $state(defaultExpanded);
	let shouldTruncate = $derived(content.length > truncateAt);

	// Simple markdown to HTML converter (handles basic markdown syntax)
	function markdownToHtml(text: string): string {
		// Escape HTML helper
		function escapeHtml(str: string): string {
			return str
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/"/g, '&quot;')
				.replace(/'/g, '&#39;');
		}

		let html = text;

		// Code blocks first (before other processing)
		const codeBlockPlaceholders: string[] = [];
		html = html.replace(/```([\s\S]*?)```/g, (match, code) => {
			const placeholder = `__CODE_BLOCK_${codeBlockPlaceholders.length}__`;
			codeBlockPlaceholders.push(escapeHtml(code));
			return placeholder;
		});

		// Inline code (preserve placeholders)
		html = html.replace(/`([^`\n]+)`/g, '<code class="bg-muted px-1.5 py-0.5 rounded text-sm font-mono">$1</code>');

		// Split into lines for processing
		const lines = html.split('\n');
		const processed: string[] = [];
		let inList = false;
		let listItems: string[] = [];
		let listType: 'ul' | 'ol' = 'ul';

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const trimmed = line.trim();

			// Headers
			if (trimmed.match(/^### /)) {
				flushList();
				processed.push(`<h3 class="text-lg font-semibold mt-4 mb-2">${escapeHtml(trimmed.substring(4))}</h3>`);
				continue;
			}
			if (trimmed.match(/^## /)) {
				flushList();
				processed.push(`<h2 class="text-xl font-semibold mt-5 mb-3">${escapeHtml(trimmed.substring(3))}</h2>`);
				continue;
			}
			if (trimmed.match(/^# /)) {
				flushList();
				processed.push(`<h1 class="text-2xl font-bold mt-6 mb-4">${escapeHtml(trimmed.substring(2))}</h1>`);
				continue;
			}

			// Unordered lists
			if (trimmed.match(/^[\*\-+] /)) {
				if (!inList || listType !== 'ul') {
					flushList();
					inList = true;
					listType = 'ul';
				}
				listItems.push(`<li class="ml-4">${escapeHtml(trimmed.substring(2))}</li>`);
				continue;
			}

			// Ordered lists
			if (trimmed.match(/^\d+\. /)) {
				if (!inList || listType !== 'ol') {
					flushList();
					inList = true;
					listType = 'ol';
				}
				const content = trimmed.replace(/^\d+\. /, '');
				listItems.push(`<li class="ml-4">${escapeHtml(content)}</li>`);
				continue;
			}

			// Blockquotes
			if (trimmed.match(/^> /)) {
				flushList();
				processed.push(`<blockquote class="border-l-4 border-muted-foreground/30 pl-4 my-2 italic">${escapeHtml(trimmed.substring(2))}</blockquote>`);
				continue;
			}

			// Horizontal rules
			if (trimmed.match(/^(---|\*\*\*)$/)) {
				flushList();
				processed.push('<hr class="my-4 border-border" />');
				continue;
			}

			// Regular line
			flushList();
			if (trimmed) {
				processed.push(`<p class="mb-3 leading-relaxed">${escapeHtml(trimmed)}</p>`);
			} else {
				processed.push('');
			}
		}

		flushList();

		function flushList() {
			if (inList && listItems.length > 0) {
				const listClass = listType === 'ul' 
					? 'list-disc list-inside my-2 space-y-1' 
					: 'list-decimal list-inside my-2 space-y-1';
				processed.push(`<${listType} class="${listClass}">${listItems.join('')}</${listType}>`);
				listItems = [];
				inList = false;
			}
		}

		html = processed.join('\n');

		// Restore code blocks
		codeBlockPlaceholders.forEach((code, index) => {
			html = html.replace(`__CODE_BLOCK_${index}__`, `<pre class="bg-muted p-3 rounded-md overflow-x-auto my-2"><code>${code}</code></pre>`);
		});

		// Process inline formatting (after escaping)
		// Bold (must be before italic to avoid conflicts)
		html = html.replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold">$1</strong>');
		html = html.replace(/__(.*?)__/g, '<strong class="font-semibold">$1</strong>');

		// Italic (avoid matching bold)
		html = html.replace(/(?<!\*)\*(?!\*)([^*]+?)\*(?!\*)/g, '<em class="italic">$1</em>');
		html = html.replace(/(?<!_)_(?!_)([^_]+?)_(?!_)/g, '<em class="italic">$1</em>');

		// Links
		html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-primary underline hover:text-primary/80" target="_blank" rel="noopener noreferrer">$1</a>');

		return html;
	}

	const truncatedContent = $derived(shouldTruncate && !isExpanded ? content.substring(0, truncateAt) + '...' : content);
	const displayContent = $derived(markdownToHtml(truncatedContent));
</script>

<div class="space-y-2">
	{#if shouldTruncate}
		<Collapsible bind:open={isExpanded}>
			<div class="prose prose-sm max-w-none dark:prose-invert {className}">
				{@html displayContent}
			</div>
			<div class="mt-2">
				<CollapsibleTrigger asChild let:builder>
					<Button variant="ghost" size="sm" builders={[builder]} class="h-auto p-0 text-muted-foreground hover:text-foreground">
						{#if isExpanded}
							<Icon icon="lucide:chevron-up" class="h-4 w-4 mr-1" />
							Show less
						{:else}
							<Icon icon="lucide:chevron-down" class="h-4 w-4 mr-1" />
							Show more
						{/if}
					</Button>
				</CollapsibleTrigger>
			</div>
		</Collapsible>
	{:else}
		<div class="prose prose-sm max-w-none dark:prose-invert {className}">
			{@html displayContent}
		</div>
	{/if}
</div>

