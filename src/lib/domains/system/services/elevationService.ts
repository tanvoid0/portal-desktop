import { invoke } from '@tauri-apps/api/core';
import { isTauriEnvironment } from '$lib/utils/tauri';
import type { ElevatedRunResult, ElevationStatus } from '../types';

function requireTauri(): void {
  if (!isTauriEnvironment()) {
    throw new Error('Elevation is only available in the desktop app.');
  }
}

/** Is this process already running elevated (admin / root)? */
export async function getElevationStatus(): Promise<ElevationStatus> {
  requireTauri();
  return invoke<ElevationStatus>('elevation_status');
}

/**
 * Run one external program elevated (UAC / pkexec / osascript) while Portal
 * itself stays unprivileged. Use for a single step that needs admin rights.
 */
export async function runElevated(exe: string, args: string[] = []): Promise<ElevatedRunResult> {
  requireTauri();
  return invoke<ElevatedRunResult>('run_elevated', { exe, args });
}

/**
 * Relaunch Portal itself elevated and close this instance. Use when the rest
 * of the session needs admin rights, not just one step. The app restarts in
 * place — persisted UI state (route, etc.) should already be in
 * localStorage so it survives the relaunch.
 */
export async function restartElevated(): Promise<void> {
  requireTauri();
  return invoke('restart_elevated');
}
