<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { PageHeader, PageLoading } from '$lib/components/shell';
  import {
    createGitHubStatusQuery,
    createGitHubTokenCheckQuery,
    GitHubError,
    githubService,
  } from '$lib/domains/github';
  import type { GitHubDeviceFlowStart } from '$lib/domains/github';
  import { isTokenUnreadableError } from '$lib/domains/github/utils/tokenError';
  import { openExternalUrl } from '$lib/utils/tauri';
  import { toast } from '$lib/utils/toast';
  import FolderGit2 from '~icons/fa6-solid/folder-tree';
  import Bug from '~icons/fa6-solid/bug';
  import Unplug from '~icons/fa6-solid/plug-circle-xmark';

  const statusQuery = createGitHubStatusQuery();
  const status = $derived(statusQuery.data);
  const tokenCheck = createGitHubTokenCheckQuery(() => status?.connected ?? false);
  const tokenUnreadable = $derived(isTokenUnreadableError(tokenCheck.error));

  let connecting = $state(false);
  let disconnecting = $state(false);
  let deviceFlow = $state<GitHubDeviceFlowStart | null>(null);
  let flowMessage = $state<string | null>(null);

  async function handleConnect() {
    try {
      if (!status?.clientIdConfigured) {
        goto('/settings/github');
        return;
      }
      connecting = true;
      deviceFlow = null;
      flowMessage = null;
      await githubService.connectWithDeviceFlow(undefined, {
        onStarted: (start) => {
          deviceFlow = start;
          flowMessage = 'Authorize GitHub in your browser. If nothing opened, use the code below.';
        },
        onPolling: () => {
          flowMessage = 'Waiting for you to authorize on GitHub...';
        },
      });
      toast.success('GitHub connected');
      await statusQuery.refetch();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to connect GitHub';
      toast.error(message);
    } finally {
      connecting = false;
      deviceFlow = null;
      flowMessage = null;
    }
  }

  async function handleOpenGitHub() {
    if (!deviceFlow) return;
    const target = deviceFlow.verificationUriComplete || deviceFlow.verificationUri;
    try {
      await openExternalUrl(target);
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to open GitHub authorization page'
      );
    }
  }

  async function handleDisconnect() {
    try {
      disconnecting = true;
      await githubService.disconnect();
      toast.success('GitHub disconnected');
      await statusQuery.refetch();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to disconnect GitHub';
      toast.error(message);
    } finally {
      disconnecting = false;
    }
  }
</script>

<svelte:head>
  <title>GitHub - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader title="GitHub" description="Connect GitHub, browse repositories, and manage issues">
    {#snippet actions()}
      {#if status?.connected}
        <Button variant="outline" onclick={handleDisconnect} disabled={disconnecting}>
          <Unplug class="mr-2 h-4 w-4" />
          Disconnect
        </Button>
      {:else}
        <Button onclick={handleConnect} disabled={connecting}>
          <FolderGit2 class="mr-2 h-4 w-4" />
          Connect GitHub
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if statusQuery.isError}
    <GitHubError
      title="GitHub unavailable"
      message={statusQuery.error instanceof Error
        ? statusQuery.error.message
        : 'Failed to load GitHub status'}
      onRetry={() => statusQuery.refetch()}
    />
  {:else}
    <div class="grid gap-4 md:grid-cols-3">
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <FolderGit2 class="h-4 w-4" />
            Account
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-2">
          {#if status?.connected && status.account}
            <div class="font-medium">{status.account.login}</div>
            {#if status.account.name}
              <div class="text-sm text-muted-foreground">{status.account.name}</div>
            {/if}
            <div class="flex flex-wrap gap-2 pt-1">
              {#if tokenUnreadable}
                <Badge variant="destructive">Needs reconnect</Badge>
              {:else}
                <Badge variant="secondary">Connected</Badge>
              {/if}
              {#each status.account.scopes as scope}
                <Badge variant="outline">{scope}</Badge>
              {/each}
            </div>
            {#if tokenUnreadable}
              <p class="pt-1 text-sm text-muted-foreground">
                Stored GitHub token can't be read. Reconnect GitHub to continue.
              </p>
              <Button size="sm" onclick={() => goto('/settings/github')}>Reconnect GitHub</Button>
            {/if}
          {:else if status?.clientIdConfigured}
            <p class="text-sm text-muted-foreground">No GitHub account connected yet.</p>
          {:else}
            <p class="text-sm text-muted-foreground">
              GitHub OAuth is not configured yet. Add a client ID in Settings > GitHub.
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card class="cursor-pointer" onclick={() => goto('/github/repos')}>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <FolderGit2 class="h-4 w-4" />
            Repositories
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-sm text-muted-foreground">
            Browse, clone, and link GitHub repositories into Projects.
          </p>
        </CardContent>
      </Card>

      <Card class="cursor-pointer" onclick={() => goto('/github/issues')}>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Bug class="h-4 w-4" />
            Issues
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p class="text-sm text-muted-foreground">
            View your assigned issues and create or update repository issues.
          </p>
        </CardContent>
      </Card>
    </div>
  {/if}
</div>
