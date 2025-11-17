<!--
	Parameter List Component
	Manages the list of script parameters
-->

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Plus } from '@lucide/svelte';
	import ParameterEditor from './ParameterEditor.svelte';
	import type { ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';

	interface Props {
		parameters: ScriptParameter[];
		onParametersChange: (parameters: ScriptParameter[]) => void;
	}

	let { parameters, onParametersChange }: Props = $props();

	function addParameter() {
		onParametersChange([
			...parameters,
			{
				name: `param${parameters.length + 1}`,
				label: `Parameter ${parameters.length + 1}`,
				parameter_type: 'string',
				required: false,
			},
		]);
	}

	function removeParameter(index: number) {
		onParametersChange(parameters.filter((_param, i) => i !== index));
	}

	function updateParameter(index: number, updates: Partial<ScriptParameter>) {
		onParametersChange(
			parameters.map((param, i) => (i === index ? { ...param, ...updates } : param))
		);
	}
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between">
		<h3 class="text-lg font-semibold">Parameters</h3>
		<Button type="button" variant="outline" size="sm" onclick={addParameter}>
			<Plus class="h-4 w-4 mr-2" />
			Add Parameter
		</Button>
	</div>

	{#if parameters.length === 0}
		<div class="text-center py-8 border border-dashed rounded-lg">
			<p class="text-sm text-muted-foreground mb-4">No parameters defined</p>
			<Button type="button" variant="outline" size="sm" onclick={addParameter}>
				<Plus class="h-4 w-4 mr-2" />
				Add First Parameter
			</Button>
		</div>
	{:else}
		<div class="space-y-4">
			{#each parameters as param, index (param.name)}
				<ParameterEditor
					parameter={param}
					{index}
					onUpdate={updateParameter}
					onRemove={removeParameter}
				/>
			{/each}
		</div>
	{/if}
</div>

