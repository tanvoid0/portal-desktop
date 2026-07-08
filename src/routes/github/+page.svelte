<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { PageHeader, PageLoading, PageError } from "$lib/components/shell";
  import { createGitHubStatusQuery, githubService } from "$lib/domains/github";
  import { toast } from "$lib/utils/toast";
  import { FolderGit2, Bug, Unplug } from "@lucide/svelte";

  const statusQuery = createGitHubStatusQuery();
  const status = $derived(statusQuery.data);

  let connecting = $state(false);
  let disconnecting = $state(false);

  async function handleConnect() {
    try {
      if (!status?.clientIdConfigured) {
        goto("/settings/github");
        return;
      }
      connecting = true;
      await githubService.connectWithDeviceFlow();
      toast.success("GitHub connected");
      await statusQuery.refetch();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to connect GitHub";
      toast.error(message);
    } finally {
      connecting = false;
    }
  }

  async function handleDisconnect() {
    try {
      disconnecting = true;
      await githubService.disconnect();
      toast.success("GitHub disconnected");
      await statusQuery.refetch();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to disconnect GitHub";
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
  <PageHeader
    title="GitHub"
    description="Connect GitHub, browse repositories, and manage issues"
  >
    {#snippet actions()}
      {#if status?.connected}
        <Button
          variant="outline"
          onclick={handleDisconnect}
          disabled={disconnecting}
        >
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
    <PageError
      title="GitHub unavailable"
      message={statusQuery.error instanceof Error
        ? statusQuery.error.message
        : "Failed to load GitHub status"}
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
              <Badge variant="secondary">Connected</Badge>
              {#each status.account.scopes as scope}
                <Badge variant="outline">{scope}</Badge>
              {/each}
            </div>
          {:else if status?.clientIdConfigured}
            <p class="text-sm text-muted-foreground">
              No GitHub account connected yet.
            </p>
          {:else}
            <p class="text-sm text-muted-foreground">
              GitHub OAuth is not configured yet. Add a client ID in Settings > GitHub.
            </p>
          {/if}
        </CardContent>
      </Card>

      <Card class="cursor-pointer" onclick={() => goto("/github/repos")}>
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

      <Card class="cursor-pointer" onclick={() => goto("/github/issues")}>
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
