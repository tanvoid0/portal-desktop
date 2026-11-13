import { invokeClient } from '$lib/utils/invokeClient';
import type { MoveSummary } from '$lib/domains/projects/generated/MoveSummary';
import type { PreflightResult } from '$lib/domains/projects/generated/PreflightResult';

/** `project-move://progress` payload. Not ts-rs generated — `mover::Progress` is emitted ad
 *  hoc on the event sink, not returned from a command — so this mirrors its fields by hand. */
export interface MoveProgress {
  id: string;
  phase: 'copying' | 'verifying';
  files_done: number;
  file_count: number;
  bytes_done: number;
  total_bytes: number;
}

export const MOVE_PROGRESS_EVENT = 'project-move://progress';

/** States `recovery_options`/`rearm_interrupted_moves` (mover.rs) treat as mid-flight. */
export const ACTIVE_MOVE_STATES = new Set(['copying', 'verifying', 'verified']);

export function isMoveActive(state: string): boolean {
  return ACTIVE_MOVE_STATES.has(state);
}

/** The project's own in-flight move, if any. `list_project_moves` has no project filter. */
export function findActiveMove(
  moves: MoveSummary[] | undefined,
  projectId: number
): MoveSummary | undefined {
  return moves?.find((m) => m.project_id === projectId && isMoveActive(m.state));
}

export async function preflightProjectMove(
  id: number,
  dest: string,
  includeRegenerable: boolean
): Promise<PreflightResult> {
  return invokeClient.post<PreflightResult>('preflight_project_move', {
    id,
    dest,
    includeRegenerable,
  });
}

export async function startProjectMove(
  id: number,
  dest: string,
  includeRegenerable: boolean,
  acknowledgedWarnings: boolean
): Promise<MoveSummary> {
  return invokeClient.post<MoveSummary>('start_project_move', {
    id,
    dest,
    includeRegenerable,
    acknowledgedWarnings,
  });
}

export async function cancelProjectMove(id: string): Promise<void> {
  await invokeClient.post<void>('cancel_project_move', { id });
}

export async function listProjectMoves(): Promise<MoveSummary[]> {
  return invokeClient.post<MoveSummary[]>('list_project_moves');
}

export async function resolveInterruptedMove(
  id: string,
  action: 'resume' | 'finish' | 'rollback'
): Promise<MoveSummary | null> {
  return invokeClient.post<MoveSummary | null>('resolve_interrupted_move', { id, action });
}

export async function endPortProcess(port: number, pid: number, startTime: number): Promise<void> {
  await invokeClient.post<void>('end_port_process', { port, pid, startTime });
}

export async function removeMovedSource(id: string): Promise<void> {
  await invokeClient.post<void>('remove_moved_source', { id });
}

export type { MoveSummary, PreflightResult };
export type { PortWarning } from '$lib/domains/projects/generated/PortWarning';
export type { SkippedDir } from '$lib/domains/projects/generated/SkippedDir';
export type { NestedProject } from '$lib/domains/projects/generated/NestedProject';
