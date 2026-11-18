<script lang="ts">
  import { fmtBytes } from '$lib/domains/disk/utils';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import AuditStatusBadge from './AuditStatusBadge.svelte';

  interface Props {
    project: DiscoveredProject | null;
    /** Paths of the other members of this project's duplicate clusters. */
    siblings: string[];
  }
  let { project: focused, siblings }: Props = $props();
</script>

<section class="space-y-2 border-t pt-3">
  <h3 class="text-xs font-semibold text-muted-foreground uppercase">Selected project</h3>
  {#if focused}
    <div class="flex flex-wrap items-center gap-2">
      <span class="text-sm font-medium">{focused.name}</span>
      <AuditStatusBadge status={focused.status} ageDays={focused.ageDays} />
    </div>
    <p class="font-mono text-xs break-all text-muted-foreground">{focused.abs}</p>
    <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 text-xs">
      <dt class="text-muted-foreground">Reclaimable</dt>
      <dd>{focused.regenerableBytes < 0 ? 'not measured' : fmtBytes(focused.regenerableBytes)}</dd>
      <dt class="text-muted-foreground">Last commit</dt>
      <dd>
        {focused.git.lastCommit
          ? new Date(focused.git.lastCommit).toLocaleDateString()
          : focused.git.repos
            ? 'no commits'
            : 'not a repo'}
      </dd>
      <dt class="text-muted-foreground">Unpushed</dt>
      <dd>{focused.git.ahead > 0 ? `${focused.git.ahead} commits` : 'none'}</dd>
      <dt class="text-muted-foreground">Remote</dt>
      <dd class="font-mono break-all">
        {focused.git.remote ?? (focused.git.repos ? 'none — local only' : '—')}
      </dd>
      <dt class="text-muted-foreground">Duplicates</dt>
      <dd>
        {#each siblings as sibling (sibling)}
          <span class="block truncate font-mono" title={sibling}>{sibling}</span>
        {:else}
          none
        {/each}
      </dd>
    </dl>
    {#if focused.risk.length}
      <p class="text-xs">
        <span class="text-muted-foreground">Flags:</span>
        {focused.risk.join(' · ')}
      </p>
    {/if}
  {:else}
    <p class="text-xs text-muted-foreground">
      Nothing under the cursor — press <span class="font-mono">j</span> to start down the list.
    </p>
  {/if}
</section>
