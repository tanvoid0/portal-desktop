<!--
	Script Metadata Form Component
	Handles name, description, category, and icon inputs
-->

<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';

	interface Props {
		name: string;
		description: string;
		category: string;
		icon: string;
		onNameChange: (value: string) => void;
		onDescriptionChange: (value: string) => void;
		onCategoryChange: (value: string) => void;
		onIconChange: (value: string) => void;
	}

	let {
		name,
		description,
		category,
		icon,
		onNameChange,
		onDescriptionChange,
		onCategoryChange,
		onIconChange
	}: Props = $props();
</script>

<div class="space-y-4">
	<div>
		<Label for="script-name">Name *</Label>
		<Input
			id="script-name"
			value={name}
			oninput={(e) => onNameChange((e.target as HTMLInputElement).value)}
			placeholder="e.g., VPN Connection"
			required
		/>
	</div>

	<div>
		<Label for="script-description">Description</Label>
		<Textarea
			id="script-description"
			value={description}
			oninput={(e) => onDescriptionChange((e.target as HTMLTextAreaElement).value)}
			placeholder="Optional description of what this script does"
			rows={3}
		/>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<Label for="script-category">Category</Label>
			<Input
				id="script-category"
				value={category}
				oninput={(e) => onCategoryChange((e.target as HTMLInputElement).value)}
				placeholder="e.g., Network, Security"
			/>
		</div>

		<div>
			<Label for="script-icon">Icon</Label>
			<Input
				id="script-icon"
				value={icon}
				oninput={(e) => onIconChange((e.target as HTMLInputElement).value)}
				placeholder="e.g., 🔒"
				maxlength={2}
			/>
		</div>
	</div>
</div>

