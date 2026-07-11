<script lang="ts">
  import { onMount } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { tauriInvoke } from "$lib/utils/tauri";
  import { AlertCircle, CheckCircle, Clock, XCircle } from "@lucide/svelte";

  interface PendingDevice {
    device_id: string;
    device_name: string;
    device_info: any;
    created_at: string;
  }

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let pendingDevices = $state<PendingDevice[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let approvingDeviceId = $state<string | null>(null);

  $effect(() => {
    if (open) {
      loadPendingDevices();
      // Poll for new devices every 5 seconds (less aggressive)
      const interval = setInterval(() => {
        // Only poll if dialog is still open
        if (open) {
          loadPendingDevices();
        }
      }, 5000);
      return () => clearInterval(interval);
    }
  });

  async function loadPendingDevices() {
    try {
      loading = true;
      error = null;
      const devices = await tauriInvoke<PendingDevice[]>(
        "get_pending_device_approvals",
      );

      // Filter out localhost requests - only show external device requests
      const filteredDevices = devices.filter((device) => {
        const info = getDeviceInfo(device);

        // Check if IP is localhost
        const isLocalhostIP =
          info.ip === "127.0.0.1" ||
          info.ip === "localhost" ||
          info.ip === "::1" ||
          info.ip === "[::1]";

        // Also check device_info JSON string for localhost indicators
        const deviceInfoStr =
          typeof device.device_info === "string"
            ? device.device_info
            : JSON.stringify(device.device_info);
        const hasLocalhostIndicator =
          deviceInfoStr.includes("localhost") ||
          deviceInfoStr.includes("127.0.0.1") ||
          deviceInfoStr.includes("::1");

        // Only include if it's NOT localhost
        return !isLocalhostIP && !hasLocalhostIndicator;
      });

      pendingDevices = filteredDevices;
    } catch (err) {
      console.error("[DeviceApproval] Failed to load pending devices:", err);
      error =
        err instanceof Error ? err.message : "Failed to load pending devices";
    } finally {
      loading = false;
    }
  }

  async function approveDevice(
    deviceId: string,
    approvalType: "temporary" | "long_term",
  ) {
    try {
      approvingDeviceId = deviceId;
      const result = await tauriInvoke("approve_device", {
        request: {
          device_id: deviceId,
          approval_type: approvalType,
        },
      });

      // The result contains access_token that should be sent to the device
      // For now, the device will poll for it via verify_access_token
      // In a full implementation, we'd send this via WebSocket or Server-Sent Events

      // Reload pending devices
      await loadPendingDevices();
    } catch (err) {
      console.error("[DeviceApproval] Failed to approve device:", err);
      error = err instanceof Error ? err.message : "Failed to approve device";
    } finally {
      approvingDeviceId = null;
    }
  }

  function getDeviceInfo(device: PendingDevice) {
    try {
      const info =
        typeof device.device_info === "string"
          ? JSON.parse(device.device_info)
          : device.device_info;
      return {
        userAgent: info.userAgent || "Unknown",
        platform: info.platform || "Unknown",
        ip: info.ip || "Unknown",
      };
    } catch {
      return {
        userAgent: "Unknown",
        platform: "Unknown",
        ip: "Unknown",
      };
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[80vh] max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>Device Approval Requests</Dialog.Title>
      <Dialog.Description>
        Review and approve devices requesting access to the application.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex max-h-[60vh] flex-col gap-4 overflow-y-auto py-4">
      {#if loading && pendingDevices.length === 0}
        <div class="flex items-center justify-center py-8">
          <div
            class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"
          ></div>
        </div>
      {:else if error}
        <div class="flex items-center gap-2 rounded-md bg-destructive/10 p-4">
          <AlertCircle class="h-5 w-5 text-destructive" />
          <p class="text-sm text-destructive">{error}</p>
        </div>
      {:else if pendingDevices.length === 0}
        <div class="flex flex-col items-center justify-center py-8 text-center">
          <CheckCircle class="mb-2 h-12 w-12 text-muted-foreground" />
          <p class="text-sm text-muted-foreground">
            No pending device approvals
          </p>
        </div>
      {:else}
        {#each pendingDevices as device}
          {@const info = getDeviceInfo(device)}
          <div class="space-y-3 rounded-lg border p-4">
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="mb-2 flex items-center gap-2">
                  <h4 class="font-semibold">{device.device_name}</h4>
                  <Badge variant="outline" class="text-xs">
                    <Clock class="mr-1 h-3 w-3" />
                    Pending
                  </Badge>
                </div>
                <div class="space-y-1 text-sm text-muted-foreground">
                  <p><strong>Platform:</strong> {info.platform}</p>
                  <p>
                    <strong>User Agent:</strong>
                    <code class="text-xs">{info.userAgent}</code>
                  </p>
                  <p><strong>IP Address:</strong> {info.ip}</p>
                  <p>
                    <strong>Requested:</strong>
                    {new Date(device.created_at).toLocaleString()}
                  </p>
                </div>
              </div>
            </div>
            <div class="divider-edge-t divider-edge-full flex items-center gap-2 pt-2">
              <Button
                variant="outline"
                size="sm"
                onclick={() => approveDevice(device.device_id, "temporary")}
                disabled={approvingDeviceId === device.device_id}
                class="flex-1"
              >
                {approvingDeviceId === device.device_id
                  ? "Approving..."
                  : "Approve (24h)"}
              </Button>
              <Button
                variant="default"
                size="sm"
                onclick={() => approveDevice(device.device_id, "long_term")}
                disabled={approvingDeviceId === device.device_id}
                class="flex-1"
              >
                {approvingDeviceId === device.device_id
                  ? "Approving..."
                  : "Approve (30 days)"}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onclick={() => {
                  // Remove from list (could add reject functionality later)
                  pendingDevices = pendingDevices.filter(
                    (d) => d.device_id !== device.device_id,
                  );
                }}
                title="Dismiss"
              >
                <XCircle class="h-4 w-4" />
              </Button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => loadPendingDevices()}>
        Refresh
      </Button>
      <Dialog.Close>
        <Button>Close</Button>
      </Dialog.Close>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
