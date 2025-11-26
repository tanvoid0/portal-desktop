<!--
	Device Authentication Guard
	Blocks access to the application for unauthorized devices
	Shows authentication screen for browser access
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import { isTauriEnvironment } from '$lib/utils/tauri';
	import { InvokeClient } from '$lib/utils/invokeClient';
	import DeviceAuthService from '$lib/services/deviceAuthService';
	import type { Snippet } from 'svelte';
	import AuthScreen from './AuthScreen.svelte';

	let { children }: { children: Snippet<[]> } = $props();

	let isAuthenticated = $state(false);
	let isChecking = $state(true);
	let isTauri = $state(false);
	let isLocalhost = $state(false);
	let authCheckInterval: ReturnType<typeof setInterval> | null = $state(null);

	onMount(async () => {
		isTauri = isTauriEnvironment();
		isLocalhost = InvokeClient.isLocalhost();
		
		// In Tauri environment or localhost, always allow access (no auth needed)
		if (isTauri || isLocalhost) {
			isAuthenticated = true;
			isChecking = false;
			return;
		}

		// In browser from remote device, check authentication
		await checkAuthentication();
		
		// Set up periodic re-checking for remote browser access
		if (!isTauri && !isLocalhost) {
			authCheckInterval = setInterval(async () => {
				if (!isTauri && !isLocalhost) {
					await checkAuthentication();
				} else {
					if (authCheckInterval) {
						clearInterval(authCheckInterval);
						authCheckInterval = null;
					}
				}
			}, 5000); // Check every 5 seconds
		}
	});

	onDestroy(() => {
		if (authCheckInterval) {
			clearInterval(authCheckInterval);
			authCheckInterval = null;
		}
	});

	async function checkAuthentication() {
		// Don't show loading spinner on periodic checks
		const wasChecking = isChecking;
		if (!wasChecking) {
			isChecking = true;
		}
		
		try {
			const authState = DeviceAuthService.getAuthState();
			
			if (authState.is_authenticated && authState.access_token) {
				// Verify token is still valid (public command, no auth required)
				try {
					const { invokeClient } = await import('$lib/utils/invokeClient');
					const result = await invokeClient.post<{ valid: boolean }>('verify_access_token', {
						access_token: authState.access_token,
					}, {
						requireAuth: false,
					});

					if (result?.valid) {
						isAuthenticated = true;
						isChecking = false;
						return;
					}
				} catch (err) {
					console.error('Token verification failed:', err);
					// Token verification failed - clear auth state
					DeviceAuthService.clearAuthState();
				}
			}

			// Not authenticated or token invalid
			isAuthenticated = false;
			// Clear invalid auth state
			DeviceAuthService.clearAuthState();
		} catch (error) {
			console.error('Auth check failed:', error);
			isAuthenticated = false;
			DeviceAuthService.clearAuthState();
		} finally {
			// Always stop checking after initial check
			isChecking = false;
		}
	}

	function handleAuthSuccess() {
		isAuthenticated = true;
		// Reload to ensure all components get the auth state
		if (typeof window !== 'undefined') {
			window.location.reload();
		}
	}
</script>

{#if isTauri || isLocalhost}
	<!-- Tauri environment or localhost - always allow access -->
	{@render children()}
{:else if isChecking}
	<!-- Browser - checking authentication -->
	<div class="flex items-center justify-center min-h-screen">
		<div class="flex flex-col items-center gap-4">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
			<p class="text-sm text-muted-foreground">Checking authentication...</p>
		</div>
	</div>
{:else if !isAuthenticated}
	<!-- Browser - not authenticated - BLOCK ACCESS -->
	<AuthScreen onAuthSuccess={handleAuthSuccess} />
{:else}
	<!-- Browser - authenticated - allow access -->
	{@render children()}
{/if}

