import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import {
  queryClient,
  invalidateProjectsList,
  invalidateProjectDetail,
} from '$lib/domains/shared/query';
import {
  listProjectMoves,
  startProjectMove,
  cancelProjectMove,
  resolveInterruptedMove,
  removeMovedSource,
  type MoveSummary,
} from '../api/projectMoveApi';

/** The move journal. Polled: a move's progress/state can change with nothing else on
 *  screen triggering a refetch (the banner and the dialog both read off this one query). */
export function createProjectMovesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.projects.moves,
    queryFn: listProjectMoves,
    refetchInterval: 5000,
  }));
}

/** A committed move changes `project.path`, so every mutation below invalidates the moves
 *  list plus the projects list/detail — not just the move journal. */
function invalidateAfterMove(projectId?: number) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.projects.moves });
  invalidateProjectsList(queryClient);
  if (projectId !== undefined) invalidateProjectDetail(queryClient, projectId);
}

export async function startProjectMoveMutation(
  id: number,
  dest: string,
  includeRegenerable: boolean,
  acknowledgedWarnings: boolean
): Promise<MoveSummary> {
  const summary = await startProjectMove(id, dest, includeRegenerable, acknowledgedWarnings);
  invalidateAfterMove(id);
  return summary;
}

export async function cancelProjectMoveMutation(id: string): Promise<void> {
  await cancelProjectMove(id);
  invalidateAfterMove();
}

export async function resolveInterruptedMoveMutation(
  id: string,
  action: 'resume' | 'finish' | 'rollback'
): Promise<MoveSummary | null> {
  const summary = await resolveInterruptedMove(id, action);
  invalidateAfterMove(summary?.project_id);
  return summary;
}

export async function removeMovedSourceMutation(id: string): Promise<void> {
  await removeMovedSource(id);
  invalidateAfterMove();
}
