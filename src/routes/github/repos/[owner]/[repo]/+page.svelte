<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import FolderPicker from "$lib/components/ui/folder-picker.svelte";
  import { Input } from "$lib/components/ui/input";
  import { PageHeader, PageLoading, PageError, PageEmpty } from "$lib/components/shell";
  import {
    createGitHubIssuesQuery,
    createGitHubRepositoryQuery,
    createGitHubStatusQuery,
    githubService,
    GitHubConnectPrompt,
  } from "$lib/domains/github";
  import { toast } from "$lib/utils/toast";
  import { ExternalLink, FolderGit2, Link2, Bug } from "@lucide/svelte";

  const owner = $derived($page.params.owner);
  const repo = $derived($page.params.repo);

  const statusQuery = createGitHubStatusQuery();
  const isConnected = $derived(statusQuery.data?.connected ?? false);
  const repoQuery = createGitHubRepositoryQuery(
    () => owner,
    () => repo,
    () => isConnected,
  );
  const repoIssuesQuery = createGitHubIssuesQuery(
    () => ({
      owner,
      repo,
      state: "open",
      page: 1,
      perPage: 20,
    }),
    () => isConnected,
  );

  let cloneDialogOpen = $state(false);
  let linkDialogOpen = $state(false);
  let clonePath = $state("");
  let linkPath = $state("");
  let cloning = $state(false);
  let linking = $state(false);

  $effect(() => {
    if (repoQuery.data?.repository && !clonePath) {
      clonePath = `${repoQuery.data.repository.name}`;
    }
  });

  async function handleClone() {
    if (!owner || !repo || !clonePath.trim()) return;
    try {
      cloning = true;
      const result = await githubService.cloneRepository({
        owner,
        repo,
        destinationPath: clonePath,
      });
      toast.success(`Cloned into ${result.localPath}`);
      cloneDialogOpen = false;
      await repoQuery.refetch();
      goto(`/projects/${result.project.id}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Clone failed");
    } finally {
      cloning = false;
    }
  }

  async function handleLink() {
    if (!linkPath.trim()) return;
    try {
      linking = true;
      const result = await githubService.linkExistingRepository({
        path: linkPath,
        owner,
        repo,
      });
      toast.success(`Linked ${result.project.name}`);
      linkDialogOpen = false;
      await repoQuery.refetch();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Link failed");
    } finally {
      linking = false;
    }
  }
</script>

<svelte:head>
  <title>{owner}/{repo} - GitHub - Portal Desktop</title>
</svelte:head>

<div class="space-y-6">
  <PageHeader
    title={`${owner}/${repo}`}
    description="Repository details, linked projects, and issue overview"
  >
    {#snippet actions()}
      <Button variant="outline" onclick={() => (linkDialogOpen = true)}>
        <Link2 class="mr-2 h-4 w-4" />
        Link Existing Repo
      </Button>
      <Button onclick={() => (cloneDialogOpen = true)}>
        <FolderGit2 class="mr-2 h-4 w-4" />
        Clone as Project
      </Button>
    {/snippet}
  </PageHeader>

  {#if statusQuery.isPending}
    <PageLoading message="Checking GitHub connection..." />
  {:else if !statusQuery.data?.connected}
    <GitHubConnectPrompt
      status={statusQuery.data}
      onConnected={() => statusQuery.refetch()}
    />
  {:else if repoQuery.isPending}
    <PageLoading message="Loading repository..." />
  {:else if repoQuery.isError}
    <PageError
      title="Failed to load repository"
      message={repoQuery.error instanceof Error
        ? repoQuery.error.message
        : "Unable to load repository"}
      onRetry={() => repoQuery.refetch()}
    />
  {:else if !repoQuery.data}
    <PageEmpty
      title="Repository not found"
      description="The requested repository could not be loaded."
      icon={FolderGit2}
    />
  {:else}
    {@const repoData = repoQuery.data.repository}
    <div class="grid gap-4 lg:grid-cols-[2fr,1fr]">
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center justify-between gap-3">
            <span>{repoData.fullName}</span>
            <a
              href={repoData.htmlUrl}
              target="_blank"
              rel="noreferrer"
              class="inline-flex items-center text-sm text-primary"
            >
              Open on GitHub
              <ExternalLink class="ml-1 h-4 w-4" />
            </a>
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <p class="text-sm text-muted-foreground">
            {repoData.description || "No description"}
          </p>
          <div class="flex flex-wrap gap-2">
            <Badge variant="outline">Branch: {repoData.defaultBranch}</Badge>
            <Badge variant="outline">Issues: {repoData.openIssuesCount}</Badge>
            {#if repoData.language}
              <Badge variant="outline">Language: {repoData.language}</Badge>
            {/if}
            <Badge variant={repoData.private ? "secondary" : "outline"}>
              {repoData.private ? "Private" : "Public"}
            </Badge>
          </div>
          <div class="grid gap-2 text-sm">
            <div><span class="font-medium">Clone URL:</span> {repoData.cloneUrl}</div>
            {#if repoData.sshUrl}
              <div><span class="font-medium">SSH URL:</span> {repoData.sshUrl}</div>
            {/if}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Linked Projects</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3">
          {#if repoQuery.data.linkedProjects.length === 0}
            <p class="text-sm text-muted-foreground">
              No local projects linked yet.
            </p>
          {:else}
            {#each repoQuery.data.linkedProjects as project}
              <button
                class="w-full rounded border p-3 text-left transition-colors hover:border-primary/50"
                onclick={() => goto(`/projects/${project.id}`)}
              >
                <div class="font-medium">{project.name}</div>
                <div class="text-xs text-muted-foreground">{project.path}</div>
              </button>
            {/each}
          {/if}
        </CardContent>
      </Card>
    </div>

    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Bug class="h-4 w-4" />
          Open Issues
        </CardTitle>
      </CardHeader>
      <CardContent>
        {#if repoIssuesQuery.isPending}
          <PageLoading message="Loading issues..." />
        {:else if repoIssuesQuery.isError}
          <PageError
            title="Failed to load issues"
            message={repoIssuesQuery.error instanceof Error
              ? repoIssuesQuery.error.message
              : "Unable to load issues"}
            onRetry={() => repoIssuesQuery.refetch()}
          />
        {:else if (repoIssuesQuery.data ?? []).length === 0}
          <p class="text-sm text-muted-foreground">No open issues.</p>
        {:else}
          <div class="space-y-3">
            {#each repoIssuesQuery.data ?? [] as issue}
              <div class="rounded border p-3">
                <div class="flex items-center justify-between gap-3">
                  <div class="font-medium">
                    #{issue.number} {issue.title}
                  </div>
                  <a
                    href={issue.htmlUrl}
                    target="_blank"
                    rel="noreferrer"
                    class="inline-flex items-center text-xs text-primary"
                  >
                    Open
                    <ExternalLink class="ml-1 h-3 w-3" />
                  </a>
                </div>
                <div class="mt-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
                  <span>{issue.state}</span>
                  {#if issue.authorLogin}
                    <span>by {issue.authorLogin}</span>
                  {/if}
                  {#if issue.labels.length > 0}
                    <span>labels: {issue.labels.join(", ")}</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </CardContent>
    </Card>
  {/if}

  <Dialog.Root open={cloneDialogOpen} onOpenChange={(open) => (cloneDialogOpen = open)}>
    <Dialog.Content class="max-w-xl">
      <Dialog.Header>
        <Dialog.Title>Clone Repository</Dialog.Title>
      </Dialog.Header>
      <div class="space-y-4">
        <FolderPicker
          bind:value={clonePath}
          label="Destination Path"
          description="Select the final local folder for the cloned repository."
        />
        <Button onclick={handleClone} disabled={cloning || !clonePath.trim()}>
          Clone
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Root>

  <Dialog.Root open={linkDialogOpen} onOpenChange={(open) => (linkDialogOpen = open)}>
    <Dialog.Content class="max-w-xl">
      <Dialog.Header>
        <Dialog.Title>Link Existing Local Repository</Dialog.Title>
      </Dialog.Header>
      <div class="space-y-4">
        <FolderPicker
          bind:value={linkPath}
          label="Repository Path"
          description="Choose an existing local git repository to link to this GitHub repo."
        />
        <Input bind:value={linkPath} placeholder="D:\\dev\\my-repo" />
        <Button onclick={handleLink} disabled={linking || !linkPath.trim()}>
          Link Repository
        </Button>
      </div>
    </Dialog.Content>
  </Dialog.Root>
</div>
