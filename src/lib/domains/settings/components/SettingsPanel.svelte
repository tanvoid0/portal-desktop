<!--
	Settings Panel - Main settings interface
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { settingsActions, settings, isLoadingSettings, settingsError } from '../stores/settingsStore';
	import { settingsService } from '../services/settingsService';
	import { logger } from '$lib/domains/shared';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import GeneralSettings from './GeneralSettings.svelte';
	import EditorSettings from './EditorSettings.svelte';
	import TerminalSettings from './TerminalSettings.svelte';
	import ThemeCustomizer from './ThemeCustomizer.svelte';
	import IdeSettings from './IdeSettings.svelte';
	import FrameworkIdeSettings from './FrameworkIdeSettings.svelte';
	import SettingsNavigation from './SettingsNavigation.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Settings, Download, RotateCcw, AlertCircle, Loader2, Save, RefreshCw } from 'lucide-svelte';
	import type { SettingsCategory } from '../types';
	
	type SettingsSection = 'general' | 'editor' | 'terminal' | 'theme' | 'ides' | 'framework-ides';

	// Derive active section from URL
	const activeSection = $derived.by((): SettingsSection => {
		const path = $page.url.pathname;
		if (path === '/settings' || path === '/settings/') return 'general';
		const section = path.replace('/settings/', '').replace(/\/$/, '');
		if (['general', 'editor', 'terminal', 'theme', 'ides', 'framework-ides'].includes(section)) {
			return section as SettingsSection;
		}
		return 'general';
	});
	let isSaving = $state(false);
	let isExporting = $state(false);
	let isResetting = $state(false);

	// Reactive stores
	let settingsData = $derived($settings);
	let loading = $derived($isLoadingSettings);
	let errorMessage = $derived($settingsError);

	onMount(async () => {
		await loadSettings();
	});

	async function loadSettings() {
		try {
			await settingsActions.loadSettings();
		} catch (err) {
			logger.error('Failed to load settings', { context: 'SettingsPanel', 
				error: err
			});
			toast.error('Failed to load settings');
		}
	}

	async function handleSave() {
		if (!settingsData) return;
		
		isSaving = true;
		try {
			await settingsActions.saveSettings(settingsData);
			toast.success('Settings saved successfully');
		} catch (err) {
			logger.error('Failed to save settings', { context: 'SettingsPanel', 
				error: err
			});
			toast.error('Failed to save settings');
		} finally {
			isSaving = false;
		}
	}

	async function handleExport() {
		if (!settingsData) return;
		
		isExporting = true;
		try {
			const exportedSettings = await settingsActions.exportSettings();
			
			// Create download link
			const blob = new Blob([exportedSettings], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `portal-desktop-settings-${new Date().toISOString().split('T')[0]}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
			
			toast.success('Settings exported successfully');
		} catch (err) {
			logger.error('Failed to export settings', { context: 'SettingsPanel', 
				error: err
			});
			toast.error('Failed to export settings');
		} finally {
			isExporting = false;
		}
	}

	async function handleReset() {
		isResetting = true;
		try {
			await settingsActions.resetSettings();
			toast.success('Settings reset to defaults');
		} catch (err) {
			logger.error('Failed to reset settings', { context: 'SettingsPanel', 
				error: err
			});
			toast.error('Failed to reset settings');
		} finally {
			isResetting = false;
		}
	}


	function handleGeneralUpdate(updates: Partial<import('../types').AppSettings>) {
		if (!settingsData) return;
		const newSettings = {
			...settingsData,
			app: { ...settingsData.app, ...updates }
		};
		settingsData = newSettings;
	}

	function handleEditorUpdate(updates: Partial<import('../types').EditorSettings>) {
		if (!settingsData) return;
		const newSettings = {
			...settingsData,
			editor: { ...settingsData.editor, ...updates }
		};
		settingsData = newSettings;
	}

	function handleTerminalUpdate(updates: Partial<import('../types').TerminalSettings>) {
		if (!settingsData) return;
		const newSettings = {
			...settingsData,
			terminal: { ...settingsData.terminal, ...updates }
		};
		settingsData = newSettings;
	}

	function handleThemeUpdate(updates: Partial<import('../types').ThemeSettings>) {
		if (!settingsData) return;
		const newSettings = {
			...settingsData,
			theme: { ...settingsData.theme, ...updates }
		};
		settingsData = newSettings;
	}

	function getSettingsStats() {
		if (!settingsData) return { total: 0, configured: 0 };
		
		const total = 4; // general, editor, terminal, theme
		let configured = 0;
		
		if (settingsData.app) configured++;
		if (settingsData.editor) configured++;
		if (settingsData.terminal) configured++;
		if (settingsData.theme) configured++;
		
		return { total, configured };
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight flex items-center gap-2">
				<Settings class="h-8 w-8" />
				Settings
			</h1>
			<p class="text-muted-foreground">
				Configure your application preferences and behavior
			</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" onclick={handleExport} disabled={isExporting || !settingsData}>
				{#if isExporting}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{:else}
					<Download class="h-4 w-4 mr-2" />
				{/if}
				Export Settings
			</Button>
			<Button variant="outline" onclick={handleReset} disabled={isResetting}>
				{#if isResetting}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{:else}
					<RotateCcw class="h-4 w-4 mr-2" />
				{/if}
				Reset to Defaults
			</Button>
			<Button onclick={handleSave} disabled={isSaving || !settingsData}>
				{#if isSaving}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
				{:else}
					<Save class="h-4 w-4 mr-2" />
				{/if}
				Save Changes
			</Button>
		</div>
	</div>

	<!-- Error Alert -->
	{#if errorMessage}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertTitle>Error</AlertTitle>
			<AlertDescription>
				{errorMessage}
			</AlertDescription>
		</Alert>
	{/if}

	<!-- Settings Stats -->
	{#if settingsData}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<RefreshCw class="h-5 w-5" />
					Settings Overview
				</CardTitle>
				<CardDescription>
					Current configuration status
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="flex items-center gap-4">
					<Badge variant="outline">
						Configured: {getSettingsStats().configured}/{getSettingsStats().total}
					</Badge>
					<Badge variant="outline">
						Theme: {settingsData.app?.theme || 'system'}
					</Badge>
					<Badge variant="outline">
						Language: {settingsData.app?.language || 'en'}
					</Badge>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Settings Content with Sidebar Navigation -->
	<div class="flex flex-col lg:flex-row gap-8">
		<!-- Navigation Sidebar -->
		<div class="lg:w-64 flex-shrink-0">
			<div class="sticky top-8">
				<Card class="p-4">
					<SettingsNavigation
						currentSection={activeSection === 'framework-ides' ? 'ides' : activeSection}
					/>
				</Card>
			</div>
		</div>

		<!-- Main Content -->
		<div class="flex-1 min-w-0">
			{#if loading}
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
					{#each Array(6) as _}
						<Card>
							<CardHeader>
								<div class="h-4 bg-muted animate-pulse rounded"></div>
								<div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
							</CardHeader>
							<CardContent>
								<div class="space-y-2">
									<div class="h-3 bg-muted animate-pulse rounded"></div>
									<div class="h-3 bg-muted animate-pulse rounded w-1/2"></div>
								</div>
							</CardContent>
						</Card>
					{/each}
				</div>
			{:else if settingsData}
				{#if activeSection === 'general'}
					<GeneralSettings 
						settings={settingsData.app} 
						onUpdate={handleGeneralUpdate} 
					/>
				{:else if activeSection === 'editor'}
					<EditorSettings 
						settings={settingsData.editor} 
						onUpdate={handleEditorUpdate} 
					/>
				{:else if activeSection === 'terminal'}
					<TerminalSettings 
						settings={settingsData.terminal} 
						onUpdate={handleTerminalUpdate} 
					/>
				{:else if activeSection === 'ides'}
					<IdeSettings />
				{:else if activeSection === 'framework-ides'}
					<FrameworkIdeSettings />
				{:else if activeSection === 'theme'}
					<ThemeCustomizer 
						settings={settingsData.theme} 
						onUpdate={handleThemeUpdate} 
					/>
				{/if}
			{:else}
				<Card>
					<CardContent class="flex flex-col items-center justify-center py-12">
						<Settings class="h-12 w-12 text-muted-foreground mb-4" />
						<h3 class="text-lg font-semibold mb-2">No Settings Found</h3>
						<p class="text-muted-foreground text-center mb-4">
							Unable to load settings. Please try refreshing the page.
						</p>
						<Button onclick={loadSettings}>
							<RefreshCw class="h-4 w-4 mr-2" />
							Reload Settings
						</Button>
					</CardContent>
				</Card>
			{/if}
		</div>
	</div>

</div>
