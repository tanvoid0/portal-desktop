<!--
	Theme Customizer - Theme and appearance customization
-->

<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { Palette, Download, Upload, RotateCcw } from 'lucide-svelte';
	import { settingsService } from '../services/settingsService';
	import type { ThemeSettings, CustomTheme } from '../types';

	interface Props {
		settings: ThemeSettings | null;
		onUpdate: (updates: Partial<ThemeSettings>) => void;
	}

	let { settings, onUpdate }: Props = $props();

	if (!settings) {
		settings = {
			primaryColor: '#3b82f6',
			secondaryColor: '#64748b',
			accentColor: '#f59e0b',
			backgroundColor: '#ffffff',
			surfaceColor: '#f8fafc',
			textColor: '#1e293b',
			borderRadius: 8,
			shadowIntensity: 5,
			animationSpeed: 'normal',
			customThemes: [],
			activeTheme: 'default'
		};
	}

	function handleColorChange(field: string, value: string) {
		onUpdate({ [field]: value } as Partial<ThemeSettings>);
	}

	function handleResetColors() {
		onUpdate({
			primaryColor: '#3b82f6',
			secondaryColor: '#64748b',
			accentColor: '#f59e0b',
			backgroundColor: '#ffffff',
			surfaceColor: '#f8fafc',
			textColor: '#1e293b'
		});
	}

	function handleSaveCustomTheme() {
		if (!settings) return;
		
		const themeName = prompt('Enter theme name:');
		if (!themeName) return;

		const customTheme: CustomTheme = {
			id: `custom-${Date.now()}`,
			name: themeName,
			description: 'Custom theme',
			colors: {
				primary: settings.primaryColor,
				secondary: settings.secondaryColor,
				accent: settings.accentColor,
				background: settings.backgroundColor,
				surface: settings.surfaceColor,
				text: settings.textColor
			},
			createdAt: new Date(),
			updatedAt: new Date()
		};

		onUpdate({
			customThemes: [...settings.customThemes, customTheme]
		});
	}
</script>

<div class="space-y-6">
	<!-- Color Scheme -->
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle class="flex items-center gap-2">
						<Palette class="h-5 w-5" />
						Color Scheme
					</CardTitle>
					<CardDescription>Customize the color palette</CardDescription>
				</div>
				<Button variant="outline" size="sm" onclick={handleResetColors}>
					<RotateCcw class="h-4 w-4 mr-2" />
					Reset
				</Button>
			</div>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="primaryColor">Primary Color</Label>
					<div class="flex gap-2">
						<Input
							id="primaryColor"
							type="color"
							bind:value={settings.primaryColor}
							onchange={() => handleColorChange('primaryColor', settings.primaryColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.primaryColor}
							onchange={() => handleColorChange('primaryColor', settings.primaryColor)}
							placeholder="#3b82f6"
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="secondaryColor">Secondary Color</Label>
					<div class="flex gap-2">
						<Input
							id="secondaryColor"
							type="color"
							bind:value={settings.secondaryColor}
							onchange={() => handleColorChange('secondaryColor', settings.secondaryColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.secondaryColor}
							onchange={() => handleColorChange('secondaryColor', settings.secondaryColor)}
							placeholder="#64748b"
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="accentColor">Accent Color</Label>
					<div class="flex gap-2">
						<Input
							id="accentColor"
							type="color"
							bind:value={settings.accentColor}
							onchange={() => handleColorChange('accentColor', settings.accentColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.accentColor}
							onchange={() => handleColorChange('accentColor', settings.accentColor)}
							placeholder="#f59e0b"
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="textColor">Text Color</Label>
					<div class="flex gap-2">
						<Input
							id="textColor"
							type="color"
							bind:value={settings.textColor}
							onchange={() => handleColorChange('textColor', settings.textColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.textColor}
							onchange={() => handleColorChange('textColor', settings.textColor)}
							placeholder="#1e293b"
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="backgroundColor">Background Color</Label>
					<div class="flex gap-2">
						<Input
							id="backgroundColor"
							type="color"
							bind:value={settings.backgroundColor}
							onchange={() => handleColorChange('backgroundColor', settings.backgroundColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.backgroundColor}
							onchange={() => handleColorChange('backgroundColor', settings.backgroundColor)}
							placeholder="#ffffff"
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="surfaceColor">Surface Color</Label>
					<div class="flex gap-2">
						<Input
							id="surfaceColor"
							type="color"
							bind:value={settings.surfaceColor}
							onchange={() => handleColorChange('surfaceColor', settings.surfaceColor)}
							class="h-10 w-20 p-1"
						/>
						<Input
							type="text"
							bind:value={settings.surfaceColor}
							onchange={() => handleColorChange('surfaceColor', settings.surfaceColor)}
							placeholder="#f8fafc"
						/>
					</div>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- UI Elements -->
	<Card>
		<CardHeader>
			<CardTitle>UI Elements</CardTitle>
			<CardDescription>Configure UI appearance and behavior</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="borderRadius">Border Radius</Label>
					<Input
						id="borderRadius"
						type="number"
						min="0"
						max="24"
						bind:value={settings.borderRadius}
						onchange={() => onUpdate({ borderRadius: parseInt(String(settings.borderRadius)) || 8 })}
					/>
				</div>

				<div class="space-y-2">
					<Label for="shadowIntensity">Shadow Intensity</Label>
					<Input
						id="shadowIntensity"
						type="number"
						min="0"
						max="10"
						bind:value={settings.shadowIntensity}
						onchange={() => onUpdate({ shadowIntensity: parseInt(String(settings.shadowIntensity)) || 5 })}
					/>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="animationSpeed">Animation Speed</Label>
				<Select
					defaultValue={settings.animationSpeed}
					options={['slow', 'normal', 'fast']}
					onSelect={(value) => onUpdate({ animationSpeed: value as 'slow' | 'normal' | 'fast' })}
					placeholder="Select speed"
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Custom Themes -->
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle>Custom Themes</CardTitle>
					<CardDescription>Save and manage your custom themes</CardDescription>
				</div>
				<Button variant="outline" size="sm" onclick={handleSaveCustomTheme}>
					<Download class="h-4 w-4 mr-2" />
					Save Theme
				</Button>
			</div>
		</CardHeader>
		<CardContent>
			{#if settings.customThemes.length > 0}
				<div class="space-y-2">
					{#each settings.customThemes as theme (theme.id)}
						<div class="flex items-center justify-between p-3 border rounded-lg">
							<div>
								<p class="font-medium">{theme.name}</p>
								{#if theme.description}
									<p class="text-sm text-muted-foreground">{theme.description}</p>
								{/if}
								<p class="text-xs text-muted-foreground mt-1">
									Created: {new Date(theme.createdAt).toLocaleDateString()}
								</p>
							</div>
							<div class="flex gap-2">
								<Button
									variant="outline"
									size="sm"
									onclick={() => onUpdate({ activeTheme: theme.id })}
								>
									Apply
								</Button>
								<Button
									variant="ghost"
									size="sm"
									onclick={() => {
										onUpdate({
											customThemes: settings.customThemes.filter(t => t.id !== theme.id)
										});
									}}
								>
									Delete
								</Button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="text-center py-8 text-muted-foreground">
					<p>No custom themes yet.</p>
					<p class="text-sm mt-2">Adjust colors above and click "Save Theme" to create one.</p>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

