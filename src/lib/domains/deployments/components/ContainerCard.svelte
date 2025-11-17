<script lang="ts">
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { Badge } from '@/lib/components/ui/badge';
	import { Button } from '@/lib/components/ui/button';
	import type { DockerContainer } from '../types';
	import { Play, Square, Trash2, ExternalLink } from '@lucide/svelte';

	interface Props {
		container: DockerContainer;
		onStart?: (id: string) => void;
		onStop?: (id: string) => void;
		onRemove?: (id: string) => void;
	}

	let { container, onStart, onStop, onRemove }: Props = $props();

	function isRunning(): boolean {
		const status = container.status?.toLowerCase() || '';
		return status.includes('running') || status.includes('up');
	}

	function getStatusColor(): string {
		if (isRunning()) {
			return 'bg-green-100 text-green-800 border-green-200';
		}
		return 'bg-gray-100 text-gray-800 border-gray-200';
	}

	function formatPorts(): string {
		if (Array.isArray(container.ports)) {
			if (container.ports.length === 0) return 'No ports';
			// Handle string array or object array
			return container.ports.map((p: any) => {
				if (typeof p === 'string') return p;
				return `${p.hostPort || ''}:${p.containerPort || ''}`;
			}).join(', ');
		}
		if (typeof container.ports === 'string') {
			return container.ports || 'No ports';
		}
		return 'No ports';
	}
</script>

<Card class="w-full hover:shadow-md transition-shadow">
	<CardContent class="p-4">
		<div class="flex items-start justify-between gap-4">
			<div class="flex-1 min-w-0">
				<div class="flex items-center gap-2 mb-2">
					<h3 class="font-semibold text-lg truncate" title={container.name}>
						{container.name}
					</h3>
					<Badge class={getStatusColor()}>
						{isRunning() ? 'Running' : 'Stopped'}
					</Badge>
				</div>
				
				<div class="space-y-1 text-sm text-muted-foreground">
					<div class="flex items-center gap-2">
						<span class="font-medium">Image:</span>
						<span class="font-mono text-xs truncate" title={container.image}>
							{container.image}
						</span>
					</div>
					
					{#if container.ports && (Array.isArray(container.ports) ? container.ports.length > 0 : container.ports)}
						<div class="flex items-center gap-2">
							<span class="font-medium">Ports:</span>
							<span class="font-mono text-xs">{formatPorts()}</span>
						</div>
					{/if}
					
					{#if container.createdAt}
						{@const createdDate = container.createdAt instanceof Date ? container.createdAt : new Date(container.createdAt)}
						{@const isValidDate = !isNaN(createdDate.getTime())}
						{#if isValidDate}
							<div class="flex items-center gap-2">
								<span class="font-medium">Created:</span>
								<span class="text-xs">
									{createdDate.toLocaleDateString()} {createdDate.toLocaleTimeString()}
								</span>
							</div>
						{/if}
					{/if}
					
					<div class="flex items-center gap-2">
						<span class="font-medium">ID:</span>
						<span class="font-mono text-xs truncate" title={container.id}>
							{container.id.substring(0, 12)}...
						</span>
					</div>
				</div>
			</div>
			
			<div class="flex flex-col gap-2">
				{#if isRunning()}
					<Button variant="outline" size="sm" onclick={() => onStop?.(container.id)}>
						<Square class="h-4 w-4 mr-1" />
						Stop
					</Button>
				{:else}
					<Button variant="default" size="sm" onclick={() => onStart?.(container.id)}>
						<Play class="h-4 w-4 mr-1" />
						Start
					</Button>
				{/if}
				<Button variant="destructive" size="sm" onclick={() => onRemove?.(container.id)}>
					<Trash2 class="h-4 w-4 mr-1" />
					Remove
				</Button>
			</div>
		</div>
	</CardContent>
</Card>

