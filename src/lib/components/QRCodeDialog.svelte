<script lang="ts">
	import { onMount } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { NetworkService } from '$lib/services/networkService';
	import DeviceAuthService from '$lib/services/deviceAuthService';
	import { isTauriEnvironment, tauriInvoke } from '$lib/utils/tauri';
	import QRCode from 'qrcode';
	import { Copy, Check, Loader2, AlertCircle, Shield } from 'lucide-svelte';

	let { 
		open = $bindable(false),
		onDeviceRequest = () => {}
	}: { 
		open?: boolean;
		onDeviceRequest?: () => void;
	} = $props();

	let qrCodeDataUrl = $state<string | null>(null);
	let url = $state<string>('');
	let passcode = $state<string>('');
	let loading = $state(false);
	let error = $state<string | null>(null);
	let copied = $state(false);
	let passcodeCopied = $state(false);
	let generatingPasscode = $state(false);

	$effect(() => {
		if (open) {
			generateQRCode();
		} else {
			// Reset state when dialog closes
			qrCodeDataUrl = null;
			url = '';
			error = null;
			copied = false;
		}
	});

	async function generateQRCode() {
		loading = true;
		error = null;
		qrCodeDataUrl = null;
		passcode = '';

		try {
			const port = NetworkService.getPort();
			const appUrl = await NetworkService.getApplicationURL(port);
			url = appUrl;

			// Generate passcode for device authentication
			await generatePasscode();

			if (!passcode) {
				throw new Error('Failed to generate passcode');
			}

			// Generate QR code with URL and passcode
			const qrData = JSON.stringify({
				url: appUrl,
				passcode: passcode,
			});
			
			const dataUrl = await QRCode.toDataURL(qrData, {
				width: 300,
				margin: 2,
				color: {
					dark: '#000000',
					light: '#FFFFFF'
				}
			});

			if (!dataUrl) {
				throw new Error('Failed to generate QR code image');
			}

			qrCodeDataUrl = dataUrl;
		} catch (err) {
			console.error('Failed to generate QR code:', err);
			error = err instanceof Error ? err.message : 'Failed to generate QR code';
			qrCodeDataUrl = null;
		} finally {
			loading = false;
		}
	}

	async function generatePasscode() {
		generatingPasscode = true;
		try {
			const deviceInfo = DeviceAuthService.getDeviceInfo();
			
			let result;
			if (isTauriEnvironment()) {
				// Use Tauri command directly - wrap in request object
				result = await tauriInvoke('generate_device_passcode', {
					request: {
						device_id: deviceInfo.device_id,
						device_name: deviceInfo.device_name,
						device_info: JSON.stringify(deviceInfo.device_info),
					},
				});
			} else {
				// Use API endpoint (for browser access)
				const response = await fetch('/api/tauri/generate_device_passcode', {
					method: 'POST',
					headers: {
						'Content-Type': 'application/json',
					},
					body: JSON.stringify({
						device_id: deviceInfo.device_id,
						device_name: deviceInfo.device_name,
						device_info: JSON.stringify(deviceInfo.device_info),
					}),
				});

				if (!response.ok) {
					throw new Error('Failed to generate passcode');
				}

				result = await response.json();
			}
			
			passcode = result.passcode;
			
			// Notify parent that a device is requesting access
			onDeviceRequest();
		} catch (err) {
			console.error('Failed to generate passcode:', err);
			throw err;
		} finally {
			generatingPasscode = false;
		}
	}

	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(url);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy to clipboard:', err);
		}
	}

	async function copyPasscode() {
		try {
			await navigator.clipboard.writeText(passcode);
			passcodeCopied = true;
			setTimeout(() => {
				passcodeCopied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy passcode:', err);
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-md">
		<Dialog.Header>
			<Dialog.Title>Share via QR Code</Dialog.Title>
			<Dialog.Description>
				Scan this QR code with your smartphone to access the application on your local network.
			</Dialog.Description>
		</Dialog.Header>

		<div class="flex flex-col items-center gap-4 py-4">
			{#if loading}
				<div class="flex flex-col items-center gap-2 py-8">
					<Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
					<p class="text-sm text-muted-foreground">Detecting network address...</p>
				</div>
			{:else if error}
				<div class="flex flex-col items-center gap-2 py-8">
					<AlertCircle class="h-8 w-8 text-destructive" />
					<p class="text-sm text-destructive font-medium">Error</p>
					<p class="text-sm text-muted-foreground text-center">{error}</p>
					<Button variant="outline" size="sm" onclick={generateQRCode} class="mt-2">
						Retry
					</Button>
				</div>
			{:else if qrCodeDataUrl}
				<div class="flex flex-col items-center gap-4">
					<div class="p-4 bg-white rounded-lg border-2 border-border flex items-center justify-center">
						<img src={qrCodeDataUrl} alt="QR Code" class="w-64 h-64 block" />
					</div>
					
					<div class="w-full space-y-3">
						<div class="flex items-center gap-2 p-3 bg-muted rounded-md">
							<code class="text-sm flex-1 break-all">{url}</code>
							<Button
								variant="ghost"
								size="icon"
								onclick={copyToClipboard}
								class="shrink-0"
								title="Copy URL"
							>
								{#if copied}
									<Check class="h-4 w-4 text-green-600" />
								{:else}
									<Copy class="h-4 w-4" />
								{/if}
							</Button>
						</div>
						
						{#if passcode}
							<div class="space-y-2">
								<div class="flex items-center gap-2 p-3 bg-primary/10 rounded-md border border-primary/20">
									<Shield class="h-4 w-4 text-primary" />
									<div class="flex-1">
										<p class="text-xs text-muted-foreground mb-1">Access Passcode</p>
										<div class="flex items-center gap-2">
											<code class="text-lg font-mono font-bold tracking-wider">{passcode}</code>
											<Button
												variant="ghost"
												size="icon"
												onclick={copyPasscode}
												class="shrink-0 h-6 w-6"
												title="Copy Passcode"
											>
												{#if passcodeCopied}
													<Check class="h-3 w-3 text-green-600" />
												{:else}
													<Copy class="h-3 w-3" />
												{/if}
											</Button>
										</div>
									</div>
								</div>
								<p class="text-xs text-muted-foreground text-center">
									Share this passcode with the device. It expires in 5 minutes.
								</p>
							</div>
						{/if}
						
						<p class="text-xs text-muted-foreground text-center">
							Make sure your smartphone is on the same network as this device.
						</p>
					</div>
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			<Dialog.Close>
				<Button variant="outline">
					Close
				</Button>
			</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

