<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';
	import type { Document } from '../types';

	interface Props {
		document: Document;
		onClick?: () => void;
		onDelete?: () => void;
	}

	let { document, onClick, onDelete }: Props = $props();

	const preview = $derived(
		document.content.length > 150
			? document.content.substring(0, 150) + '...'
			: document.content
	);
</script>

<Card class="cursor-pointer hover:shadow-md transition-shadow" onclick={onClick}>
	<CardHeader class="pb-3">
		<div class="flex items-start justify-between gap-2">
			<CardTitle class="text-lg line-clamp-2">{document.title}</CardTitle>
			{#if onDelete}
				<Button
					variant="ghost"
					size="sm"
					class="h-8 w-8 p-0"
					onclick={(e) => {
						e.stopPropagation();
						onDelete?.();
					}}
				>
					<Icon icon="lucide:trash-2" class="h-4 w-4" />
				</Button>
			{/if}
		</div>
	</CardHeader>
	<CardContent class="space-y-3">
		<p class="text-sm text-muted-foreground line-clamp-3">{preview}</p>
		<div class="flex items-center justify-between">
			<div class="flex flex-wrap gap-1">
				{#if document.tags && document.tags.length > 0}
					{#each document.tags.slice(0, 3) as tag}
						<Badge variant="secondary" class="text-xs">{tag}</Badge>
					{/each}
					{#if document.tags.length > 3}
						<Badge variant="outline" class="text-xs">+{document.tags.length - 3}</Badge>
					{/if}
				{/if}
			</div>
			<div class="flex items-center gap-2 text-xs text-muted-foreground">
				{#if document.isDraft}
					<Badge variant="outline" class="text-xs">Draft</Badge>
				{/if}
				{#if document.isArchived}
					<Badge variant="secondary" class="text-xs">Archived</Badge>
				{/if}
				<Icon icon="lucide:clock" class="h-3 w-3" />
				{new Date(document.updatedAt).toLocaleDateString()}
			</div>
		</div>
	</CardContent>
</Card>

