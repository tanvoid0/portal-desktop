<!--
	General Settings - Application preferences and general configuration
-->

<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import Select from '@/lib/components/ui/select.svelte';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { settingsService } from '../services/settingsService';
	import type { AppSettings } from '../types';

	interface Props {
		settings: AppSettings | null;
		onUpdate: (updates: Partial<AppSettings>) => void;
	}

	let { settings: settingsProp, onUpdate }: Props = $props();

	// Ensure settings has all required nested properties
	const defaultSettings: AppSettings = {
		theme: 'system',
		language: 'en',
		timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
		dateFormat: 'YYYY-MM-DD',
		timeFormat: '24h',
		windowState: {
			width: 1280,
			height: 720,
			maximized: false,
			rememberPosition: true
		},
		startupBehavior: {
			openLastSession: false,
			restoreWindows: true,
			showWelcomeScreen: true,
			minimizeToTray: false,
			startMinimized: false
		},
		notifications: {
			enabled: true,
			desktopNotifications: true,
			soundEnabled: false,
			showInTaskbar: true,
			types: {
				success: true,
				info: true,
				warning: true,
				error: true,
				updates: true,
				security: true
			}
		},
		privacy: {
			analytics: false,
			crashReports: true,
			telemetry: false,
			usageData: false,
			marketing: false
		},
		updates: {
			autoCheck: true,
			autoDownload: false,
			autoInstall: false,
			checkInterval: 24,
			channel: 'stable',
			notifyOnUpdate: true
		}
	};

	// Merge settings with defaults to ensure all properties exist
	const settings = $derived(settingsProp ? {
		...defaultSettings,
		...settingsProp,
		windowState: { ...defaultSettings.windowState, ...(settingsProp.windowState || {}) },
		startupBehavior: { ...defaultSettings.startupBehavior, ...(settingsProp.startupBehavior || {}) },
		notifications: {
			...defaultSettings.notifications,
			...(settingsProp.notifications || {}),
			types: { ...defaultSettings.notifications.types, ...(settingsProp.notifications?.types || {}) }
		},
		privacy: { ...defaultSettings.privacy, ...(settingsProp.privacy || {}) },
		updates: { ...defaultSettings.updates, ...(settingsProp.updates || {}) }
	} : defaultSettings);

	// Reactive local state for inputs
	let timezone = $state('');
	let dateFormat = $state('YYYY-MM-DD');

	// Update local state when settings change
	$effect(() => {
		timezone = settings.timezone || '';
		dateFormat = settings.dateFormat || 'YYYY-MM-DD';
	});

	const availableThemes = settingsService.getAvailableThemes();
	const availableLanguages = settingsService.getAvailableLanguages();
</script>

<div class="space-y-6">
	<!-- Appearance -->
	<Card>
		<CardHeader>
			<CardTitle>Appearance</CardTitle>
			<CardDescription>Configure the look and feel of the application</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="space-y-2">
				<Label for="theme">Theme</Label>
				<Select
					defaultValue={settings.theme}
					options={availableThemes.map(t => ({ value: t.id, label: t.name }))}
					onSelect={(value) => onUpdate({ theme: value as 'light' | 'dark' | 'system' })}
					placeholder="Select theme"
				/>
			</div>

			<div class="space-y-2">
				<Label for="language">Language</Label>
				<Select
					defaultValue={settings.language}
					options={availableLanguages.map(l => ({ value: l.id, label: `${l.name} (${l.native})` }))}
					onSelect={(value) => onUpdate({ language: value })}
					placeholder="Select language"
				/>
			</div>

			<div class="space-y-2">
				<Label for="timezone">Timezone</Label>
				<Input
					id="timezone"
					bind:value={timezone}
					onchange={() => onUpdate({ timezone })}
				/>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="dateFormat">Date Format</Label>
					<Input
						id="dateFormat"
						bind:value={dateFormat}
						onchange={() => onUpdate({ dateFormat })}
						placeholder="YYYY-MM-DD"
					/>
				</div>

				<div class="space-y-2">
					<Label for="timeFormat">Time Format</Label>
					<Select
						defaultValue={settings.timeFormat}
						options={['12h', '24h']}
						onSelect={(value) => onUpdate({ timeFormat: value as '12h' | '24h' })}
						placeholder="Select format"
					/>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Startup Behavior -->
	<Card>
		<CardHeader>
			<CardTitle>Startup Behavior</CardTitle>
			<CardDescription>Configure how the application starts</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="openLastSession">Open Last Session</Label>
					<p class="text-sm text-muted-foreground">Automatically open the last session on startup</p>
				</div>
				<Switch
					id="openLastSession"
					checked={settings.startupBehavior?.openLastSession ?? false}
					onclick={() => onUpdate({
						startupBehavior: { ...settings.startupBehavior, openLastSession: !(settings.startupBehavior?.openLastSession ?? false) }
					})}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="restoreWindows">Restore Windows</Label>
					<p class="text-sm text-muted-foreground">Restore window positions and sizes</p>
				</div>
				<Switch
					id="restoreWindows"
					checked={settings.startupBehavior?.restoreWindows ?? false}
					onclick={() => onUpdate({
						startupBehavior: { ...settings.startupBehavior, restoreWindows: !(settings.startupBehavior?.restoreWindows ?? false) }
					})}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="showWelcomeScreen">Show Welcome Screen</Label>
					<p class="text-sm text-muted-foreground">Display welcome screen on first launch</p>
				</div>
				<Switch
					id="showWelcomeScreen"
					checked={settings.startupBehavior?.showWelcomeScreen ?? false}
					onclick={() => onUpdate({
						startupBehavior: { ...settings.startupBehavior, showWelcomeScreen: !(settings.startupBehavior?.showWelcomeScreen ?? false) }
					})}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="minimizeToTray">Minimize to Tray</Label>
					<p class="text-sm text-muted-foreground">Minimize to system tray instead of taskbar</p>
				</div>
				<Switch
					id="minimizeToTray"
					checked={settings.startupBehavior?.minimizeToTray ?? false}
					onclick={() => onUpdate({
						startupBehavior: { ...settings.startupBehavior, minimizeToTray: !(settings.startupBehavior?.minimizeToTray ?? false) }
					})}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Notifications -->
	<Card>
		<CardHeader>
			<CardTitle>Notifications</CardTitle>
			<CardDescription>Configure notification preferences</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="notificationsEnabled">Enable Notifications</Label>
					<p class="text-sm text-muted-foreground">Receive notifications from the application</p>
				</div>
				<Switch
					id="notificationsEnabled"
					checked={settings.notifications?.enabled ?? false}
					onclick={() => onUpdate({
						notifications: { ...settings.notifications, enabled: !(settings.notifications?.enabled ?? false) }
					})}
				/>
			</div>

			<Separator />

			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<Label for="desktopNotifications">Desktop Notifications</Label>
					<Switch
						id="desktopNotifications"
						checked={settings.notifications?.desktopNotifications ?? false}
						disabled={!(settings.notifications?.enabled ?? false)}
						onclick={() => onUpdate({
							notifications: { ...settings.notifications, desktopNotifications: !(settings.notifications?.desktopNotifications ?? false) }
						})}
					/>
				</div>

				<div class="flex items-center justify-between">
					<Label for="soundEnabled">Sound Notifications</Label>
					<Switch
						id="soundEnabled"
						checked={settings.notifications?.soundEnabled ?? false}
						disabled={!(settings.notifications?.enabled ?? false)}
						onclick={() => onUpdate({
							notifications: { ...settings.notifications, soundEnabled: !(settings.notifications?.soundEnabled ?? false) }
						})}
					/>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Privacy -->
	<Card>
		<CardHeader>
			<CardTitle>Privacy</CardTitle>
			<CardDescription>Configure privacy and data collection settings</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="crashReports">Send Crash Reports</Label>
					<p class="text-sm text-muted-foreground">Automatically send crash reports to help improve the app</p>
				</div>
				<Switch
					id="crashReports"
					checked={settings.privacy?.crashReports ?? false}
					onclick={() => onUpdate({
						privacy: { ...settings.privacy, crashReports: !(settings.privacy?.crashReports ?? false) }
					})}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="analytics">Analytics</Label>
					<p class="text-sm text-muted-foreground">Share usage analytics to improve features</p>
				</div>
				<Switch
					id="analytics"
					checked={settings.privacy?.analytics ?? false}
					onclick={() => onUpdate({
						privacy: { ...settings.privacy, analytics: !(settings.privacy?.analytics ?? false) }
					})}
				/>
			</div>
		</CardContent>
	</Card>

	<!-- Updates -->
	<Card>
		<CardHeader>
			<CardTitle>Updates</CardTitle>
			<CardDescription>Configure automatic update behavior</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="autoCheck">Auto-check for Updates</Label>
					<p class="text-sm text-muted-foreground">Automatically check for updates</p>
				</div>
				<Switch
					id="autoCheck"
					checked={settings.updates?.autoCheck ?? false}
					onclick={() => onUpdate({
						updates: { ...settings.updates, autoCheck: !(settings.updates?.autoCheck ?? false) }
					})}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div class="space-y-0.5">
					<Label for="notifyOnUpdate">Notify on Update</Label>
					<p class="text-sm text-muted-foreground">Show notification when updates are available</p>
				</div>
				<Switch
					id="notifyOnUpdate"
					checked={settings.updates?.notifyOnUpdate ?? false}
					disabled={!(settings.updates?.autoCheck ?? false)}
					onclick={() => onUpdate({
						updates: { ...settings.updates, notifyOnUpdate: !(settings.updates?.notifyOnUpdate ?? false) }
					})}
				/>
			</div>

			<div class="space-y-2">
				<Label for="updateChannel">Update Channel</Label>
				<Select
					defaultValue={settings.updates?.channel || 'stable'}
					options={['stable', 'beta', 'alpha']}
					onSelect={(value) => onUpdate({
						updates: { ...settings.updates, channel: value as 'stable' | 'beta' | 'alpha' }
					})}
					placeholder="Select channel"
				/>
			</div>
		</CardContent>
	</Card>
</div>

