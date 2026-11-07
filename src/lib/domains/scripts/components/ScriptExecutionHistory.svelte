<!--
	Recent executions for one script. Loads itself; bump `refreshKey` to reload.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import { scriptExecutionService, type ScriptExecutionInfo } from '../services';
  import { getStatusIcon, getStatusColor, formatDuration } from '../utils/executionStatus';

  interface Props {
    scriptId: string;
    /** Highlights the execution currently shown elsewhere on the page. */
    selectedId?: string | null;
    /** Changing this reloads the list — e.g. after a run finishes. */
    refreshKey?: number;
    onSelect?: (execution: ScriptExecutionInfo) => void;
  }

  let { scriptId, selectedId = null, refreshKey = 0, onSelect }: Props = $props();

  let executions = $state<ScriptExecutionInfo[]>([]);

  async function load() {
    executions = await scriptExecutionService.getExecutionsByBlock(scriptId, 10);
  }

  $effect(() => {
    void scriptId;
    void refreshKey;
    void load();
  });
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle>Execution History</CardTitle>
      <Button variant="ghost" size="sm" aria-label="Refresh history" onclick={() => load()}>
        <RefreshCw class="h-4 w-4" />
      </Button>
    </div>
  </CardHeader>
  <CardContent>
    {#if executions.length > 0}
      <div class="space-y-2">
        {#each executions as execution}
          {@const StatusIcon = getStatusIcon(execution.status)}
          <Button
            variant="outline"
            class="h-auto w-full justify-start rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 {selectedId ===
            execution.id
              ? 'bg-muted'
              : ''}"
            onclick={() => onSelect?.(execution)}
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <StatusIcon class="h-4 w-4 {getStatusColor(execution.status)}" />
                <span class="text-sm font-medium">{execution.status}</span>
              </div>
              <span class="text-xs text-muted-foreground">
                {new Date(execution.startedAt).toLocaleString()}
              </span>
            </div>
            {#if execution.finishedAt}
              <p class="mt-1 text-xs text-muted-foreground">
                Duration: {formatDuration(execution.startedAt, execution.finishedAt)}
              </p>
            {/if}
          </Button>
        {/each}
      </div>
    {:else}
      <p class="py-4 text-center text-muted-foreground">No executions yet</p>
    {/if}
  </CardContent>
</Card>
