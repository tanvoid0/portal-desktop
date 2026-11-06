<!--
  Git status in plain language: branch, uncommitted work, unpushed commits, remote.
-->

<script lang="ts">
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import type { Project } from '$lib/domains/projects/types';
  import { getProjectGitBranch } from '$lib/domains/projects/utils/display';

  interface Props {
    project: Project;
    githubUrl: string | null;
  }

  let { project, githubUrl }: Props = $props();

  const branch = $derived(getProjectGitBranch(project));
  const unpushed = $derived(project.unpushed_commits ?? 0);
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <GitBranch class="h-5 w-5" />
      Git
    </CardTitle>
    <CardDescription>Is your work saved and backed up?</CardDescription>
  </CardHeader>
  <CardContent class="space-y-1.5 text-sm">
    {#if !branch && !project.git_repository}
      <p class="text-muted-foreground">This folder is not a git repository.</p>
    {:else}
      {#if branch}
        <p>On branch <span class="font-mono">{branch}</span></p>
      {/if}
      <p class={project.has_uncommitted_changes ? 'text-amber-600' : 'text-muted-foreground'}>
        {project.has_uncommitted_changes
          ? 'You have changes that are not committed.'
          : 'No uncommitted changes.'}
      </p>
      {#if unpushed > 0}
        <p class="text-amber-600">
          {unpushed}
          {unpushed === 1 ? 'commit' : 'commits'} not pushed
        </p>
      {/if}
      <!-- has_remote is only set by discovery; a hand-added repo may just have the URL. -->
      {#if !project.has_remote && !project.git_repository}
        <p class="text-muted-foreground">Not backed up to a remote yet.</p>
      {:else if githubUrl}
        <a
          href={githubUrl}
          target="_blank"
          rel="noreferrer"
          class="inline-flex items-center text-primary"
        >
          View on GitHub
          <ExternalLink class="ml-1 h-3 w-3" />
        </a>
      {:else if project.git_repository}
        <p class="font-mono text-xs break-all text-muted-foreground">{project.git_repository}</p>
      {/if}
    {/if}
  </CardContent>
</Card>
