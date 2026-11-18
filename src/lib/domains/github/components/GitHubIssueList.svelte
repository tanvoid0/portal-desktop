<!-- Open issues or pull requests as a list, with loading/error/empty states. -->
<script lang="ts">
  import { PageLoading } from '$lib/components/shell';
  import type { GitHubIssue } from '../types';
  import GitHubError from './GitHubError.svelte';
  import ExternalLink from '~icons/fa6-solid/arrow-up-right-from-square';

  interface Props {
    items: GitHubIssue[];
    isPending: boolean;
    error: unknown;
    /** What the list holds, lowercase plural: "issues" / "pull requests". */
    noun: string;
    onRetry: () => void;
  }

  let { items, isPending, error, noun, onRetry }: Props = $props();
</script>

{#if isPending}
  <PageLoading message="Loading {noun}..." />
{:else if error}
  <GitHubError
    title="Failed to load {noun}"
    message={error instanceof Error ? error.message : `Unable to load ${noun}`}
    {onRetry}
  />
{:else if items.length === 0}
  <p class="text-sm text-muted-foreground">No open {noun}.</p>
{:else}
  <div class="space-y-3">
    {#each items as item}
      <div class="rounded border p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="font-medium">
            #{item.number}
            {item.title}
          </div>
          <a
            href={item.htmlUrl}
            target="_blank"
            rel="noreferrer"
            class="inline-flex items-center text-xs text-primary"
          >
            Open
            <ExternalLink class="ml-1 h-3 w-3" />
          </a>
        </div>
        <div class="mt-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
          <span>{item.state}</span>
          {#if item.authorLogin}
            <span>by {item.authorLogin}</span>
          {/if}
          {#if item.labels.length > 0}
            <span>labels: {item.labels.join(', ')}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}
