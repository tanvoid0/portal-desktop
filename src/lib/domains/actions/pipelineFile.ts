/**
 * Load and parse optional `.portal/pipeline.yml` (or .yaml / .json).
 */

import { parse as parseYaml } from "yaml";
import { invoke } from "@tauri-apps/api/core";
import type { PortalPipelineFile } from "./types";

const PIPELINE_RELATIVE_PATHS = [
  ".portal/pipeline.yml",
  ".portal/pipeline.yaml",
  ".portal/pipeline.json",
];

export function parsePortalPipelineFile(raw: string): PortalPipelineFile {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("Empty pipeline file");
  }

  let data: unknown;
  if (trimmed.startsWith("{")) {
    data = JSON.parse(trimmed);
  } else {
    data = parseYaml(trimmed);
  }

  return validatePortalPipelineFile(data);
}

export function validatePortalPipelineFile(data: unknown): PortalPipelineFile {
  if (!data || typeof data !== "object") {
    throw new Error("Pipeline file must be an object");
  }
  const obj = data as Record<string, unknown>;
  const version = obj.version;
  if (version !== 1 && version !== "1") {
    throw new Error(`Unsupported pipeline file version: ${String(version)}`);
  }

  return {
    version: 1,
    actions: (obj.actions as PortalPipelineFile["actions"]) ?? undefined,
    workflows: (obj.workflows as PortalPipelineFile["workflows"]) ?? undefined,
  };
}

/**
 * Try to read `.portal/pipeline.yml` from a project path.
 * Returns null if missing (not an error).
 */
export async function loadPortalPipelineFile(
  projectPath: string,
): Promise<{ path: string; file: PortalPipelineFile } | null> {
  for (const relative of PIPELINE_RELATIVE_PATHS) {
    try {
      const content = await invoke<string>("coder_read_file", {
        workspaceRoot: projectPath,
        path: relative,
      });
      if (!content?.trim()) continue;
      return { path: relative, file: parsePortalPipelineFile(content) };
    } catch {
      // Missing file or outside workspace — try next candidate
    }
  }
  return null;
}
