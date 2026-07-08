<script lang="ts">
  import { goto } from "$app/navigation";
  import { get } from "svelte/store";
  import { settings, settingsActions } from "$lib/domains/settings/stores/settingsStore";
  import type { AppSettings } from "$lib/domains/settings/types";
  import { createGitHubStatusQuery, githubService } from "$lib/domains/github";
  import type { GitHubDeviceFlowStart } from "$lib/domains/github";
  import { openExternalUrl } from "$lib/utils/tauri";
  import { toast } from "$lib/utils/toast";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { PageLoading, PageError } from "$lib/components/shell";
  import { FolderGit2, Unplug, ExternalLink } from "@lucide/svelte";

  const settingsData = $derived($settings);
  const statusQuery = createGitHubStatusQuery();

  let clientId = $state("");
  let saving = $state(false);
  let connecting = $state(false);
  let disconnecting = $state(false);
  let deviceFlow = $state<GitHubDeviceFlowStart | null>(null);
  let flowMessage = $state<string | null>(null);

  $effect(() => {
    clientId = settingsData?.app.integrations?.github?.clientId ?? "";
  });

  function updateAppSettings(updates: Partial<AppSettings>) {
    const current = get(settings);
    if (!current) return;
    settings.set({
      ...current,
      app: {
        ...current.app,
        ...updates,
      },
    });
  }

  async function handleSaveClientId() {
    const current = get(settings);
    if (!current) return;

    try {
      saving = true;
      const nextSettings = {
        ...current,
        app: {
          ...current.app,
          integrations: {
            github: {
              clientId: clientId.trim(),
            },
          },
        },
      };
      await settingsActions.saveSettings(nextSettings);
      await statusQuery.refetch();
      toast.success("GitHub settings saved");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to save GitHub settings",
      );
    } finally {
      saving = false;
    }
  }

  async function handleConnect() {
    try {
      if (!statusQuery.data?.clientIdConfigured) {
        toast.error("Save a GitHub Client ID first");
        return;
      }
      connecting = true;
      deviceFlow = null;
      flowMessage = null;
      await githubService.connectWithDeviceFlow(undefined, {
        onStarted: (start) => {
          deviceFlow = start;
          flowMessage =
            "Authorize GitHub in your browser. If nothing opened, use the code below.";
        },
        onPolling: () => {
          flowMessage = "Waiting for you to authorize on GitHub...";
        },
      });
      await statusQuery.refetch();
      toast.success("GitHub connected");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to connect GitHub",
      );
    } finally {
      connecting = false;
      deviceFlow = null;
      flowMessage = null;
    }
  }

  async function handleOpenGitHub() {
    if (!deviceFlow) return;
    const target =
      deviceFlow.verificationUriComplete || deviceFlow.verificationUri;
    try {
      await openExternalUrl(target);
    } catch (error) {
      toast.error(
        error instanceof Error
          ? error.message
          : "Failed to open GitHub authorization page",
      );
    }
  }

  async function handleDisconnect() {
    try {
      disconnecting = true;
      await githubService.disconnect();
      await statusQuery.refetch();
      toast.success("GitHub disconnected");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to disconnect GitHub",
      );
    } finally {
      disconnecting = false;
    }
  }
</script>

<svelte:head>
  <title>GitHub Settings - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight">GitHub Settings</h2>
    <p class="text-muted-foreground">
      Configure the GitHub OAuth app and connect your account for repositories and issues.
    </p>
  </div>

  <Card>
    <CardHeader>
      <CardTitle>OAuth Configuration</CardTitle>
      <CardDescription>
        Enter the GitHub OAuth App client ID used for Device Flow authentication.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <Label for="github-client-id">GitHub Client ID</Label>
        <Input
          id="github-client-id"
          bind:value={clientId}
          placeholder="Iv1.1234567890abcdef"
          onchange={() =>
            updateAppSettings({
              integrations: {
                github: {
                  clientId,
                },
              },
            })}
        />
        <p class="text-sm text-muted-foreground">
          Create a GitHub OAuth App with Device Flow enabled, then paste its client ID here.
        </p>
      </div>

      <div class="flex flex-wrap gap-2">
        <Button onclick={handleSaveClientId} disabled={saving}>
          {saving ? "Saving..." : "Save GitHub Settings"}
        </Button>
        <Button
          variant="outline"
          onclick={() => goto("/github")}
        >
          Open GitHub Workspace
        </Button>
        <a
          href="https://github.com/settings/developers"
          target="_blank"
          rel="noreferrer"
          class="inline-flex items-center text-sm text-primary"
        >
          Open GitHub Developer Settings
          <ExternalLink class="ml-1 h-4 w-4" />
        </a>
      </div>
    </CardContent>
  </Card>

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if statusQuery.isError}
    <PageError
      title="GitHub unavailable"
      message={statusQuery.error instanceof Error
        ? statusQuery.error.message
        : "Failed to load GitHub status"}
      onRetry={() => statusQuery.refetch()}
    />
  {:else}
    <Card>
      <CardHeader>
        <CardTitle>Connection Status</CardTitle>
        <CardDescription>
          Connect or disconnect the GitHub account used by this desktop app.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="flex flex-wrap gap-2">
          <Badge variant={statusQuery.data?.clientIdConfigured ? "secondary" : "outline"}>
            {statusQuery.data?.clientIdConfigured
              ? "Client ID configured"
              : "Client ID missing"}
          </Badge>
          <Badge variant={statusQuery.data?.connected ? "secondary" : "outline"}>
            {statusQuery.data?.connected ? "Account connected" : "Not connected"}
          </Badge>
        </div>

        {#if statusQuery.data?.account}
          <div class="rounded-lg border p-4">
            <div class="font-medium">{statusQuery.data.account.login}</div>
            {#if statusQuery.data.account.name}
              <div class="text-sm text-muted-foreground">
                {statusQuery.data.account.name}
              </div>
            {/if}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each statusQuery.data.account.scopes as scope}
                <Badge variant="outline">{scope}</Badge>
              {/each}
            </div>
          </div>
        {:else}
          <p class="text-sm text-muted-foreground">
            Save a client ID, then connect GitHub to enable repository browsing, cloning, and issue management.
          </p>
        {/if}

        <div class="flex flex-wrap gap-2">
          {#if statusQuery.data?.connected}
            <Button
              variant="outline"
              onclick={handleDisconnect}
              disabled={disconnecting}
            >
              <Unplug class="mr-2 h-4 w-4" />
              {disconnecting ? "Disconnecting..." : "Disconnect GitHub"}
            </Button>
          {:else if deviceFlow}
            <div class="w-full space-y-4 rounded-lg border bg-muted/30 p-4">
              <p class="text-sm text-muted-foreground">{flowMessage}</p>
              <div class="rounded-lg border bg-background px-4 py-3 text-center">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">
                  Your code
                </div>
                <div class="mt-1 font-mono text-2xl font-semibold tracking-widest">
                  {deviceFlow.userCode}
                </div>
              </div>
              <div class="flex flex-wrap gap-2">
                <Button onclick={handleOpenGitHub}>Open GitHub</Button>
                <Button variant="outline" disabled>Connecting...</Button>
              </div>
            </div>
          {:else}
            <Button
              onclick={handleConnect}
              disabled={connecting || !statusQuery.data?.clientIdConfigured}
            >
              <FolderGit2 class="mr-2 h-4 w-4" />
              {connecting ? "Connecting..." : "Connect GitHub"}
            </Button>
          {/if}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
