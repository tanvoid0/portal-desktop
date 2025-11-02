<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/lib/components/ui/card';
	import { Button } from '@/lib/components/ui/button';
	import { Input } from '@/lib/components/ui/input';
	import { Label } from '@/lib/components/ui/label';
	import { Textarea } from '@/lib/components/ui/textarea';

	interface Props {
		environment: Record<string, string>;
		onSave?: (env: Record<string, string>) => void;
		onCancel?: () => void;
	}

	let { environment, onSave, onCancel }: Props = $props();

	let envVars = $state(Object.entries(environment || {}));

	function addVariable() {
		envVars = [...envVars, ['', '']];
	}

	function removeVariable(index: number) {
		envVars = envVars.filter((_, i) => i !== index);
	}

	function handleSave() {
		const env = Object.fromEntries(envVars.filter(([key]) => key.trim()));
		onSave?.(env);
	}
</script>

<Card class="w-full">
	<CardHeader>
		<CardTitle>Environment Variables</CardTitle>
		<CardDescription>Configure environment variables for your deployment</CardDescription>
	</CardHeader>
	<CardContent class="space-y-4">
		{#each envVars as [key, value], index}
			<div class="flex gap-2 items-center">
				<div class="flex-1">
					<Label for="key-{index}">Key</Label>
					<Input id="key-{index}" bind:value={envVars[index][0]} placeholder="VARIABLE_NAME" />
				</div>
				<div class="flex-1">
					<Label for="value-{index}">Value</Label>
					<Input id="value-{index}" bind:value={envVars[index][1]} placeholder="value" />
				</div>
				<Button variant="outline" size="sm" onclick={() => removeVariable(index)}>
					Remove
				</Button>
			</div>
		{/each}
		
		<Button variant="outline" onclick={addVariable}>
			Add Variable
		</Button>
		
		<div class="flex gap-2">
			<Button onclick={handleSave}>
				Save Environment
			</Button>
			<Button variant="outline" onclick={onCancel}>
				Cancel
			</Button>
		</div>
	</CardContent>
</Card>
