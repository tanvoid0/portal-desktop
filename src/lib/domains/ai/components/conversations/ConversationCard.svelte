<script lang="ts">
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { MessageSquare, Clock, Trash2 } from 'lucide-svelte';
	import type { Conversation } from '../../types/index.js';

	interface Props {
		conversation: Conversation;
		onClick?: () => void;
		onDelete?: (conversation: Conversation) => void;
		isActive?: boolean;
	}

	let { conversation, onClick, onDelete, isActive = false }: Props = $props();

	function handleDelete(e: MouseEvent) {
		e.stopPropagation();
		onDelete?.(conversation);
	}
</script>

<Card 
	class="cursor-pointer hover:bg-muted/50 transition-colors {isActive ? 'bg-muted border-primary' : ''}"
	onclick={onClick}
>
	<CardContent class="p-3">
		<div class="flex items-start justify-between gap-2 mb-1.5">
			<h3 class="text-sm font-medium line-clamp-1 flex-1 min-w-0">{conversation.title}</h3>
			<div class="flex items-center gap-1 shrink-0">
				<Badge variant="secondary" class="text-xs">
					{conversation.provider}
				</Badge>
				{#if onDelete}
					<Button
						variant="ghost"
						size="icon"
						class="h-6 w-6 text-muted-foreground hover:text-destructive"
						onclick={handleDelete}
						title="Delete conversation"
					>
						<Trash2 class="h-3.5 w-3.5" />
					</Button>
				{/if}
			</div>
		</div>
		<div class="flex items-center gap-3 text-xs text-muted-foreground">
			<div class="flex items-center gap-1">
				<MessageSquare class="h-3 w-3" />
				<span>{conversation.message_count || 0}</span>
			</div>
			<div class="flex items-center gap-1">
				<Clock class="h-3 w-3" />
				<span>{new Date(conversation.updated_at).toLocaleDateString()}</span>
			</div>
		</div>
	</CardContent>
</Card>

