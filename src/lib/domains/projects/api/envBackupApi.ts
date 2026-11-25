import { invokeClient } from '$lib/utils/invokeClient';
import type { EnvBackupReport, EnvBackupScan, EnvFileRef, EnvRestoreItem } from '../generated';

/** Env files on disk next to their vault copies. Read-only; never prompts for the vault. */
export function scanEnvBackup(): Promise<EnvBackupScan> {
  return invokeClient.post<EnvBackupScan>('env_backup_scan');
}

/** Disk to vault. `null` backs up every env file found, otherwise only the listed ones. */
export function importEnvFiles(files: EnvFileRef[] | null): Promise<EnvBackupReport> {
  return invokeClient.post<EnvBackupReport>('env_backup_import', { files });
}

/** Vault to disk. Never replaces a differing file unless that item says `overwrite`. */
export function restoreEnvFiles(items: EnvRestoreItem[]): Promise<EnvBackupReport> {
  return invokeClient.post<EnvBackupReport>('env_backup_restore', { items });
}
