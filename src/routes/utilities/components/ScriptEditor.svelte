<!--
	Script Editor Component
	Allows creating and editing custom scripts with dynamic parameters
-->

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Switch } from '$lib/components/ui/switch';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Plus, X, Trash2 } from '@lucide/svelte';
	import { CustomScriptService, type CustomScript, type ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';
	import FilePicker from '$lib/components/ui/file-picker.svelte';

	interface Props {
		script?: CustomScript | null;
		onClose: () => void;
		onSaved: () => void;
	}

	let { script, onClose, onSaved }: Props = $props();

	// Form state
	let name = $state(script?.name || '');
	let description = $state(script?.description || '');
	let command = $state(script?.command || '');
	let category = $state(script?.category || '');
	let icon = $state(script?.icon || '');
	let requiresSudo = $state(script?.requires_sudo || false);
	let isInteractive = $state(script?.is_interactive || false);
	let parameters = $state<ScriptParameter[]>(
		script ? CustomScriptService.parseParameters(script.parameters_json) : []
	);

	let saving = $state(false);
	let error = $state<string | null>(null);

	function addParameter() {
		parameters = [
			...parameters,
			{
				name: `param${parameters.length + 1}`,
				label: `Parameter ${parameters.length + 1}`,
				parameter_type: 'string',
				required: false,
			},
		];
	}

	function removeParameter(index: number) {
		parameters = parameters.filter((_param: ScriptParameter, i: number) => i !== index);
	}

	function updateParameter(index: number, updates: Partial<ScriptParameter>) {
		parameters = parameters.map((param: ScriptParameter, i: number) =>
			i === index ? { ...param, ...updates } : param
		);
	}

	async function handleSave() {
		if (!name.trim() || !command.trim()) {
			error = 'Name and command are required';
			return;
		}

		saving = true;
		error = null;

		try {
			const parametersJson = JSON.stringify(parameters);
			if (script) {
				await CustomScriptService.updateScript(script.id, {
					name,
					description: description || undefined,
					command,
					parameters: parameters,
					category: category || undefined,
					icon: icon || undefined,
					requires_sudo: requiresSudo,
					is_interactive: isInteractive,
				});
			} else {
				await CustomScriptService.createScript({
					name,
					description: description || undefined,
					command,
					parameters,
					category: category || undefined,
					icon: icon || undefined,
					requires_sudo: requiresSudo,
					is_interactive: isInteractive,
				});
			}
			onSaved();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save script';
			console.error('Failed to save script:', err);
		} finally {
			saving = false;
		}
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
	<div class="bg-background rounded-lg shadow-lg w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
		<div class="p-6 border-b">
			<h2 class="text-2xl font-semibold">
				{script ? 'Edit Script' : 'Create Script'}
			</h2>
		</div>

		<div class="flex-1 overflow-y-auto p-6 space-y-6">
			{#if error}
				<div class="p-4 border border-red-200 bg-red-50 rounded-md">
					<p class="text-sm text-red-600">{error}</p>
				</div>
			{/if}

			<div class="space-y-4">
				<div>
					<Label for="name">Name *</Label>
					<Input id="name" bind:value={name} placeholder="e.g., OpenVPN Connection" />
				</div>

				<div>
					<Label for="description">Description</Label>
					<Textarea
						id="description"
						bind:value={description}
						placeholder="Brief description of what this script does"
						rows={2}
					/>
				</div>

				<div>
					<Label for="command">Command Template *</Label>
					<Textarea
						id="command"
						bind:value={command}
						placeholder={'e.g., openvpn --config "${' + 'CONFIG_FILE' + '}" --auth-retry interact --auth-user-pass "${' + 'AUTH_FILE' + '}"'}
						rows={4}
						class="font-mono text-sm"
					/>
					<p class="text-sm text-muted-foreground mt-1">
						Use <code>$param_name</code> or <code>$&#123;param_name&#125;</code> to reference parameters.
						{#if requiresSudo}
							<br />
							<span class="text-amber-600 dark:text-amber-400">
								Note: Don't include "sudo" in the command - it will be added automatically when "Requires Sudo" is enabled.
							</span>
						{/if}
					</p>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div>
						<Label for="category">Category</Label>
						<Input id="category" bind:value={category} placeholder="e.g., VPN, Network" />
					</div>

					<div>
							<Label for="icon">Icon (emoji)</Label>
						<Input id="icon" bind:value={icon} placeholder="🔒" maxlength={2} />
					</div>
				</div>

				<div class="flex items-center gap-6">
					<div class="flex items-center gap-2">
						<Switch id="sudo" bind:checked={requiresSudo} />
						<Label for="sudo">Requires Sudo</Label>
					</div>

					<div class="flex items-center gap-2">
						<Switch id="interactive" bind:checked={isInteractive} />
						<Label for="interactive">Interactive (requires input)</Label>
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<Label>Parameters</Label>
						<Button size="sm" variant="outline" onclick={addParameter}>
							<Plus class="h-4 w-4 mr-2" />
							Add Parameter
						</Button>
					</div>

					{#each parameters as param, index (index)}
						<Card>
							<CardHeader>
								<div class="flex items-center justify-between">
									<CardTitle class="text-sm">Parameter {index + 1}</CardTitle>
									<Button
										size="sm"
										variant="ghost"
										onclick={() => removeParameter(index)}
									>
										<X class="h-4 w-4" />
									</Button>
								</div>
							</CardHeader>
							<CardContent class="space-y-4">
								<div class="grid grid-cols-2 gap-4">
									<div>
										<Label>Name (variable)</Label>
									<Input
										value={(param as ScriptParameter).name}
										oninput={(e: Event) =>
											updateParameter(index, {
												name: (e.target as HTMLInputElement).value,
											})
										}
										placeholder="VPN_DIR"
										class="font-mono"
									/>
									</div>
									<div>
										<Label>Label</Label>
									<Input
										value={(param as ScriptParameter).label}
										oninput={(e: Event) =>
											updateParameter(index, {
												label: (e.target as HTMLInputElement).value,
											})
										}
										placeholder="VPN Directory"
									/>
									</div>
								</div>

								<div>
									<Label>Type</Label>
									<select
										value={(param as ScriptParameter).parameter_type}
										onchange={(e: Event) =>
											updateParameter(index, {
												parameter_type: (e.target as HTMLSelectElement)
													.value as ScriptParameter['parameter_type'],
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

								{#if param.parameter_type === 'file'}
									<div>
										<Label>File Filters (comma-separated)</Label>
									<Input
										value={(param as ScriptParameter).file_filters?.join(', ') || ''}
										oninput={(e: Event) =>
											updateParameter(index, {
												file_filters: (e.target as HTMLInputElement).value
													.split(',')
													.map((f: string) => f.trim())
													.filter(Boolean),
											})
										}
										placeholder="*.ovpn, *.txt"
									/>
									</div>
								{/if}

								<div>
									<Label>Description</Label>
									<Input
										value={(param as ScriptParameter).description || ''}
										oninput={(e: Event) =>
											updateParameter(index, {
												description: (e.target as HTMLInputElement).value || undefined,
											})
										}
										placeholder="Optional description"
									/>
								</div>

								<div>
									<Label>Default Value</Label>
									{#if (param as ScriptParameter).parameter_type === 'file'}
										{@const fileFilters = (param as ScriptParameter).file_filters || []}
										<FilePicker
											value={(param as ScriptParameter).default_value || ''}
											label=""
											description=""
											filters={fileFilters.length > 0 ? [{ name: 'Files', extensions: fileFilters }] : []}
											selectFolder={false}
											onChange={(path) => {
												updateParameter(index, {
													default_value: path || undefined,
												});
											}}
										/>
									{:else if (param as ScriptParameter).parameter_type === 'folder'}
										<FilePicker
											value={(param as ScriptParameter).default_value || ''}
											label=""
											description=""
											selectFolder={true}
											onChange={(path) => {
												updateParameter(index, {
													default_value: path || undefined,
												});
											}}
										/>
									{:else}
										<Input
											value={(param as ScriptParameter).default_value || ''}
											oninput={(e: Event) =>
												updateParameter(index, {
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
										checked={(param as ScriptParameter).required}
										onchange={(e: Event) =>
											updateParameter(index, {
												required: (e.target as HTMLInputElement).checked,
											})
										}
										class="rounded border-gray-300"
									/>
									<Label>Required</Label>
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			</div>
		</div>

		<div class="p-6 border-t flex justify-end gap-2">
			<Button variant="outline" onclick={onClose} disabled={saving}>
				Cancel
			</Button>
			<Button onclick={handleSave} disabled={saving}>
				{saving ? 'Saving...' : 'Save'}
			</Button>
		</div>
	</div>
</div>

