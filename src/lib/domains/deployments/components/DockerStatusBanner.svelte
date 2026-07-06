<!--
	Docker Status Banner - Shows Docker daemon status with start/retry actions
-->

<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "$lib/components/ui/alert";
  import { Badge } from "$lib/components/ui/badge";
  import {
    deploymentActions,
    dockerStatus,
    isStartingDocker,
    isDockerOffline,
  } from "../stores/deploymentStore";
  import { toast } from "$lib/utils/toast";
  import { Container, Loader2, Play, RefreshCw } from "@lucide/svelte";

  let {
    onReady,
  }: {
    onReady?: () => void | Promise<void>;
  } = $props();

  let status = $derived($dockerStatus);
  let starting = $derived($isStartingDocker);
  let offline = $derived($isDockerOffline);

  async function handleStartDocker() {
    try {
      await deploymentActions.startDocker();
      toast.success("Docker is running");
      await onReady?.();
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to start Docker",
      );
    }
  }

  async function handleRetry() {
    try {
      const nextStatus = await deploymentActions.checkDockerStatus();
      if (nextStatus.running) {
        toast.success("Docker is running");
        await onReady?.();
      }
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to check Docker status",
      );
    }
  }
</script>

{#if status}
  <Alert variant={offline ? "destructive" : "default"}>
    <Container class="h-4 w-4" />
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div class="space-y-1">
        <AlertTitle class="flex items-center gap-2">
          Docker
          {#if status.running}
            <Badge variant="default" class="bg-green-100 text-green-800">
              Running
            </Badge>
            {#if status.version}
              <span class="text-sm font-normal text-muted-foreground">
                v{status.version}
              </span>
            {/if}
          {:else if starting}
            <Badge variant="outline">Starting...</Badge>
          {:else if !status.installed}
            <Badge variant="destructive">Not Installed</Badge>
          {:else}
            <Badge variant="destructive">Offline</Badge>
          {/if}
        </AlertTitle>
        <AlertDescription>
          {#if status.running}
            Docker is ready. Container operations are available.
          {:else if starting}
            Docker Desktop is starting. This can take up to a minute.
          {:else}
            {status.message ??
              "Docker is not running. Start Docker Desktop to manage containers."}
          {/if}
        </AlertDescription>
      </div>

      {#if !status.running}
        <div class="flex shrink-0 gap-2">
          {#if status.installed}
            <Button
              size="sm"
              onclick={handleStartDocker}
              disabled={starting}
            >
              {#if starting}
                <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                Starting...
              {:else}
                <Play class="mr-2 h-4 w-4" />
                Start Docker
              {/if}
            </Button>
          {/if}
          <Button
            size="sm"
            variant="outline"
            onclick={handleRetry}
            disabled={starting}
          >
            <RefreshCw class="mr-2 h-4 w-4" />
            Retry
          </Button>
        </div>
      {/if}
    </div>
  </Alert>
{/if}
