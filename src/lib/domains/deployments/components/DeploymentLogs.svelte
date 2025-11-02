<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { ScrollArea } from '@/lib/components/ui/scroll-area';

	interface Props {
		deploymentId: string;
		logs: string[];
		onRefresh?: () => void;
	}

	let { deploymentId, logs, onRefresh }: Props = $props();
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle>Deployment Logs</CardTitle>
				<CardDescription>Logs for deployment {deploymentId}</CardDescription>
			</div>
			<Button variant="outline" size="sm" onclick={onRefresh}>
				Refresh
			</Button>
		</div>
	</CardHeader>
	<CardContent>
		<ScrollArea class="h-96 w-full">
			<div class="font-mono text-sm">
				{#each logs as log}
					<div class="py-1">{log}</div>
				{/each}
			</div>
		</ScrollArea>
	</CardContent>
</Card>
