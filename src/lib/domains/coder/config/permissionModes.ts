import type { PermissionMode } from "../types.js";

export interface PermissionModeConfig {
  value: PermissionMode;
  label: string;
  hint: string;
}

export const PERMISSION_MODES: PermissionModeConfig[] = [
  {
    value: "review",
    label: "Review",
    hint: "Prompt before mutating actions",
  },
  {
    value: "auto-accept-all",
    label: "Auto",
    hint: "Run allowed actions automatically",
  },
];

export function getPermissionModeConfig(mode: PermissionMode): PermissionModeConfig {
  return PERMISSION_MODES.find((m) => m.value === mode) ?? PERMISSION_MODES[0];
}

export function nextPermissionMode(current: PermissionMode): PermissionMode {
  return current === "review" ? "auto-accept-all" : "review";
}
