import { invoke } from '@tauri-apps/api/core';
import { isTauriEnvironment } from '$lib/utils/tauri';

export interface InstalledProgram {
  id: string;
  name: string;
  version: string;
  publisher: string;
  installLocation: string;
  installDate: string;
  sizeKb: number;
  uninstallString: string;
  quietUninstallString: string;
  isMsi: boolean;
  systemComponent: boolean;
}

export interface Leftover {
  kind: 'folder' | 'registry';
  path: string;
  sizeBytes: number;
}

export interface UninstallOutcome {
  exitCode: number | null;
  removed: boolean;
  silent: boolean;
  message: string;
  cleanup: { removed: string[]; failed: string[]; backupDir: string } | null;
}

export const silentAvailable = (p: InstalledProgram) => p.isMsi || !!p.quietUninstallString;

function requireTauri(): void {
  if (!isTauriEnvironment())
    throw new Error('The uninstaller is only available in the desktop app.');
}

export async function listPrograms(): Promise<InstalledProgram[]> {
  requireTauri();
  return invoke<InstalledProgram[]>('uninstaller_list_programs');
}

export async function scanLeftovers(id: string): Promise<Leftover[]> {
  requireTauri();
  return invoke<Leftover[]>('uninstaller_scan_leftovers', { id });
}

/** `cleanupPaths` are removed after a successful uninstall (backend re-validates them). */
export async function uninstallProgram(
  id: string,
  silent: boolean,
  cleanupPaths: string[]
): Promise<UninstallOutcome> {
  requireTauri();
  return invoke<UninstallOutcome>('uninstaller_run', {
    id,
    silent,
    cleanupPaths: cleanupPaths.length ? cleanupPaths : null,
  });
}
