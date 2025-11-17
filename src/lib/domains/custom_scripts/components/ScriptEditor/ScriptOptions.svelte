<!--
	Script Options Component
	Handles requiresSudo and isInteractive checkboxes
-->

<script lang="ts">
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';

	interface Props {
		requiresSudo: boolean;
		isInteractive: boolean;
		onRequiresSudoChange: (value: boolean) => void;
		onIsInteractiveChange: (value: boolean) => void;
	}

	let {
		requiresSudo,
		isInteractive,
		onRequiresSudoChange,
		onIsInteractiveChange
	}: Props = $props();
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<div class="space-y-0.5">
			<Label>Requires Sudo</Label>
			<p class="text-sm text-muted-foreground">
				Automatically prepend sudo to the command (Linux/Mac) or note admin requirement (Windows)
			</p>
		</div>
		<Switch checked={requiresSudo} onCheckedChange={onRequiresSudoChange} />
	</div>

	{#if requiresSudo}
		<div class="p-3 border border-amber-200 bg-amber-50 dark:bg-amber-950 dark:border-amber-800 rounded-md">
			<p class="text-sm text-amber-800 dark:text-amber-200">
				<strong>Note:</strong> You don't need to include "sudo" in your command template. It will be added automatically.
			</p>
		</div>
	{/if}

	<div class="flex items-center justify-between">
		<div class="space-y-0.5">
			<Label>Interactive</Label>
			<p class="text-sm text-muted-foreground">
				Script requires user input during execution (e.g., password prompts)
			</p>
		</div>
		<Switch checked={isInteractive} onCheckedChange={onIsInteractiveChange} />
	</div>
</div>

