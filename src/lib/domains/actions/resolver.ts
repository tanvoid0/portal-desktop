/**
 * ActionResolver — merge metadata defaults → custom DB → `.portal/pipeline.yml`.
 * Highest priority wins for the same action/workflow id.
 */

import type { Project } from "$lib/domains/projects/types";
import { buildDefaultActions, buildDefaultWorkflows } from "./defaults";
import {
  loadPortalPipelineFile,
  parsePortalPipelineFile,
} from "./pipelineFile";
import {
  profileFromDirectory,
  profileFromProject,
  profileVariables,
  substituteVars,
} from "./profile";
import type {
  PortalPipelineFile,
  ProjectAutomationProfile,
  UnifiedAction,
  UnifiedWorkflow,
} from "./types";

export interface ResolveCatalogOptions {
  /** Custom actions from Portal UI / DB (mid priority) */
  customActions?: UnifiedAction[];
  customWorkflows?: UnifiedWorkflow[];
  /** Pre-parsed file (skips disk read) — highest priority */
  file?: PortalPipelineFile | null;
  /** Skip reading `.portal/pipeline.yml` from disk */
  skipFileLoad?: boolean;
  /** Include lint only when true (caller may detect script) */
  includeLint?: boolean;
}

export interface ActionCatalog {
  profile: ProjectAutomationProfile;
  actions: UnifiedAction[];
  workflows: UnifiedWorkflow[];
  filePath?: string;
  warnings: string[];
}

function mergeById<T extends { id: string }>(layers: T[][]): T[] {
  const map = new Map<string, T>();
  for (const layer of layers) {
    for (const item of layer) {
      map.set(item.id, item);
    }
  }
  return [...map.values()];
}

function fileActionsToUnified(
  file: PortalPipelineFile,
  profile: ProjectAutomationProfile,
): UnifiedAction[] {
  if (!file.actions) return [];
  const vars = profileVariables(profile);
  return Object.entries(file.actions).map(([id, def]) => ({
    id,
    name: def.name ?? id,
    description: def.description,
    source: "file" as const,
    runner: "local" as const,
    category: def.category,
    command: substituteVars(def.run, vars),
    longRunning: def.longRunning,
  }));
}

function fileWorkflowsToUnified(file: PortalPipelineFile): UnifiedWorkflow[] {
  if (!file.workflows) return [];
  return Object.entries(file.workflows).map(([id, def]) => ({
    id,
    name: def.name ?? id,
    description: def.description,
    source: "file" as const,
    runner: "local" as const,
    steps: def.steps.map((s) => ({
      action: s.action,
      needs: s.needs,
    })),
  }));
}

export function mergeCatalog(
  profile: ProjectAutomationProfile,
  options: ResolveCatalogOptions = {},
): ActionCatalog {
  const warnings: string[] = [];
  let defaults = buildDefaultActions(profile);
  if (options.includeLint === false) {
    defaults = defaults.filter((a) => a.id !== "lint");
  }
  const defaultWorkflows = buildDefaultWorkflows(defaults);

  const customActions = options.customActions ?? [];
  const customWorkflows = options.customWorkflows ?? [];

  const file = options.file ?? null;
  const fromFileActions = file ? fileActionsToUnified(file, profile) : [];
  const fromFileWorkflows = file ? fileWorkflowsToUnified(file) : [];

  // Priority (last wins): defaults → custom → file
  const actions = mergeById([defaults, customActions, fromFileActions]);
  const workflows = mergeById([
    defaultWorkflows,
    customWorkflows,
    fromFileWorkflows,
  ]);

  // Validate workflow steps reference known actions
  for (const wf of workflows) {
    for (const step of wf.steps) {
      if (!actions.find((a) => a.id === step.action)) {
        warnings.push(
          `Workflow "${wf.id}" references unknown action "${step.action}"`,
        );
      }
    }
  }

  return { profile, actions, workflows, warnings };
}

/** Resolve catalog for a registered project (loads `.portal/pipeline.yml` when present). */
export async function resolveProjectCatalog(
  project: Project,
  options: ResolveCatalogOptions = {},
): Promise<ActionCatalog> {
  await import("$lib/domains/projects/utils/iconRegistry").then((m) =>
    m.projectIconRegistry.ensureLoaded(),
  );

  const profile = profileFromProject(project);
  let file = options.file ?? null;
  let filePath: string | undefined;

  if (!options.skipFileLoad && file == null) {
    const loaded = await loadPortalPipelineFile(project.path);
    if (loaded) {
      file = loaded.file;
      filePath = loaded.path;
    }
  }

  const catalog = mergeCatalog(profile, { ...options, file });
  return { ...catalog, filePath };
}

/** Resolve catalog for an arbitrary directory (local defaults + optional file). */
export async function resolveDirectoryCatalog(
  cwd: string,
  options: ResolveCatalogOptions & { packageManager?: string } = {},
): Promise<ActionCatalog> {
  const profile = profileFromDirectory(cwd, options.packageManager ?? "npm");
  let file = options.file ?? null;
  let filePath: string | undefined;

  if (!options.skipFileLoad && file == null) {
    const loaded = await loadPortalPipelineFile(cwd);
    if (loaded) {
      file = loaded.file;
      filePath = loaded.path;
    }
  }

  const catalog = mergeCatalog(profile, { ...options, file });
  return { ...catalog, filePath };
}

/** Sync helper for tests — merge with optional YAML/JSON string */
export function resolveCatalogFromFileContent(
  profile: ProjectAutomationProfile,
  rawFile: string | null,
  options: ResolveCatalogOptions = {},
): ActionCatalog {
  const file = rawFile ? parsePortalPipelineFile(rawFile) : options.file ?? null;
  return mergeCatalog(profile, { ...options, file, skipFileLoad: true });
}
