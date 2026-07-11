<!--
	Update Checker Component - Displays update status and allows checking/installing updates
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Alert,
    AlertDescription,
    AlertTitle,
  } from "$lib/components/ui/alert";
  import { Progress } from "$lib/components/ui/progress";
  import {
    RefreshCw,
    Download,
    CheckCircle2,
    AlertCircle,
    Loader2,
    Info,
    WifiOff,
  } from "@lucide/svelte";
  import {
    checkForUpdates,
    installUpdateAndRelaunch,
    getCurrentVersion,
    type UpdateErrorInfo,
    type UpdateInfo,
  } from "../services/updateService";
  import { toast } from "$lib/utils/toast";
  import { logger } from "$lib/domains/shared";

  let currentVersion = $state<string | null>(null);
  let updateInfo = $state<UpdateInfo | null>(null);
  let checkError = $state<UpdateErrorInfo | null>(null);
  let installError = $state<UpdateErrorInfo | null>(null);
  let isChecking = $state(false);
  let isInstalling = $state(false);
  let versionLoadFailed = $state(false);

  onMount(async () => {
    const version = await getCurrentVersion();
    if (version) {
      currentVersion = version;
    } else {
      versionLoadFailed = true;
      logger.warn("Current version unavailable in update settings");
    }
  });

  function notifyCheckError(error: UpdateErrorInfo) {
    toast.warning(error.title, { description: error.message });
  }

  function notifyInstallError(error: UpdateErrorInfo) {
    toast.error(error.title, { description: error.message });
  }

  async function handleCheckForUpdates() {
    isChecking = true;
    checkError = null;
    updateInfo = null;

    const result = await checkForUpdates();

    if (result.status === "available") {
      updateInfo = result.info;
      toast.success(`Update available: ${result.info.version}`);
    } else if (result.status === "current") {
      updateInfo = result.info;
      toast.info("You are running the latest version");
    } else {
      checkError = result.error;
      notifyCheckError(result.error);
    }

    isChecking = false;
  }

  async function handleInstallUpdate() {
    if (!updateInfo?.available) {
      return;
    }

    isInstalling = true;
    installError = null;

    toast.info("Installing update... The app will restart automatically.");

    const result = await installUpdateAndRelaunch();

    if (result?.status === "error") {
      installError = result.error;
      notifyInstallError(result.error);
      isInstalling = false;
    }
  }
</script>

<div class="space-y-4">
  <!-- Current Version -->
  <Card>
    <CardHeader>
      <CardTitle>Current Version</CardTitle>
      <CardDescription>Your current application version</CardDescription>
    </CardHeader>
    <CardContent class="space-y-3">
      <div class="flex items-center gap-2">
        <Badge variant="outline" class="px-3 py-1 text-lg">
          {#if currentVersion}
            {currentVersion}
          {:else if versionLoadFailed}
            Unknown
          {:else}
            Loading...
          {/if}
        </Badge>
      </div>

      {#if versionLoadFailed}
        <Alert>
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>Version unavailable</AlertTitle>
          <AlertDescription>
            The installed version could not be read, but you can still check
            for updates.
          </AlertDescription>
        </Alert>
      {/if}
    </CardContent>
  </Card>

  <!-- Update Check -->
  <Card>
    <CardHeader>
      <CardTitle>Check for Updates</CardTitle>
      <CardDescription>Manually check for available updates</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if checkError}
        <Alert variant={checkError.recoverable ? "default" : "destructive"}>
          {#if checkError.category === "network"}
            <WifiOff class="h-4 w-4" />
          {:else}
            <AlertCircle class="h-4 w-4" />
          {/if}
          <AlertTitle>{checkError.title}</AlertTitle>
          <AlertDescription>
            <div class="space-y-2">
              <p>{checkError.message}</p>
              {#if checkError.hint}
                <p class="text-sm text-muted-foreground">{checkError.hint}</p>
              {/if}
            </div>
          </AlertDescription>
        </Alert>
      {/if}

      {#if installError}
        <Alert variant={installError.recoverable ? "default" : "destructive"}>
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>{installError.title}</AlertTitle>
          <AlertDescription>
            <div class="space-y-2">
              <p>{installError.message}</p>
              {#if installError.hint}
                <p class="text-sm text-muted-foreground">{installError.hint}</p>
              {/if}
            </div>
          </AlertDescription>
        </Alert>
      {/if}

      {#if updateInfo}
        {#if updateInfo.available}
          <Alert>
            <Info class="h-4 w-4" />
            <AlertTitle>Update Available</AlertTitle>
            <AlertDescription>
              <div class="mt-2 space-y-2">
                <p>
                  <strong>Version {updateInfo.version}</strong> is available.
                </p>
                {#if updateInfo.date}
                  <p class="text-sm text-muted-foreground">
                    Released: {new Date(updateInfo.date).toLocaleDateString()}
                  </p>
                {/if}
                {#if updateInfo.body}
                  <div class="whitespace-pre-wrap text-sm">
                    {updateInfo.body}
                  </div>
                {/if}
              </div>
            </AlertDescription>
          </Alert>

          {#if isInstalling}
            <div class="space-y-2">
              <div
                class="flex items-center gap-2 text-sm text-muted-foreground"
              >
                <Loader2 class="h-4 w-4 animate-spin" />
                Installing update...
              </div>
              <Progress value={undefined} class="w-full" />
              <p class="text-xs text-muted-foreground">
                The application will restart automatically after installation.
              </p>
            </div>
          {:else}
            <Button onclick={handleInstallUpdate} class="w-full" size="lg">
              <Download class="mr-2 h-4 w-4" />
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
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          Checking for Updates...
        {:else}
          <RefreshCw class="mr-2 h-4 w-4" />
          Check for Updates
        {/if}
      </Button>
    </CardContent>
  </Card>
</div>
