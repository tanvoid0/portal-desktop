<!--
  Git status in plain language: branch, uncommitted work, unpushed commits, remote.
-->

<script lang="ts">
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';
  import GitBranch from '~icons/fa6-solid/code-branch';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import type { Project } from '$lib/domains/projects/types';
  import { getProjectGitBranch } from '$lib/domains/projects/utils/display';

  interface Props {
    project: Project;
    githubUrl: string | null;
    refreshing?: boolean;
    onRedetect?: () => void;
    onTerminal?: () => void;
    onSetPushOk?: (pushOk: boolean) => void;
  }

  let {
    project,
    githubUrl,
    refreshing = false,
    onRedetect,
    onTerminal,
    onSetPushOk,
  }: Props = $props();

  const branch = $derived(getProjectGitBranch(project));
  const unpushed = $derived(project.unpushed_commits ?? 0);
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <GitBranch class="h-5 w-5" />
      Git
    </CardTitle>
  </CardHeader>
  <CardContent class="space-y-1.5 text-sm">
    {#if !branch && !project.git_repository}
      <p class="text-muted-foreground">
        This folder is not a git repository — or the details are out of date.
        {#if onRedetect}
          <Button
            variant="link"
            size="sm"
            class="h-auto p-0"
            onclick={onRedetect}
            disabled={refreshing}
          >
            {refreshing ? 'Re-detecting…' : 'Re-detect'}
          </Button>
        {/if}
      </p>
    {:else}
      {#if branch}
        <p>On branch <span class="font-mono">{branch}</span></p>
      {/if}
      <div class="flex flex-wrap gap-1.5">
        {#if project.has_uncommitted_changes}
          <Badge variant="secondary">Uncommitted changes</Badge>
        {/if}
        {#if unpushed > 0}
          <Badge variant="secondary">{unpushed} unpushed</Badge>
        {/if}
        {#if !project.has_uncommitted_changes && unpushed === 0}
          <Badge variant="outline">Clean</Badge>
        {/if}
      </div>
      {#if onTerminal && (project.has_uncommitted_changes || unpushed > 0)}
        <Button variant="outline" size="sm" class="gap-2" onclick={onTerminal}>
          Open terminal to commit / push
        </Button>
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
    {#if onSetPushOk}
      <div class="flex items-center justify-between gap-3 border-t pt-3">
        <div class="space-y-0.5">
          <Label for="push-ok">Allow push to remote</Label>
          <p class="text-xs text-muted-foreground">Sync all only pushes repos you've cleared</p>
        </div>
        <Switch id="push-ok" checked={project.push_ok} onCheckedChange={onSetPushOk} />
      </div>
    {/if}
  </CardContent>
</Card>
