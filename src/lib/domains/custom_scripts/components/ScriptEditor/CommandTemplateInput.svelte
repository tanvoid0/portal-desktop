<!--
	Command Template Input Component
	Handles the command template input with parameter reference hints
-->

<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Info } from '@lucide/svelte';

	interface Props {
		command: string;
		onChange: (value: string) => void;
	}

	let { command, onChange }: Props = $props();
</script>

<div class="space-y-2">
	<Label for="command-template">Command Template *</Label>
	<Input
		id="command-template"
		value={command}
		oninput={(e) => onChange((e.target as HTMLInputElement).value)}
		placeholder="e.g., openvpn --config $VPN_CONFIG --auth-user-pass $VPN_AUTH"
		required
	/>
	<div class="flex items-start gap-2 p-3 border border-blue-200 bg-blue-50 dark:bg-blue-950 dark:border-blue-800 rounded-md">
		<Info class="h-4 w-4 mt-0.5 text-blue-600 dark:text-blue-400 flex-shrink-0" />
		<p class="text-sm text-blue-800 dark:text-blue-200">
			Use <code>$param_name</code> or <code>$&#123;param_name&#125;</code> to reference parameters in your command.
		</p>
	</div>
</div>

