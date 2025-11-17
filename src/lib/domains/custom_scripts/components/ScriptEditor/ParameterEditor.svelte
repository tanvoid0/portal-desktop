<!--
	Parameter Editor Component
	Handles editing a single script parameter
-->

<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Button } from '$lib/components/ui/button';
	import { X } from '@lucide/svelte';
	import { FilePicker } from '$lib/domains/shared/components';
	import type { ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';

	interface Props {
		parameter: ScriptParameter;
		index: number;
		onUpdate: (index: number, updates: Partial<ScriptParameter>) => void;
		onRemove: (index: number) => void;
	}

	let { parameter, index, onUpdate, onRemove }: Props = $props();
</script>

<div class="border rounded-lg p-4 space-y-4">
	<div class="flex items-center justify-between">
		<h4 class="font-medium">Parameter {index + 1}</h4>
		<Button
			type="button"
			variant="ghost"
			size="sm"
			onclick={() => onRemove(index)}
		>
			<X class="h-4 w-4" />
		</Button>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<Label>Name</Label>
			<Input
				value={parameter.name}
				oninput={(e) => onUpdate(index, { name: (e.target as HTMLInputElement).value })}
				placeholder="VPN_CONFIG"
			/>
		</div>

		<div>
			<Label>Label</Label>
			<Input
				value={parameter.label}
				oninput={(e) => onUpdate(index, { label: (e.target as HTMLInputElement).value })}
				placeholder="VPN Directory"
			/>
		</div>
	</div>

	<div>
		<Label>Type</Label>
		<select
			value={parameter.parameter_type}
			onchange={(e) =>
				onUpdate(index, {
					parameter_type: (e.target as HTMLSelectElement).value as ScriptParameter['parameter_type'],
				})
			}
			class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
		>
			<option value="string">String</option>
			<option value="file">File</option>
			<option value="folder">Folder</option>
			<option value="number">Number</option>
			<option value="boolean">Boolean</option>
			<option value="password">Password</option>
		</select>
	</div>

	{#if parameter.parameter_type === 'file'}
		<div>
			<Label>File Filters (comma-separated)</Label>
			<Input
				value={parameter.file_filters?.join(', ') || ''}
				oninput={(e) => {
					const filters = (e.target as HTMLInputElement).value
						.split(',')
						.map((f) => f.trim())
						.filter(Boolean);
					onUpdate(index, { file_filters: filters.length > 0 ? filters : undefined });
				}}
				placeholder="e.g., *.ovpn, *.conf"
			/>
		</div>
	{/if}

	<div>
		<Label>Description</Label>
		<Input
			value={parameter.description || ''}
			oninput={(e) =>
				onUpdate(index, {
					description: (e.target as HTMLInputElement).value || undefined,
				})
			}
			placeholder="Optional description"
		/>
	</div>

	<div>
		<Label>Default Value</Label>
		{#if parameter.parameter_type === 'file'}
			{@const fileFilters = parameter.file_filters || []}
			<FilePicker
				value={parameter.default_value || ''}
				label=""
				description=""
				filters={fileFilters.length > 0 ? [{ name: 'Files', extensions: fileFilters }] : []}
				selectFolder={false}
				onChange={(path) => {
					onUpdate(index, {
						default_value: path || undefined,
					});
				}}
			/>
		{:else if parameter.parameter_type === 'folder'}
			<FilePicker
				value={parameter.default_value || ''}
				label=""
				description=""
				selectFolder={true}
				onChange={(path) => {
					onUpdate(index, {
						default_value: path || undefined,
					});
				}}
			/>
		{:else}
			<Input
				value={parameter.default_value || ''}
				oninput={(e) =>
					onUpdate(index, {
						default_value: (e.target as HTMLInputElement).value || undefined,
					})
				}
				placeholder="Optional default value"
			/>
		{/if}
	</div>

	<div class="flex items-center gap-2">
		<input
			type="checkbox"
			checked={parameter.required}
			onchange={(e) =>
				onUpdate(index, {
					required: (e.target as HTMLInputElement).checked,
				})
			}
			class="rounded border-gray-300"
		/>
		<Label>Required</Label>
	</div>
</div>

