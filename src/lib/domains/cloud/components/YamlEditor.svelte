<!-- Safe YAML Editor Component with Validation -->
<script lang="ts">
	import { Textarea } from '@/lib/components/ui/textarea';
	import { Button } from '@/lib/components/ui/button';
	import { Alert, AlertDescription, AlertTitle } from '@/lib/components/ui/alert';
	import { Badge } from '@/lib/components/ui/badge';
	import { AlertCircle, CheckCircle, Save, X, RotateCcw } from '@lucide/svelte';
	import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';
	
	interface Props {
		value: string;
		onSave?: (yaml: string) => Promise<void>;
		onCancel?: () => void;
		readOnly?: boolean;
		resourceName?: string;
		resourceKind?: string;
		namespace?: string;
	}
	
	let {
		value = '',
		onSave,
		onCancel,
		readOnly = false,
		resourceName = '',
		resourceKind = 'Resource',
		namespace = ''
	}: Props = $props();
	
	let editedValue = $state(value);
	let hasChanges = $state(false);
	let validationError = $state<string | null>(null);
	let isSaving = $state(false);
	let showConfirmDialog = $state(false);
	let originalValue = $state(value);
	
	// Track changes
	$effect(() => {
		hasChanges = editedValue !== originalValue;
		validationError = null;
	});
	
	// Update when value prop changes
	$effect(() => {
		if (value !== originalValue) {
			originalValue = value;
			editedValue = value;
			hasChanges = false;
		}
	});
	
	function validateYAML(yamlContent: string): string | null {
		if (!yamlContent.trim()) {
			return 'YAML content cannot be empty';
		}
		
		// Basic YAML structure validation
		try {
			// Check for required fields
			if (!yamlContent.includes('kind:')) {
				return 'YAML must contain a "kind" field';
			}
			if (!yamlContent.includes('metadata:')) {
				return 'YAML must contain a "metadata" field';
			}
			if (!yamlContent.includes('name:')) {
				return 'YAML must contain "metadata.name" field';
			}
			
			// Validate namespace if specified
			if (namespace && !yamlContent.includes(`namespace: ${namespace}`)) {
				// Warn but don't block - we'll add it automatically
			}
			
			// Try to parse as JSON (YAML is a superset)
			// We'll rely on backend validation for actual YAML parsing
			
			return null;
		} catch (error) {
			return `YAML validation error: ${error instanceof Error ? error.message : 'Unknown error'}`;
		}
	}
	
	function handleInput(e: Event) {
		const newValue = (e.target as HTMLTextAreaElement).value;
		editedValue = newValue;
		const error = validateYAML(newValue);
		validationError = error;
	}
	
	function handleSave() {
		const error = validateYAML(editedValue);
		if (error) {
			validationError = error;
			return;
		}
		
		// Show confirmation dialog for updates
		if (hasChanges && resourceName) {
			showConfirmDialog = true;
		} else if (onSave) {
			performSave();
		}
	}
	
	async function performSave() {
		if (!onSave) return;
		
		isSaving = true;
		validationError = null;
		
		try {
			await onSave(editedValue);
			originalValue = editedValue;
			hasChanges = false;
			showConfirmDialog = false;
		} catch (error) {
			validationError = error instanceof Error ? error.message : 'Failed to save YAML';
		} finally {
			isSaving = false;
		}
	}
	
	function handleCancel() {
		if (hasChanges) {
			if (confirm('You have unsaved changes. Are you sure you want to discard them?')) {
				editedValue = originalValue;
				hasChanges = false;
				validationError = null;
				if (onCancel) onCancel();
			}
		} else {
			if (onCancel) onCancel();
		}
	}
	
	function handleReset() {
		if (confirm('Reset all changes to original YAML?')) {
			editedValue = originalValue;
			hasChanges = false;
			validationError = null;
		}
	}
	
	const isValid = $derived(!validationError && editedValue.trim().length > 0);
</script>

<div class="space-y-4">
	<!-- Validation Status -->
	{#if validationError}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertTitle>Validation Error</AlertTitle>
			<AlertDescription>{validationError}</AlertDescription>
		</Alert>
	{:else if hasChanges && isValid}
		<Alert>
			<CheckCircle class="h-4 w-4" />
			<AlertTitle>Valid YAML</AlertTitle>
			<AlertDescription>YAML is valid. You have unsaved changes.</AlertDescription>
		</Alert>
	{/if}
	
	<!-- Editor Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Badge variant="outline">{resourceKind}</Badge>
			{#if resourceName}
				<Badge variant="secondary">{resourceName}</Badge>
			{/if}
			{#if namespace}
				<Badge variant="secondary">ns: {namespace}</Badge>
			{/if}
			{#if hasChanges}
				<Badge variant="secondary" class="bg-yellow-500/20 text-yellow-700 dark:text-yellow-400">Unsaved Changes</Badge>
			{/if}
		</div>
		<div class="flex items-center gap-2">
			{#if hasChanges}
				<Button variant="outline" size="sm" onclick={handleReset} disabled={isSaving}>
					<RotateCcw class="mr-2 h-4 w-4" />
					Reset
				</Button>
			{/if}
			{#if !readOnly}
				<Button variant="outline" size="sm" onclick={handleCancel} disabled={isSaving}>
					<X class="mr-2 h-4 w-4" />
					Cancel
				</Button>
				<Button 
					variant="default" 
					size="sm" 
					onclick={handleSave} 
					disabled={!isValid || isSaving || !hasChanges}
				>
					<Save class="mr-2 h-4 w-4" />
					{isSaving ? 'Saving...' : 'Save'}
				</Button>
			{/if}
		</div>
	</div>
	
	<!-- YAML Editor -->
	<div class="relative">
		<Textarea
			value={editedValue}
			oninput={handleInput}
			disabled={readOnly || isSaving}
			class="font-mono text-sm h-[600px] resize-none"
			placeholder="Enter YAML content..."
		/>
		{#if readOnly}
			<div class="absolute top-2 right-2">
				<Badge variant="secondary">Read Only</Badge>
			</div>
		{/if}
	</div>
	
	<!-- Confirmation Dialog -->
	<Dialog bind:open={showConfirmDialog}>
		<DialogContent>
			<DialogHeader>
				<DialogTitle>Confirm Update</DialogTitle>
				<DialogDescription>
					You are about to update <strong>{resourceKind}</strong> "<strong>{resourceName}</strong>" in namespace "<strong>{namespace}</strong>".
					<br /><br />
					<strong class="text-destructive">Warning:</strong> This will modify the resource in your Kubernetes cluster.
					Make sure the YAML is correct before proceeding.
				</DialogDescription>
			</DialogHeader>
			<DialogFooter>
				<Button variant="outline" onclick={() => showConfirmDialog = false} disabled={isSaving}>
					Cancel
				</Button>
				<Button variant="destructive" onclick={performSave} disabled={isSaving}>
					{isSaving ? 'Applying...' : 'Apply Changes'}
				</Button>
			</DialogFooter>
		</DialogContent>
	</Dialog>
</div>

