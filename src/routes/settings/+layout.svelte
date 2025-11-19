<!--
	Settings Layout - Shared layout for all settings pages with sidebar and top bar
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { Snippet } from 'svelte';
	import SettingsNavigation from '$lib/domains/settings/components/SettingsNavigation.svelte';
	import { settingsActions, settings, isLoadingSettings, settingsError } from '$lib/domains/settings/stores/settingsStore';
	import { Button } from '$lib/components/ui/button';
	import { Card } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Settings, Download, RotateCcw, Save, RefreshCw, Loader2, AlertCircle } from '@lucide/svelte';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import { logger } from '$lib/domains/shared';
	import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
	import { get } from 'svelte/store';

	// Get children snippet from props for Svelte 5
	let { children }: { children: Snippet<[]> } = $props();

	let isSaving = $state(false);
	let isExporting = $state(false);
	let isResetting = $state(false);

	// Get current section from URL
	let currentSection = $derived(() => {
		const path = $page.url.pathname;
		if (path === '/settings' || path === '/settings/') return 'general' as const;
		const section = path.replace('/settings/', '').replace(/\/$/, '');
		// Redirect framework-ides to ides
		if (section === 'framework-ides') return 'ides' as const;
		return (section || 'general') as 'general' | 'editor' | 'terminal' | 'theme' | 'ides' | 'updates';
	});

	onMount(async () => {
		// Only load if settings haven't been loaded yet
		const currentSettings = get(settings);
		if (currentSettings) {
			return; // Settings already loaded
		}
		
		try {
			await settingsActions.loadSettings();
		} catch (err) {
			logger.error('Failed to load settings', { context: 'SettingsLayout', error: err });
			toast.error('Failed to load settings');
		}
	});

	async function handleSave() {
		const settingsData = get(settings);
		if (!settingsData) return;
		
		isSaving = true;
		try {
			await settingsActions.saveSettings(settingsData);
			toast.success('Settings saved successfully');
		} catch (err) {
			logger.error('Failed to save settings', { context: 'SettingsLayout', error: err });
			toast.error('Failed to save settings');
		} finally {
			isSaving = false;
		}
	}

	async function handleExport() {
		const settingsData = get(settings);
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
			logger.error('Failed to export settings', { context: 'SettingsLayout', error: err });
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
			logger.error('Failed to reset settings', { context: 'SettingsLayout', error: err });
			toast.error('Failed to reset settings');
		} finally {
			isResetting = false;
		}
	}

	function getSettingsStats() {
		const settingsData = get(settings);
		if (!settingsData) return { total: 0, configured: 0 };
		
		const total = 5; // general, editor, terminal, theme, ides (framework mappings included)
		let configured = 0;
		
		if (settingsData.app) configured++;
		if (settingsData.editor) configured++;
		if (settingsData.terminal) configured++;
		if (settingsData.theme) configured++;
		// IDEs and Framework IDEs are managed separately via services
		
		return { total, configured };
	}
</script>

<svelte:head>
	<title>Settings - Portal Desktop</title>
</svelte:head>

<div class="flex flex-col h-full w-full min-h-0 overflow-hidden">
	<!-- Top Bar -->
	<div class="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 flex-shrink-0">
		<div class="flex items-center justify-between px-6 py-4">
			<div>
				<h1 class="text-2xl font-bold tracking-tight flex items-center gap-2">
					<Settings class="h-6 w-6" />
					Settings
				</h1>
				<p class="text-sm text-muted-foreground mt-1">
					Configure your application preferences and behavior
				</p>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" onclick={handleExport} disabled={isExporting || !get(settings)}>
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
				<Button onclick={handleSave} disabled={isSaving || !get(settings)}>
					{#if isSaving}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{:else}
						<Save class="h-4 w-4 mr-2" />
					{/if}
					Save Changes
				</Button>
			</div>
		</div>

		<!-- Settings Stats Bar -->
		{#if get(settings)}
			{@const currentSettings = get(settings)}
			<div class="px-6 pb-4">
				<div class="flex items-center gap-4">
					<Badge variant="outline">
						Configured: {getSettingsStats().configured}/{getSettingsStats().total}
					</Badge>
					<Badge variant="outline">
						Theme: {currentSettings?.app?.theme || 'system'}
					</Badge>
					<Badge variant="outline">
						Language: {currentSettings?.app?.language || 'en'}
					</Badge>
				</div>
			</div>
		{/if}
	</div>

	<!-- Main Content Area -->
	<div class="flex flex-1 min-h-0 overflow-hidden w-full h-full">
		<!-- Sidebar Navigation -->
		<aside class="w-80 border-r bg-background flex-shrink-0 overflow-y-auto min-w-0">
			<div class="p-4">
				<Card class="p-3">
					<SettingsNavigation currentSection={currentSection()} />
				</Card>
			</div>
		</aside>

		<!-- Page Content -->
		<main class="flex-1 overflow-y-auto min-w-0 min-h-0">
			<div class="p-6 max-w-5xl">
				{#if $isLoadingSettings}
					<div class="flex items-center justify-center py-12">
						<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
						<span class="ml-2 text-muted-foreground">Loading settings...</span>
					</div>
				{:else if $settingsError}
					<Alert variant="destructive">
						<AlertCircle class="h-4 w-4" />
						<AlertTitle>Error</AlertTitle>
						<AlertDescription>
							{$settingsError}
						</AlertDescription>
					</Alert>
				{:else}
					{@render children()}
				{/if}
			</div>
		</main>
	</div>
</div>

