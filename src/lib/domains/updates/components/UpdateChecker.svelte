<!--
	Update Checker Component - Displays update status and allows checking/installing updates
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
	import { Progress } from '$lib/components/ui/progress';
	import { 
		RefreshCw, 
		Download, 
		CheckCircle2, 
		AlertCircle, 
		Loader2,
		Info
	} from '@lucide/svelte';
	import { checkForUpdates, installUpdateAndRelaunch, getCurrentVersion, type UpdateInfo } from '../services/updateService';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import { logger } from '$lib/domains/shared';

	let currentVersion = $state<string>('');
	let updateInfo = $state<UpdateInfo | null>(null);
	let isChecking = $state(false);
	let isInstalling = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			currentVersion = await getCurrentVersion();
		} catch (err) {
			logger.error('Failed to load current version', { error: err });
			error = 'Failed to load current version';
		}
	});

	async function handleCheckForUpdates() {
		isChecking = true;
		error = null;
		updateInfo = null;

		try {
			const info = await checkForUpdates();
			updateInfo = info;
			
			if (info.available) {
				toast.success(`Update available: ${info.version}`);
			} else {
				toast.info('You are running the latest version');
			}
		} catch (err: any) {
			const errorMsg = err?.message || 'Failed to check for updates';
			error = errorMsg;
			logger.error('Update check failed', { error: err });
			toast.error(errorMsg);
		} finally {
			isChecking = false;
		}
	}

	async function handleInstallUpdate() {
		if (!updateInfo?.available) {
			return;
		}

		isInstalling = true;
		error = null;

		try {
			toast.info('Installing update... The app will restart automatically.');
			await installUpdateAndRelaunch();
			// Note: The app will relaunch, so this code may not execute
		} catch (err: any) {
			const errorMsg = err?.message || 'Failed to install update';
			error = errorMsg;
			logger.error('Update installation failed', { error: err });
			toast.error(errorMsg);
			isInstalling = false;
		}
	}
</script>

<div class="space-y-4">
	<!-- Current Version -->
	<Card>
		<CardHeader>
			<CardTitle>Current Version</CardTitle>
			<CardDescription>
				Your current application version
			</CardDescription>
		</CardHeader>
		<CardContent>
			<div class="flex items-center gap-2">
				<Badge variant="outline" class="text-lg px-3 py-1">
					{currentVersion || 'Loading...'}
				</Badge>
			</div>
		</CardContent>
	</Card>

	<!-- Update Check -->
	<Card>
		<CardHeader>
			<CardTitle>Check for Updates</CardTitle>
			<CardDescription>
				Manually check for available updates
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			{#if error}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertTitle>Error</AlertTitle>
					<AlertDescription>{error}</AlertDescription>
				</Alert>
			{/if}

			{#if updateInfo}
				{#if updateInfo.available}
					<Alert>
						<Info class="h-4 w-4" />
						<AlertTitle>Update Available</AlertTitle>
						<AlertDescription>
							<div class="space-y-2 mt-2">
								<p>
									<strong>Version {updateInfo.version}</strong> is available.
								</p>
								{#if updateInfo.date}
									<p class="text-sm text-muted-foreground">
										Released: {new Date(updateInfo.date).toLocaleDateString()}
									</p>
								{/if}
								{#if updateInfo.body}
									<div class="text-sm whitespace-pre-wrap">
										{updateInfo.body}
									</div>
								{/if}
							</div>
						</AlertDescription>
					</Alert>

					{#if isInstalling}
						<div class="space-y-2">
							<div class="flex items-center gap-2 text-sm text-muted-foreground">
								<Loader2 class="h-4 w-4 animate-spin" />
								Installing update...
							</div>
							<Progress value={undefined} class="w-full" />
							<p class="text-xs text-muted-foreground">
								The application will restart automatically after installation.
							</p>
						</div>
					{:else}
						<Button 
							onclick={handleInstallUpdate}
							class="w-full"
							size="lg"
						>
							<Download class="h-4 w-4 mr-2" />
							Install Update
						</Button>
					{/if}
				{:else}
					<Alert>
						<CheckCircle2 class="h-4 w-4" />
						<AlertTitle>Up to Date</AlertTitle>
						<AlertDescription>
							You are running the latest version ({updateInfo.version}).
						</AlertDescription>
					</Alert>
				{/if}
			{/if}

			<Button 
				onclick={handleCheckForUpdates}
				disabled={isChecking || isInstalling}
				variant="outline"
				class="w-full"
			>
				{#if isChecking}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					Checking for Updates...
				{:else}
					<RefreshCw class="h-4 w-4 mr-2" />
					Check for Updates
				{/if}
			</Button>
		</CardContent>
	</Card>
</div>

