<!--
	Terminal Settings - Terminal configuration and preferences
-->

<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { settingsService } from '../services/settingsService';
	import type { TerminalSettings } from '../types';

	interface Props {
		settings: TerminalSettings | null;
		onUpdate: (updates: Partial<TerminalSettings>) => void;
	}

	let { settings, onUpdate }: Props = $props();

	if (!settings) {
		settings = {
			fontFamily: 'monaco',
			fontSize: 14,
			lineHeight: 1.2,
			cursorStyle: 'block',
			cursorBlink: true,
			scrollback: 1000,
			bellStyle: 'none',
			rightClickSelectsWord: false,
			selectionMode: 'normal',
			shellIntegration: true,
			commandHistory: true,
			commandSuggestions: true,
			theme: {
				name: 'default',
				background: '#1e1e1e',
				foreground: '#d4d4d4',
				cursor: '#ffffff',
				selection: '#264f78',
				colors: {
					black: '#000000',
					red: '#cd3131',
					green: '#0dbc79',
					yellow: '#e5e510',
					blue: '#2472c8',
					magenta: '#bc3fbc',
					cyan: '#11a8cd',
					white: '#e5e5e5',
					brightBlack: '#666666',
					brightRed: '#f14c4c',
					brightGreen: '#23d18b',
					brightYellow: '#f5f543',
					brightBlue: '#3b8eea',
					brightMagenta: '#d670d6',
					brightCyan: '#29b8db',
					brightWhite: '#e5e5e5'
				}
			},
			encoding: 'utf-8',
			locale: 'en_US.UTF-8'
		};
	}

	const availableFonts = settingsService.getAvailableFonts();
	const availableThemes = settingsService.getAvailableTerminalThemes();
</script>

<div class="space-y-6">
	<!-- Appearance -->
	<Card>
		<CardHeader>
			<CardTitle>Appearance</CardTitle>
			<CardDescription>Configure terminal visual appearance</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="terminalFontFamily">Font Family</Label>
				<Select
					defaultValue={settings.fontFamily}
					options={availableFonts.map(f => ({ value: f.id, label: `${f.name} (${f.category})` }))}
					onSelect={(value) => onUpdate({ fontFamily: value })}
					placeholder="Select font"
				/>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="terminalFontSize">Font Size</Label>
					<Input
						id="terminalFontSize"
						type="number"
						min="8"
						max="72"
						bind:value={settings.fontSize}
						onchange={() => onUpdate({ fontSize: parseInt(String(settings.fontSize)) || 14 })}
					/>
				</div>

				<div class="space-y-2">
					<Label for="terminalLineHeight">Line Height</Label>
					<Input
						id="terminalLineHeight"
						type="number"
						min="1"
						max="3"
						step="0.1"
						bind:value={settings.lineHeight}
						onchange={() => onUpdate({ lineHeight: parseFloat(String(settings.lineHeight)) || 1.2 })}
					/>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="terminalTheme">Terminal Theme</Label>
				<Select
					defaultValue={settings.theme.name}
					options={availableThemes.map(t => ({ value: t.id, label: t.name }))}
					onSelect={(value) => {
						const theme = availableThemes.find(t => t.id === value);
						if (theme) {
							onUpdate({
								theme: {
									name: theme.name,
									background: theme.colors.background,
									foreground: theme.colors.foreground,
									cursor: theme.colors.cursor,
									selection: theme.colors.selection || '#264f78',
									colors: settings.theme.colors
								}
							});
						}
					}}
					placeholder="Select theme"
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Cursor -->
	<Card>
		<CardHeader>
			<CardTitle>Cursor</CardTitle>
			<CardDescription>Configure terminal cursor appearance</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="cursorStyle">Cursor Style</Label>
				<Select
					defaultValue={settings.cursorStyle}
					options={[
						{ value: 'block', label: 'Block' },
						{ value: 'underline', label: 'Underline' },
						{ value: 'line', label: 'Line' }
					]}
					onSelect={(value) => onUpdate({ cursorStyle: value as 'block' | 'underline' | 'line' })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="cursorBlink">Cursor Blink</Label>
					<p class="text-sm text-muted-foreground">Animate cursor blinking</p>
				</div>
				<Switch
					id="cursorBlink"
					checked={settings.cursorBlink}
					onCheckedChange={(checked) => onUpdate({ cursorBlink: checked })}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Behavior -->
	<Card>
		<CardHeader>
			<CardTitle>Behavior</CardTitle>
			<CardDescription>Configure terminal behavior and interactions</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="scrollback">Scrollback Lines</Label>
				<Input
					id="scrollback"
					type="number"
					min="100"
					max="10000"
					bind:value={settings.scrollback}
					onchange={() => onUpdate({ scrollback: parseInt(String(settings.scrollback)) || 1000 })}
				/>
			</div>

			<div class="space-y-2">
				<Label for="bellStyle">Bell Style</Label>
				<Select
					defaultValue={settings.bellStyle}
					options={[
						{ value: 'none', label: 'None' },
						{ value: 'visual', label: 'Visual' },
						{ value: 'sound', label: 'Sound' }
					]}
					onSelect={(value) => onUpdate({ bellStyle: value as 'none' | 'visual' | 'sound' })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="rightClickSelectsWord">Right Click Selects Word</Label>
					<p class="text-sm text-muted-foreground">Right click to select word</p>
				</div>
				<Switch
					id="rightClickSelectsWord"
					checked={settings.rightClickSelectsWord}
					onCheckedChange={(checked) => onUpdate({ rightClickSelectsWord: checked })}
				/>
			</div>

			<div class="space-y-2">
				<Label for="selectionMode">Selection Mode</Label>
				<Select
					defaultValue={settings.selectionMode}
					options={['normal', 'column']}
					onSelect={(value) => onUpdate({ selectionMode: value as 'normal' | 'column' })}
					placeholder="Select mode"
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Shell Integration -->
	<Card>
		<CardHeader>
			<CardTitle>Shell Integration</CardTitle>
			<CardDescription>Configure shell integration features</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="shellIntegration">Shell Integration</Label>
					<p class="text-sm text-muted-foreground">Enable shell integration features</p>
				</div>
				<Switch
					id="shellIntegration"
					checked={settings.shellIntegration}
					onCheckedChange={(checked) => onUpdate({ shellIntegration: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="commandHistory">Command History</Label>
					<p class="text-sm text-muted-foreground">Save command history</p>
				</div>
				<Switch
					id="commandHistory"
					checked={settings.commandHistory}
					onCheckedChange={(checked) => onUpdate({ commandHistory: checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="commandSuggestions">Command Suggestions</Label>
					<p class="text-sm text-muted-foreground">Show command suggestions</p>
				</div>
				<Switch
					id="commandSuggestions"
					checked={settings.commandSuggestions}
					onCheckedChange={(checked) => onUpdate({ commandSuggestions: checked })}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Advanced -->
	<Card>
		<CardHeader>
			<CardTitle>Advanced</CardTitle>
			<CardDescription>Advanced terminal configuration</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="encoding">Encoding</Label>
					<Input
						id="encoding"
						bind:value={settings.encoding}
						onchange={() => onUpdate({ encoding: settings.encoding })}
					/>
				</div>

				<div class="space-y-2">
					<Label for="locale">Locale</Label>
					<Input
						id="locale"
						bind:value={settings.locale}
						onchange={() => onUpdate({ locale: settings.locale })}
					/>
				</div>
			</div>
		</CardContent>
	</Card>
</div>

