import { invoke } from "@tauri-apps/api/core";
import { isTauriEnvironment } from "$lib/utils/tauri";
import type {
  EnvApplyResult,
  EnvChange,
  EnvPermissions,
  EnvVariable,
} from "./types";

function requireTauri(): void {
  if (!isTauriEnvironment()) {
    throw new Error(
      "Environment variable editing is only available in the desktop app.",
    );
  }
}

export async function listEnvironmentVariables(): Promise<EnvVariable[]> {
  requireTauri();
  return invoke<EnvVariable[]>("env_list_variables");
}

export async function getEnvironmentPermissions(): Promise<EnvPermissions> {
  requireTauri();
  return invoke<EnvPermissions>("env_get_permissions");
}

export async function applyEnvironmentChanges(
  changes: EnvChange[],
): Promise<EnvApplyResult> {
  requireTauri();
  return invoke<EnvApplyResult>("env_apply_changes", { changes });
}

export async function refreshProcessEnvironment(): Promise<void> {
  requireTauri();
  return invoke("env_refresh_process");
}

export async function requestElevation(): Promise<EnvPermissions> {
  requireTauri();
  return invoke<EnvPermissions>("env_request_elevation");
}

export function buildChanges(
  original: EnvVariable[],
  rows: Array<{
    name: string;
    value: string;
    scope: EnvVariable["scope"];
    isNew?: boolean;
    isDeleted?: boolean;
    isDirty?: boolean;
  }>,
): EnvChange[] {
  const changes: EnvChange[] = [];
  const originalByKey = new Map(
    original.map((v) => [`${v.scope}:${v.name}`, v]),
  );

  for (const row of rows) {
    const key = `${row.scope}:${row.name}`;
    const prev = originalByKey.get(key);

    if (row.isDeleted && prev) {
      changes.push({
        action: "delete",
        name: prev.name,
        scope: prev.scope,
      });
      continue;
    }

    if (row.isNew || row.isDirty) {
      if (!row.name.trim()) continue;
      changes.push({
        action: "set",
        name: row.name.trim(),
        value: row.value,
        scope: row.scope,
      });
    }
  }

  return changes;
}

export function hasSystemChanges(changes: EnvChange[]): boolean {
  return changes.some((c) => c.scope === "system");
}
