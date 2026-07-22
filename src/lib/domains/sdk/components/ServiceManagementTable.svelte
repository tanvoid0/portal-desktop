<!--
	Service Management Table - Comprehensive service management with all controls
	Shows services with version selection, status, and action controls
-->

<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox";
  import Select from "$lib/components/ui/select.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription } from "$lib/components/ui/alert";
  import { Progress } from "$lib/components/ui/progress";
  import {
    Play,
    Square,
    RefreshCw,
    Settings,
    FileText,
    ExternalLink,
    AlertTriangle,
    CheckCircle,
    XCircle,
    Clock,
    Activity,
    Search,
    Filter,
    Plus,
    Trash2,
    Eye,
    EyeOff,
  } from "@lucide/svelte";

  interface ServiceInfo {
    id: string;
    name: string;
    description: string;
    status: "running" | "stopped" | "starting" | "stopping" | "error";
    version: string;
    available_versions: string[];
    pid?: number;
    port?: number;
    url?: string;
    health_status: "healthy" | "unhealthy" | "unknown";
    last_started?: string;
    uptime?: number;
    config: Record<string, any>;
  }

  let { sdkType }: { sdkType: string } = $props();

  // State
  let services = $state<ServiceInfo[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let searchTerm = $state("");
  let statusFilter = $state<string>("all");
  let selectedServices = $state<Set<string>>(new Set());
  let showLogs = $state<Set<string>>(new Set());

  // Initialize
  onMount(() => {
    loadServices();
    // Set up auto-refresh
    const interval = setInterval(loadServices, 5000);
    return () => clearInterval(interval);
  });

  async function loadServices() {
    loading = true;
    error = null;

    try {
      const result = await invoke("get_services", { sdkType });
      services = Array.isArray(result) ? result : [];
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load services";
      console.error("Failed to load services:", err);
    } finally {
      loading = false;
    }
  }

  async function startService(serviceId: string) {
    loading = true;
    error = null;

    try {
      await invoke("start_service", { serviceId });
      await loadServices();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to start service";
      console.error("Failed to start service:", err);
    } finally {
      loading = false;
    }
  }

  async function stopService(serviceId: string) {
    loading = true;
    error = null;

    try {
      await invoke("stop_service", { serviceId });
      await loadServices();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to stop service";
      console.error("Failed to stop service:", err);
    } finally {
      loading = false;
    }
  }

  async function restartService(serviceId: string) {
    loading = true;
    error = null;

    try {
      await invoke("restart_service", { serviceId });
      await loadServices();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to restart service";
      console.error("Failed to restart service:", err);
    } finally {
      loading = false;
    }
  }

  async function changeServiceVersion(serviceId: string, version: string) {
    loading = true;
    error = null;

    try {
      await invoke("change_service_version", { serviceId, version });
      await loadServices();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "Failed to change service version";
      console.error("Failed to change service version:", err);
    } finally {
      loading = false;
    }
  }

  function getStatusIcon(status: string) {
    switch (status) {
      case "running":
        return CheckCircle;
      case "stopped":
        return XCircle;
      case "starting":
        return Clock;
      case "stopping":
        return Clock;
      case "error":
        return AlertTriangle;
      default:
        return XCircle;
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case "running":
        return "text-green-600";
      case "stopped":
        return "text-gray-600";
      case "starting":
        return "text-blue-600";
      case "stopping":
        return "text-orange-600";
      case "error":
        return "text-red-600";
      default:
        return "text-gray-600";
    }
  }

  function getStatusBadgeVariant(status: string) {
    switch (status) {
      case "running":
        return "default";
      case "stopped":
        return "secondary";
      case "starting":
        return "outline";
      case "stopping":
        return "outline";
      case "error":
        return "destructive";
      default:
        return "secondary";
    }
  }

  function getHealthStatusIcon(health: string) {
    switch (health) {
      case "healthy":
        return CheckCircle;
      case "unhealthy":
        return XCircle;
      case "unknown":
        return AlertTriangle;
      default:
        return AlertTriangle;
    }
  }

  function getHealthStatusColor(health: string) {
    switch (health) {
      case "healthy":
        return "text-green-600";
      case "unhealthy":
        return "text-red-600";
      case "unknown":
        return "text-yellow-600";
      default:
        return "text-gray-600";
    }
  }

  function formatUptime(seconds?: number) {
    if (!seconds) return "Unknown";
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
  }

  function formatDate(timestamp?: string) {
    if (!timestamp) return "Never";
    return new Date(timestamp).toLocaleString();
  }

  function openServiceUrl(url?: string) {
    if (url) {
      window.open(url, "_blank");
    }
  }

  function toggleServiceLogs(serviceId: string) {
    const newShowLogs = new Set(showLogs);
    if (newShowLogs.has(serviceId)) {
      newShowLogs.delete(serviceId);
    } else {
      newShowLogs.add(serviceId);
    }
    showLogs = newShowLogs;
  }

  function toggleServiceSelection(serviceId: string) {
    const newSelection = new Set(selectedServices);
    if (newSelection.has(serviceId)) {
      newSelection.delete(serviceId);
    } else {
      newSelection.add(serviceId);
    }
    selectedServices = newSelection;
  }

  function selectAllServices() {
    selectedServices = new Set(filteredServices.map((s) => s.id));
  }

  function clearSelection() {
    selectedServices = new Set();
  }

  // Filter services based on search term and status
  let filteredServices = $derived.by(() => {
    let filtered = services;

    // Search filter
    if (searchTerm.trim()) {
      const term = searchTerm.toLowerCase();
      filtered = filtered.filter(
        (service) =>
          service.name.toLowerCase().includes(term) ||
          service.description.toLowerCase().includes(term) ||
          service.version.toLowerCase().includes(term),
      );
    }

    // Status filter
    if (statusFilter !== "all") {
      filtered = filtered.filter((service) => service.status === statusFilter);
    }

    return filtered;
  });
</script>

<Card class="w-full">
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle class="text-xl">Service Management</CardTitle>
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onclick={loadServices}
          disabled={loading}
        >
          <RefreshCw class="h-4 w-4" />
        </Button>
      </div>
    </div>
  </CardHeader>

  <CardContent class="space-y-4">
    <!-- Filters -->
    <div class="flex items-center gap-4">
      <div class="relative flex-1">
        <Search
          class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
        />
        <Input
          placeholder="Search services..."
          bind:value={searchTerm}
          class="pl-10"
        />
      </div>

      <Select
        bind:value={statusFilter}
        options={[
          { value: "all", label: "All Status" },
          { value: "running", label: "Running" },
          { value: "stopped", label: "Stopped" },
          { value: "starting", label: "Starting" },
          { value: "stopping", label: "Stopping" },
          { value: "error", label: "Error" },
        ]}
        class="min-w-[140px]"
      />
    </div>

    <!-- Error Alert -->
    {#if error}
      <Alert variant="destructive">
        <AlertTriangle class="h-4 w-4" />
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <!-- Loading State -->
    {#if loading && services.length === 0}
      <div class="flex items-center justify-center py-8">
        <div
          class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"
        ></div>
        <span class="ml-2">Loading services...</span>
      </div>
    {:else if filteredServices.length === 0}
      <div class="py-8 text-center text-muted-foreground">
        <Activity class="mx-auto mb-4 h-12 w-12 opacity-50" />
        <p>No services found</p>
        {#if searchTerm || statusFilter !== "all"}
          <p class="text-sm">Try adjusting your filters</p>
        {:else}
          <p class="text-sm">Start a service to get started</p>
        {/if}
      </div>
    {:else}
      <!-- Services Table -->
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="border-b">
              <th class="p-3 text-left">
                <Checkbox
                  checked={selectedServices.size === filteredServices.length &&
                    filteredServices.length > 0}
                  onCheckedChange={(checked) => {
                    if (checked) selectAllServices();
                    else clearSelection();
                  }}
                />
              </th>
              <th class="p-3 text-left font-medium">Service</th>
              <th class="p-3 text-left font-medium">Version</th>
              <th class="p-3 text-left font-medium">Status</th>
              <th class="p-3 text-left font-medium">Health</th>
              <th class="p-3 text-left font-medium">Port/PID</th>
              <th class="p-3 text-left font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredServices as service}
              {@const StatusIcon = getStatusIcon(service.status)}
              {@const HealthIcon = getHealthStatusIcon(service.health_status)}
              <tr class="border-b transition-colors hover:bg-muted/50">
                <td class="p-3">
                  <Checkbox
                    checked={selectedServices.has(service.id)}
                    onCheckedChange={() => toggleServiceSelection(service.id)}
                  />
                </td>

                <td class="p-3">
                  <div class="space-y-1">
                    <div class="font-medium">{service.name}</div>
                    <div class="text-sm text-muted-foreground">
                      {service.description}
                    </div>
                    {#if service.last_started}
                      <div class="text-xs text-muted-foreground">
                        Last started: {formatDate(service.last_started)}
                      </div>
                    {/if}
                  </div>
                </td>

                <td class="p-3">
                  <Select
                    defaultValue={service.version}
                    options={service.available_versions}
                    onSelect={(value) => changeServiceVersion(service.id, value)}
                    disabled={loading}
                    class="min-w-[100px]"
                  />
                </td>

                <td class="p-3">
                  <div class="flex items-center gap-2">
                    <Badge variant={getStatusBadgeVariant(service.status)}>
                      <StatusIcon class="mr-1 h-3 w-3" />
                      {service.status}
                    </Badge>
                    {#if service.status === "starting" || service.status === "stopping"}
                      <Progress value={50} class="h-2 w-16" />
                    {/if}
                  </div>
                </td>

                <td class="p-3">
                  <div class="flex items-center gap-2">
                    <HealthIcon
                      class="h-4 w-4 {getHealthStatusColor(
                        service.health_status,
                      )}"
                    />
                    <span class="text-sm capitalize"
                      >{service.health_status}</span
                    >
                  </div>
                </td>

                <td class="p-3">
                  <div class="space-y-1">
                    {#if service.pid}
                      <div class="text-sm">PID: {service.pid}</div>
                    {/if}
                    {#if service.port}
                      <div class="text-sm">Port: {service.port}</div>
                    {/if}
                    {#if service.uptime}
                      <div class="text-xs text-muted-foreground">
                        Uptime: {formatUptime(service.uptime)}
                      </div>
                    {/if}
                  </div>
                </td>

                <td class="p-3">
                  <div class="flex items-center gap-1">
                    {#if service.status === "running"}
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => stopService(service.id)}
                        disabled={loading}
                      >
                        <Square class="mr-1 h-3 w-3" />
                        Stop
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={() => restartService(service.id)}
                        disabled={loading}
                      >
                        <RefreshCw class="mr-1 h-3 w-3" />
                        Restart
                      </Button>
                    {:else if service.status === "stopped"}
                      <Button
                        variant="default"
                        size="sm"
                        onclick={() => startService(service.id)}
                        disabled={loading}
                      >
                        <Play class="mr-1 h-3 w-3" />
                        Start
                      </Button>
                    {/if}

                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={() => toggleServiceLogs(service.id)}
                      title="Toggle logs"
                    >
                      {#if showLogs.has(service.id)}
                        <EyeOff class="h-3 w-3" />
                      {:else}
                        <Eye class="h-3 w-3" />
                      {/if}
                    </Button>

                    <Button variant="ghost" size="sm" title="Configure service">
                      <Settings class="h-3 w-3" />
                    </Button>

                    {#if service.url}
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => openServiceUrl(service.url)}
                        title="Open in browser"
                      >
                        <ExternalLink class="h-3 w-3" />
                      </Button>
                    {/if}
                  </div>
                </td>
              </tr>

              <!-- Service Logs (if expanded) -->
              {#if showLogs.has(service.id)}
                <tr class="border-b">
                  <td colspan="7" class="bg-muted/30 p-4">
                    <div class="space-y-2">
                      <div class="flex items-center gap-2">
                        <FileText class="h-4 w-4" />
                        <span class="font-medium">Service Logs</span>
                      </div>
                      <div
                        class="max-h-32 overflow-y-auto rounded bg-black p-3 font-mono text-sm text-green-400"
                      >
                        <div>Loading logs...</div>
                      </div>
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Service Stats -->
      <div class="divider-edge-t divider-edge-full flex items-center justify-between pt-4">
        <div class="text-sm text-muted-foreground">
          {filteredServices.length} services
          {#if selectedServices.size > 0}
            • {selectedServices.size} selected
          {/if}
        </div>

        {#if selectedServices.size > 0}
          <div class="flex items-center gap-2">
            <Button variant="outline" size="sm">
              <Play class="mr-2 h-4 w-4" />
              Start Selected
            </Button>
            <Button variant="outline" size="sm">
              <Square class="mr-2 h-4 w-4" />
              Stop Selected
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="text-red-500 hover:text-red-700"
            >
              <Trash2 class="mr-2 h-4 w-4" />
              Remove Selected
            </Button>
          </div>
        {/if}
      </div>
    {/if}
  </CardContent>
</Card>
