<!--
	Script Editor Component
	Composes atomic components to create/edit custom scripts
-->

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { X } from '@lucide/svelte';
	import { CustomScriptService, type CustomScript, type ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';
	import ScriptMetadataForm from './ScriptMetadataForm.svelte';
	import CommandTemplateInput from './CommandTemplateInput.svelte';
	import ScriptOptions from './ScriptOptions.svelte';
	import ParameterList from './ParameterList.svelte';

	interface Props {
		script?: CustomScript | null;
		onClose: () => void;
		onSaved: () => void;
	}

	let { script, onClose, onSaved }: Props = $props();

	// Form state - initialized from script prop
	let name = $state('');
	let description = $state('');
	let command = $state('');
	let category = $state('');
	let icon = $state('');
	let requiresSudo = $state(false);
	let isInteractive = $state(false);
	let parameters = $state<ScriptParameter[]>([]);

	let saving = $state(false);
	let error = $state<string | null>(null);

	// Initialize form state from script prop and react to changes
	$effect(() => {
		if (script) {
			// Edit mode: populate from existing script
			name = script.name || '';
			description = script.description || '';
			command = script.command || '';
			category = script.category || '';
			icon = script.icon || '';
			requiresSudo = script.requires_sudo || false;
			isInteractive = script.is_interactive || false;
			parameters = CustomScriptService.parseParameters(script.parameters_json);
		} else {
			// Create mode: reset to defaults
			name = '';
			description = '';
			command = '';
			category = '';
			icon = '';
			requiresSudo = false;
			isInteractive = false;
			parameters = [];
		}
		error = null; // Clear any previous errors
	});

	async function handleSave() {
		if (!name.trim() || !command.trim()) {
			error = 'Name and command are required';
			return;
		}

		saving = true;
		error = null;

		try {
			if (script) {
				await CustomScriptService.updateScript(script.id, {
					name,
					description: description || undefined,
					command,
					parameters,
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
		<div class="p-6 border-b flex items-center justify-between">
			<h2 class="text-2xl font-semibold">
				{script ? 'Edit Script' : 'Create Script'}
			</h2>
			<Button variant="ghost" size="sm" onclick={onClose}>
				<X class="h-4 w-4" />
			</Button>
		</div>

		<div class="flex-1 overflow-y-auto p-6 space-y-6">
			{#if error}
				<div class="p-4 border border-red-200 bg-red-50 rounded-md">
					<p class="text-sm text-red-600">{error}</p>
				</div>
			{/if}

			<Card>
				<CardHeader>
					<CardTitle>Script Information</CardTitle>
					<CardDescription>Basic information about your script</CardDescription>
				</CardHeader>
				<CardContent>
					<ScriptMetadataForm
						{name}
						{description}
						{category}
						{icon}
						onNameChange={(value) => (name = value)}
						onDescriptionChange={(value) => (description = value)}
						onCategoryChange={(value) => (category = value)}
						onIconChange={(value) => (icon = value)}
					/>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Command Template</CardTitle>
					<CardDescription>Define the command with parameter placeholders</CardDescription>
				</CardHeader>
				<CardContent>
					<CommandTemplateInput {command} onChange={(value) => (command = value)} />
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Options</CardTitle>
					<CardDescription>Configure script execution options</CardDescription>
				</CardHeader>
				<CardContent>
					<ScriptOptions
						{requiresSudo}
						{isInteractive}
						onRequiresSudoChange={(value) => (requiresSudo = value)}
						onIsInteractiveChange={(value) => (isInteractive = value)}
					/>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Parameters</CardTitle>
					<CardDescription>Define parameters that can be configured when running the script</CardDescription>
				</CardHeader>
				<CardContent>
					<ParameterList {parameters} onParametersChange={(params) => (parameters = params)} />
				</CardContent>
			</Card>
		</div>

		<div class="p-6 border-t flex justify-end gap-2">
			<Button variant="outline" onclick={onClose} disabled={saving}>
				Cancel
			</Button>
			<Button onclick={handleSave} disabled={saving}>
				{saving ? 'Saving...' : script ? 'Update Script' : 'Create Script'}
			</Button>
		</div>
	</div>
</div>

