<!--
	Shown app-wide when a project move was left mid-flight by a previous run (board #208,
	step 8 of #197). The backend re-arms the move lock for these at boot
	(`rearm_interrupted_moves`) but waits for an explicit user choice — this is that choice.
-->
<script lang="ts">
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Button } from '$lib/components/ui/button';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';
  import Loader2 from '~icons/fa6-solid/spinner';
  import {
    createProjectMovesQuery,
    resolveInterruptedMoveMutation,
  } from '../queries/projectMoveQueries';
  import { ACTIVE_MOVE_STATES, type MoveSummary } from '../api/projectMoveApi';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';

  const movesQuery = createProjectMovesQuery();
  // `running` excludes a move whose `start_project_move`/resume call is actively awaiting
  // right now — that's a live move, not one a crash left for the user to resolve.
  const interrupted = $derived(
    (movesQuery.data ?? []).filter((m) => ACTIVE_MOVE_STATES.has(m.state) && !m.running)
  );

  let busyId = $state<string | null>(null);

  /**
   * Judgment call: the backend already computes exactly this (`mover::recovery_options`),
   * but it reads live filesystem facts (is the destination volume present, is the staging
   * copy still there) that aren't exposed through a command — only `state` is, via
   * `MoveSummary`. This mirrors `recovery_options`'s state-only branches and lets a refused
   * action (e.g. resume on an offline drive) fail with the backend's own error message
   * instead of guessing it client-side. See board #208 report for the full seam.
   */
  function actionsFor(state: string): Array<'resume' | 'finish' | 'rollback'> {
    if (state === 'verified') return ['finish', 'rollback'];
    return ['resume', 'rollback']; // copying | verifying
  }

  function label(path: string): string {
    return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
  }

  async function act(move: MoveSummary, action: 'resume' | 'finish' | 'rollback') {
    if (action === 'rollback') {
      const ok = await confirmAction(
        `Abandon the move of "${label(move.source_path)}"? The copy in progress is deleted; the original folder is untouched.`,
        'Roll back move',
        { destructive: true, confirmLabel: 'Roll back' }
      );
      if (!ok) return;
    }
    busyId = move.id;
    try {
      await resolveInterruptedMoveMutation(move.id, action);
      toast.success(action === 'rollback' ? 'Move rolled back' : 'Move finished');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      busyId = null;
    }
  }
</script>

{#if interrupted.length > 0}
  <Alert variant="destructive" class="rounded-none border-x-0 border-t-0">
    <TriangleExclamation class="h-4 w-4" />
    <div class="flex flex-col gap-2">
      <AlertTitle>
        {interrupted.length === 1
          ? 'A project move was interrupted'
          : `${interrupted.length} project moves were interrupted`}
      </AlertTitle>
      <AlertDescription>
        <ul class="space-y-2">
          {#each interrupted as move (move.id)}
            <li class="flex flex-wrap items-center gap-2">
              <span class="font-medium text-foreground">{label(move.source_path)}</span>
              <span class="text-muted-foreground">→ {label(move.dest_path)} · {move.state}</span>
              <div class="ml-auto flex gap-1.5">
                {#each actionsFor(move.state) as action (action)}
                  <Button
                    size="sm"
                    variant={action === 'rollback' ? 'outline' : 'default'}
                    disabled={busyId === move.id}
                    onclick={() => act(move, action)}
                  >
                    {#if busyId === move.id}
                      <Loader2 class="h-3.5 w-3.5 animate-spin" />
                    {/if}
                    {action === 'resume' ? 'Resume' : action === 'finish' ? 'Finish' : 'Roll back'}
                  </Button>
                {/each}
              </div>
            </li>
          {/each}
        </ul>
      </AlertDescription>
    </div>
  </Alert>
{/if}
