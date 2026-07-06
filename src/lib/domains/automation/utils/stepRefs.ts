/**
 * Shorthand helpers for defining block sequences.
 */

import type { AutomationStepInput } from "./blockResolver";

/** Block ID string or full step config */
export type AutomationStepRef = string | AutomationStepInput;

export function normalizeStepRefs(
  refs: AutomationStepRef[],
): AutomationStepInput[] {
  return refs.map((ref) =>
    typeof ref === "string" ? { blockId: ref } : ref,
  );
}

/** Quick presets for common sequences */
export const presets = {
  install: (packageManager = "npm"): AutomationStepRef[] => [
    { blockId: "install-npm", parameters: { packageManager, installCommand: "install" } },
  ],
  test: (): AutomationStepRef[] => [{ blockId: "test-npm" }],
  build: (): AutomationStepRef[] => [{ blockId: "build-npm" }],
  ci: (packageManager = "npm"): AutomationStepRef[] => [
    { blockId: "install-npm", parameters: { packageManager, installCommand: "ci" } },
    { blockId: "test-npm" },
    { blockId: "build-npm" },
  ],
} as const;
