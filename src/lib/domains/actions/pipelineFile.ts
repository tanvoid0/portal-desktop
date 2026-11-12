/**
 * Load and parse optional `.portal/pipeline.yml` (or .yaml / .json).
 */

import { parse as parseYaml } from 'yaml';
import { invoke } from '@tauri-apps/api/core';
import type { PortalPipelineFile } from './types';

const PIPELINE_RELATIVE_PATHS = [
  '.portal/pipeline.yml',
  '.portal/pipeline.yaml',
  '.portal/pipeline.json',
];

/** `strict` (doctor verify only) also rejects workflow steps naming unknown actions. */
export function parsePortalPipelineFile(raw: string, strict = false): PortalPipelineFile {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error('Empty pipeline file');
  }

  let data: unknown;
  if (trimmed.startsWith('{')) {
    data = JSON.parse(trimmed);
  } else {
    data = parseYaml(trimmed);
  }

  return validatePortalPipelineFile(data, strict);
}

// ponytail: known-action set for workflow refs is the file's own actions plus the
// built-in default ids (defaults.ts never has a profile here). A file workflow that
// references a *custom* DB action is the only false-reject; mergeCatalog still warns.
const RESERVED_ACTION_IDS = ['install', 'build', 'test', 'dev', 'lint', 'typecheck'];

export function validatePortalPipelineFile(data: unknown, strict = false): PortalPipelineFile {
  if (!data || typeof data !== 'object') {
    throw new Error('Pipeline file must be an object');
  }
  const obj = data as Record<string, unknown>;
  const version = obj.version;
  if (version !== 1 && version !== '1') {
    throw new Error(`Unsupported pipeline file version: ${String(version)}`);
  }

  const actions = (obj.actions ?? undefined) as PortalPipelineFile['actions'];
  if (actions != null) {
    if (typeof actions !== 'object' || Array.isArray(actions)) {
      throw new Error('`actions` must be a map of action id → definition');
    }
    for (const [id, def] of Object.entries(actions)) {
      if (!def || typeof def !== 'object' || typeof def.run !== 'string' || !def.run.trim()) {
        throw new Error(`Action "${id}" must have a non-empty \`run\` command`);
      }
    }
  }

  const workflows = (obj.workflows ?? undefined) as PortalPipelineFile['workflows'];
  if (workflows != null) {
    if (typeof workflows !== 'object' || Array.isArray(workflows)) {
      throw new Error('`workflows` must be a map of workflow id → definition');
    }
    const known = new Set([...RESERVED_ACTION_IDS, ...Object.keys(actions ?? {})]);
    for (const [id, def] of Object.entries(workflows)) {
      if (!def || !Array.isArray(def.steps) || def.steps.length === 0) {
        throw new Error(`Workflow "${id}" must have a non-empty \`steps\` array`);
      }
      for (const step of def.steps) {
        if (!step || typeof step.action !== 'string' || !step.action.trim()) {
          throw new Error(`Workflow "${id}" has a step with no \`action\``);
        }
        if (strict && !known.has(step.action)) {
          throw new Error(`Workflow "${id}" references unknown action "${step.action}"`);
        }
      }
    }
  }

  return {
    version: 1,
    release: (obj.release as PortalPipelineFile['release']) ?? undefined,
    actions: actions ?? undefined,
    workflows: workflows ?? undefined,
  };
}

/**
 * Try to read `.portal/pipeline.yml` from a project path.
 * Returns null if missing (not an error).
 */
export async function loadPortalPipelineFile(
  projectPath: string
): Promise<{ path: string; file: PortalPipelineFile } | null> {
  for (const relative of PIPELINE_RELATIVE_PATHS) {
    try {
      const content = await invoke<string>('coder_read_file', {
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

/**
 * Reload + validate for the Pipeline-Doctor verify step. Unlike
 * `loadPortalPipelineFile`, a present-but-invalid file throws (parse/validation
 * error) instead of being silently treated as missing — so a bad agent edit is
 * surfaced rather than falling back to the wrong default and re-failing.
 */
export async function reloadAndValidatePortalPipelineFile(
  projectPath: string
): Promise<{ path: string; file: PortalPipelineFile } | null> {
  for (const relative of PIPELINE_RELATIVE_PATHS) {
    let content: string;
    try {
      content = await invoke<string>('coder_read_file', {
        workspaceRoot: projectPath,
        path: relative,
      });
    } catch {
      continue; // candidate missing / outside workspace — not an error
    }
    if (!content?.trim()) continue;
    return { path: relative, file: parsePortalPipelineFile(content, true) }; // invalid → throws
  }
  return null;
}
