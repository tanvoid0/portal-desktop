<!--
	Editor Settings - Code editor preferences and configuration
-->

<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { settingsService } from '../services/settingsService';
	import type { EditorSettings } from '../types';

	interface Props {
		settings: EditorSettings | null;
		onUpdate: (updates: Partial<EditorSettings>) => void;
	}

	let { settings, onUpdate }: Props = $props();

	if (!settings) {
		settings = {
			fontFamily: 'monaco',
			fontSize: 14,
			lineHeight: 1.5,
			tabSize: 2,
			insertSpaces: true,
			wordWrap: true,
			showLineNumbers: true,
			showMinimap: false,
			showWhitespace: false,
			syntaxHighlighting: true,
			bracketMatching: true,
			autoIndent: true,
			autoComplete: true,
			suggestions: true,
			parameterHints: true,
			editorTheme: 'default',
			terminalTheme: 'default',
			keybindings: {}
		};
	}

	const availableFonts = settingsService.getAvailableFonts();
</script>

<div class="space-y-6">
	<!-- Font & Display -->
	<Card>
		<CardHeader>
			<CardTitle>Font & Display</CardTitle>
			<CardDescription>Configure editor font and visual appearance</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="fontFamily">Font Family</Label>
				<Select
					defaultValue={settings.fontFamily}
					options={availableFonts.map(f => ({ value: f.id, label: `${f.name} (${f.category})` }))}
					onSelect={(value) => onUpdate({ fontFamily: value })}
					placeholder="Select font"
				/>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="fontSize">Font Size</Label>
					<Input
						id="fontSize"
						type="number"
						min="8"
						max="72"
						bind:value={settings.fontSize}
						onchange={() => onUpdate({ fontSize: parseInt(String(settings.fontSize)) || 14 })}
					/>
				</div>

				<div class="space-y-2">
					<Label for="lineHeight">Line Height</Label>
					<Input
						id="lineHeight"
						type="number"
						min="1"
						max="3"
						step="0.1"
						bind:value={settings.lineHeight}
						onchange={() => onUpdate({ lineHeight: parseFloat(String(settings.lineHeight)) || 1.5 })}
					/>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Indentation -->
	<Card>
		<CardHeader>
			<CardTitle>Indentation</CardTitle>
			<CardDescription>Configure how code is indented</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="tabSize">Tab Size</Label>
					<Input
						id="tabSize"
						type="number"
						min="1"
						max="8"
						bind:value={settings.tabSize}
						onchange={() => onUpdate({ tabSize: parseInt(String(settings.tabSize)) || 2 })}
					/>
				</div>

				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<Label for="insertSpaces">Insert Spaces</Label>
						<p class="text-sm text-muted-foreground">Use spaces instead of tabs</p>
					</div>
					{#if settings}
						<Switch
							id="insertSpaces"
							checked={settings.insertSpaces}
							onclick={() => onUpdate({ insertSpaces: !settings.insertSpaces })}
						/>
					{/if}
				</div>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="autoIndent">Auto Indent</Label>
					<p class="text-sm text-muted-foreground">Automatically indent code</p>
				</div>
				<Switch
					id="autoIndent"
					checked={settings.autoIndent}
					onCheckedChange={(checked) => onUpdate({ autoIndent: checked })}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Visual Features -->
	<Card>
		<CardHeader>
			<CardTitle>Visual Features</CardTitle>
			<CardDescription>Configure editor visual elements</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="wordWrap">Word Wrap</Label>
					<p class="text-sm text-muted-foreground">Wrap long lines</p>
				</div>
				<Switch
					id="wordWrap"
					checked={settings.wordWrap}
					onCheckedChange={(checked) => onUpdate({ wordWrap: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="showLineNumbers">Show Line Numbers</Label>
					<p class="text-sm text-muted-foreground">Display line numbers</p>
				</div>
				<Switch
					id="showLineNumbers"
					checked={settings.showLineNumbers}
					onCheckedChange={(checked) => onUpdate({ showLineNumbers: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="showMinimap">Show Minimap</Label>
					<p class="text-sm text-muted-foreground">Display code minimap</p>
				</div>
				<Switch
					id="showMinimap"
					checked={settings.showMinimap}
					onCheckedChange={(checked) => onUpdate({ showMinimap: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="showWhitespace">Show Whitespace</Label>
					<p class="text-sm text-muted-foreground">Display whitespace characters</p>
				</div>
				<Switch
					id="showWhitespace"
					checked={settings.showWhitespace}
					onCheckedChange={(checked) => onUpdate({ showWhitespace: checked })}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Code Features -->
	<Card>
		<CardHeader>
			<CardTitle>Code Features</CardTitle>
			<CardDescription>Configure code editing features</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="syntaxHighlighting">Syntax Highlighting</Label>
					<p class="text-sm text-muted-foreground">Color code syntax</p>
				</div>
				<Switch
					id="syntaxHighlighting"
					checked={settings.syntaxHighlighting}
					onCheckedChange={(checked) => onUpdate({ syntaxHighlighting: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="bracketMatching">Bracket Matching</Label>
					<p class="text-sm text-muted-foreground">Highlight matching brackets</p>
				</div>
				<Switch
					id="bracketMatching"
					checked={settings.bracketMatching}
					onCheckedChange={(checked) => onUpdate({ bracketMatching: checked })}
				/>
			</div>

			<Separator />

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="autoComplete">Auto Complete</Label>
					<p class="text-sm text-muted-foreground">Enable auto-completion</p>
				</div>
				<Switch
					id="autoComplete"
					checked={settings.autoComplete}
					onCheckedChange={(checked) => onUpdate({ autoComplete: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="suggestions">Suggestions</Label>
					<p class="text-sm text-muted-foreground">Show code suggestions</p>
				</div>
				<Switch
					id="suggestions"
					checked={settings.suggestions}
					onCheckedChange={(checked) => onUpdate({ suggestions: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="parameterHints">Parameter Hints</Label>
					<p class="text-sm text-muted-foreground">Show function parameter hints</p>
				</div>
				<Switch
					id="parameterHints"
					checked={settings.parameterHints}
					onCheckedChange={(checked) => onUpdate({ parameterHints: checked })}
				/>
			</div>
		</CardContent>
	</Card>
</div>

