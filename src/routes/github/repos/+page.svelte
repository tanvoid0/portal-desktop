<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { PageHeader, PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import {
    createGitHubRepositoriesQuery,
    createGitHubStatusQuery,
    GitHubConnectPrompt,
  } from "$lib/domains/github";
  import { FolderGit2, Lock, Globe, ExternalLink } from "@lucide/svelte";

  let search = $state("");

  const statusQuery = createGitHubStatusQuery();
  const isConnected = $derived(statusQuery.data?.connected ?? false);
  const reposQuery = createGitHubRepositoriesQuery(
    () => search,
    () => isConnected,
  );
  const repositories = $derived(reposQuery.data ?? []);
</script>

<svelte:head>
  <title>GitHub Repositories - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title="Repositories"
    description="Browse GitHub repositories and link them to local projects"
  />

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if !statusQuery.data?.connected}
    <GitHubConnectPrompt
      status={statusQuery.data}
      onConnected={() => statusQuery.refetch()}
    />
  {:else}
    <div class="flex items-center gap-3">
      <Input
        bind:value={search}
        placeholder="Search repositories..."
        class="max-w-md"
      />
      <Button variant="outline" onclick={() => reposQuery.refetch()}>
        Refresh
      </Button>
    </div>

    {#if reposQuery.isPending}
      <PageLoading message="Loading repositories..." />
    {:else if reposQuery.isError}
      <PageError
        title="Failed to load repositories"
        message={reposQuery.error instanceof Error
          ? reposQuery.error.message
          : "Unable to load repositories"}
        onRetry={() => reposQuery.refetch()}
      />
    {:else if repositories.length === 0}
      <PageEmpty
        title="No repositories found"
        description="Try a different search query or refresh your GitHub connection."
        icon={FolderGit2}
      />
    {:else}
      <div class="grid gap-4 lg:grid-cols-2">
        {#each repositories as repo}
          <Card
            class="cursor-pointer transition-colors hover:border-primary/50"
            onclick={() => goto(`/github/repos/${repo.owner.login}/${repo.name}`)}
          >
            <CardHeader>
              <CardTitle class="flex items-center justify-between gap-3">
                <span class="truncate">{repo.fullName}</span>
                <div class="flex items-center gap-2">
                  <Badge variant={repo.private ? "secondary" : "outline"}>
                    {#if repo.private}
                      <Lock class="mr-1 h-3 w-3" />
                      Private
                    {:else}
                      <Globe class="mr-1 h-3 w-3" />
                      Public
                    {/if}
                  </Badge>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <p class="min-h-10 text-sm text-muted-foreground">
                {repo.description || "No description"}
              </p>
              <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
                <span>Branch: {repo.defaultBranch}</span>
                {#if repo.language}
                  <span>Language: {repo.language}</span>
                {/if}
                <span>Issues: {repo.openIssuesCount}</span>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-xs text-muted-foreground">
                  Updated {repo.updatedAt ? new Date(repo.updatedAt).toLocaleString() : "recently"}
                </div>
                <a
                  href={repo.htmlUrl}
                  target="_blank"
                  rel="noreferrer"
                  class="inline-flex items-center text-xs text-primary"
                  onclick={(event) => event.stopPropagation()}
                >
                  Open on GitHub
                  <ExternalLink class="ml-1 h-3 w-3" />
                </a>
              </div>
            </CardContent>
          </Card>
        {/each}
      </div>
    {/if}
  {/if}
</div>
