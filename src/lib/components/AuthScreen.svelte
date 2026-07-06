<!--
	Authentication Screen
	Shown to unauthorized devices accessing via browser
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import DeviceAuthService from "$lib/services/deviceAuthService";
  import { Shield, Loader2, AlertCircle, CheckCircle } from "@lucide/svelte";

  let { onAuthSuccess = () => {} }: { onAuthSuccess?: () => void } = $props();

  let passcode = $state("");
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state(false);
  let pollingStatus = $state<string | null>(null);
  let deviceInfo = $state(DeviceAuthService.getDeviceInfo());

  onMount(async () => {
    // If a passcode was passed via the QR-code URL, auto-fill and verify it
    // so the user does not have to type anything.
    const urlPasscode = new URLSearchParams(window.location.search).get(
      "passcode",
    );
    if (urlPasscode && /^\d{6}$/.test(urlPasscode)) {
      passcode = urlPasscode;
      // Strip the passcode from the address bar so it isn't left visible or
      // saved in history after use.
      const cleanUrl = new URL(window.location.href);
      cleanUrl.searchParams.delete("passcode");
      window.history.replaceState({}, "", cleanUrl.toString());
      await verifyPasscode();
      return;
    }

    // Otherwise wait for the user to enter the passcode from the host's QR dialog.
    // The phone must NOT generate a passcode — that is the host's shared secret.
  });

  async function verifyPasscode() {
    if (!passcode.trim() || passcode.length !== 6) {
      error = "Please enter a 6-digit passcode";
      return;
    }

    loading = true;
    error = null;
    success = false;

    try {
      await DeviceAuthService.verifyPasscode(passcode.trim(), "temporary");

      // Poll for approval
      await pollForApproval();
    } catch (err) {
      console.error("Failed to verify passcode:", err);
      error = err instanceof Error ? err.message : "Failed to verify passcode";
      passcode = "";
    } finally {
      loading = false;
    }
  }

  async function pollForApproval(maxAttempts: number = 60) {
    const deviceInfo = DeviceAuthService.getDeviceInfo();

    // Poll every 2 seconds for up to 2 minutes
    const { invokeClient } = await import("$lib/utils/invokeClient");

    // Show polling status
    pollingStatus = "Waiting for host approval...";

    for (let i = 0; i < maxAttempts; i++) {
      // Update status message
      if (i > 0 && i % 5 === 0) {
        pollingStatus = `Still waiting for approval... (${i * 2}s)`;
      }

      await new Promise((resolve) => setTimeout(resolve, 2000));

      try {
        // Check device status to see if approved (public command, no auth required)
        const status = await invokeClient.post<{
          approved: boolean;
          access_token: string | null;
          expires_at: string | null;
          message: string;
        }>(
          "get_device_status",
          {
            device_id: deviceInfo.device_id,
          },
          {
            requireAuth: false,
          },
        );

        if (status?.approved && status.access_token) {
          // Device approved! Save the token
          DeviceAuthService.saveAuthState({
            access_token: status.access_token,
            token_expires_at: status.expires_at,
            is_authenticated: true,
          });

          pollingStatus = null;
          success = true;
          setTimeout(() => {
            onAuthSuccess();
          }, 1000);
          return;
        }

        // If status message indicates rejection or error, show it
        if (
          status?.message &&
          (status.message.includes("rejected") ||
            status.message.includes("denied"))
        ) {
          pollingStatus = null;
          error = status.message || "Device request was rejected by the host.";
          loading = false;
          return;
        }
      } catch (err) {
        console.error("Status check error:", err);
        // If it's an authentication error, show it
        if (err instanceof Error && err.message.includes("Authentication")) {
          pollingStatus = null;
          error = "Authentication failed. Please request a new passcode.";
          loading = false;
          return;
        }
      }
    }

    pollingStatus = null;
    error =
      "Approval timeout. The host device may not have approved your request. Please request a new passcode.";
    loading = false;
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter") {
      verifyPasscode();
    }
  }
</script>

<div class="flex min-h-screen items-center justify-center bg-background p-4">
  <Card class="w-full max-w-md">
    <CardHeader class="text-center">
      <div class="mb-4 flex justify-center">
        <div class="rounded-full bg-primary/10 p-3">
          <Shield class="h-8 w-8 text-primary" />
        </div>
      </div>
      <CardTitle>Device Authentication Required</CardTitle>
      <CardDescription>
        This application requires device approval for browser access. Please
        enter the 6-digit passcode provided by the host device.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if error}
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      {/if}

      {#if success}
        <Alert>
          <CheckCircle class="h-4 w-4" />
          <AlertDescription>Device approved! Redirecting...</AlertDescription>
        </Alert>
      {:else}
        <div class="space-y-4">
          {#if pollingStatus}
            <Alert>
              <Loader2 class="h-4 w-4 animate-spin" />
              <AlertDescription>{pollingStatus}</AlertDescription>
            </Alert>
          {/if}

          <div>
            <label for="passcode" class="mb-2 block text-sm font-medium">
              Enter Passcode
            </label>
            <Input
              id="passcode"
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              maxlength={6}
              placeholder="000000"
              bind:value={passcode}
              onkeypress={handleKeyPress}
              disabled={loading}
              class="text-center font-mono text-2xl tracking-widest"
            />
            <p class="mt-2 text-xs text-muted-foreground">
              Get the passcode from the host device's QR code dialog
            </p>
          </div>

          <div class="flex gap-2">
            <Button
              variant="outline"
              onclick={() => window.location.reload()}
              disabled={loading}
              class="flex-1"
            >
              Reload
            </Button>
            <Button
              onclick={verifyPasscode}
              disabled={loading || !passcode.trim()}
              class="flex-1"
            >
              {#if loading}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                Verifying...
              {:else}
                Verify
              {/if}
            </Button>
          </div>

          <div class="space-y-1 border-t pt-2 text-xs text-muted-foreground">
            <p><strong>Device:</strong> {deviceInfo.device_name}</p>
            <p>
              <strong>Device ID:</strong>
              <code class="text-xs">{deviceInfo.device_id}</code>
            </p>
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
