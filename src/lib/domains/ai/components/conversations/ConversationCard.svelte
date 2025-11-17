<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import Icon from '@iconify/svelte';
	import type { Conversation } from '../../types/index.js';

	interface Props {
		conversation: Conversation;
		onClick?: () => void;
	}

	let { conversation, onClick }: Props = $props();
</script>

<Card class="cursor-pointer hover:bg-muted/50 transition-colors" onclick={onClick}>
	<CardHeader>
		<div class="flex items-start justify-between">
			<CardTitle class="text-base line-clamp-2">{conversation.title}</CardTitle>
			<Badge variant="secondary" class="ml-2 shrink-0">
				{conversation.provider}
			</Badge>
		</div>
	</CardHeader>
	<CardContent>
		<div class="flex items-center gap-4 text-sm text-muted-foreground">
			<div class="flex items-center gap-1">
				<Icon icon="lucide:message-square" class="h-4 w-4" />
				{conversation.message_count || 0} messages
			</div>
			<div class="flex items-center gap-1">
				<Icon icon="lucide:clock" class="h-4 w-4" />
				{new Date(conversation.updated_at).toLocaleDateString()}
			</div>
		</div>
	</CardContent>
</Card>

