<script lang="ts">
  import { goto } from "$app/navigation";
  import { PageEmpty } from "$lib/components/shell";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { githubService } from "$lib/domains/github";
  import type {
    GitHubConnectionStatus,
    GitHubDeviceFlowStart,
  } from "$lib/domains/github";
  import { openExternalUrl } from "$lib/utils/tauri";
  import { toast } from "$lib/utils/toast";
  import { FolderGit2 } from "@lucide/svelte";

  interface Props {
    status?: GitHubConnectionStatus | null;
    onConnected?: () => void | Promise<void>;
  }

  let { status = null, onConnected }: Props = $props();

  let connecting = $state(false);
  let deviceFlow = $state<GitHubDeviceFlowStart | null>(null);
  let flowMessage = $state<string | null>(null);

  const description = $derived(
    status?.clientIdConfigured
      ? "Connect your GitHub account to browse repositories, clone projects, and manage issues."
      : "GitHub OAuth is not configured yet. Add your GitHub Client ID in Settings > GitHub, then connect your account.",
  );

  async function handleConnect() {
    if (!status?.clientIdConfigured) {
      goto("/settings/github");
      return;
    }

    try {
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
      toast.success("GitHub connected");
      await onConnected?.();
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
</script>

{#if deviceFlow}
  <Card>
    <CardContent class="flex flex-col items-center justify-center py-12">
      <FolderGit2 class="mb-4 h-12 w-12 text-muted-foreground" />
      <h3 class="mb-2 text-lg font-semibold">Authorize GitHub</h3>
      <p class="mb-4 max-w-md text-center text-muted-foreground">
        {flowMessage}
      </p>
      <div class="mb-4 rounded-lg border bg-muted/40 px-6 py-4 text-center">
        <div class="text-xs uppercase tracking-wide text-muted-foreground">
          Your code
        </div>
        <div class="mt-1 font-mono text-3xl font-semibold tracking-widest">
          {deviceFlow.userCode}
        </div>
      </div>
      <div class="flex flex-wrap justify-center gap-2">
        <Button onclick={handleOpenGitHub}>Open GitHub</Button>
        <Button variant="outline" disabled>Connecting...</Button>
      </div>
    </CardContent>
  </Card>
{:else}
  <PageEmpty
    title="Connect GitHub"
    {description}
    icon={FolderGit2}
    actionLabel={status?.clientIdConfigured
      ? connecting
        ? "Connecting..."
        : "Connect GitHub"
      : "Open GitHub Settings"}
    onAction={handleConnect}
  />
{/if}
