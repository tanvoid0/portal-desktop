<script lang="ts">
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';
	import type { TrainingData } from '../../types/index.js';

	interface Props {
		data: TrainingData[];
		onView?: (data: TrainingData) => void;
		onEdit?: (data: TrainingData) => void;
		onDelete?: (data: TrainingData) => void;
	}

	let { data, onView, onEdit, onDelete }: Props = $props();
</script>

<div class="rounded-md border">
	<Table>
		<TableHeader>
			<TableRow>
				<TableHead>Name</TableHead>
				<TableHead>Type</TableHead>
				<TableHead>Created</TableHead>
				<TableHead>Updated</TableHead>
				<TableHead class="text-right">Actions</TableHead>
			</TableRow>
		</TableHeader>
		<TableBody>
			{#if data.length === 0}
				<TableRow>
					<TableCell colspan={5} class="text-center text-muted-foreground py-8">
						No training data found
					</TableCell>
				</TableRow>
			{:else}
				{#each data as item}
					<TableRow>
						<TableCell class="font-medium">{item.name}</TableCell>
						<TableCell>
							<span class="px-2 py-1 rounded bg-muted text-xs">{item.type}</span>
						</TableCell>
						<TableCell>{new Date(item.created_at).toLocaleDateString()}</TableCell>
						<TableCell>{new Date(item.updated_at).toLocaleDateString()}</TableCell>
						<TableCell class="text-right">
							<div class="flex items-center justify-end gap-2">
								{#if onView}
									<Button variant="ghost" size="sm" onclick={() => onView(item)}>
										<Icon icon="lucide:eye" class="h-4 w-4" />
									</Button>
								{/if}
								{#if onEdit}
									<Button variant="ghost" size="sm" onclick={() => onEdit(item)}>
										<Icon icon="lucide:edit" class="h-4 w-4" />
									</Button>
								{/if}
								{#if onDelete}
									<Button variant="ghost" size="sm" onclick={() => onDelete(item)}>
										<Icon icon="lucide:trash-2" class="h-4 w-4" />
									</Button>
								{/if}
							</div>
						</TableCell>
					</TableRow>
				{/each}
			{/if}
		</TableBody>
	</Table>
</div>

