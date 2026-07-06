/**
 * Framework Starter Service
 * Provisions default Install / Dev / Build pipelines per detected framework.
 */

import { logger } from "$lib/domains/shared";
import type { Project } from "$lib/domains/projects/types";
import {
  getProjectFramework,
  getProjectPackageManager,
} from "$lib/domains/projects/utils/display";
import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
import { pipelineService } from "./pipelineService";
import type {
  CreatePipelineRequest,
  ExecutionContext,
  Pipeline,
  PipelineStep,
} from "../types";

const log = logger.createScoped("FrameworkStarterService");

export type StarterCategory = "install" | "dev" | "build";

export interface StarterPipelineDef {
  presetKey: string;
  name: string;
  description: string;
  category: StarterCategory;
  command: string;
  longRunning?: boolean;
}

export interface FrameworkStarterPack {
  frameworkSlug: string;
  displayName: string;
  pipelines: StarterPipelineDef[];
}

const FRAMEWORK_SLUG_MAP: Record<string, string> = {
  React: "react",
  "Next.js": "nextjs",
  nextjs: "nextjs",
  Vue: "vue",
  "Vue.js": "vue",
  Svelte: "svelte",
  "Node.js": "nodejs",
  Angular: "angular",
  Nuxt: "nuxt",
  Django: "django",
  Rust: "rust",
  Tauri: "rust",
};

/** Prefer specific app frameworks over generic runtime tags (e.g. Next.js over Node.js) */
const STARTER_FRAMEWORK_PRIORITY = [
  "nextjs",
  "nuxt",
  "react",
  "vue",
  "svelte",
  "angular",
  "django",
  "rust",
];

const STARTER_PACKS: FrameworkStarterPack[] = [
  {
    frameworkSlug: "nextjs",
    displayName: "Next.js",
    pipelines: [
      {
        presetKey: "nextjs-install",
        name: "Install Dependencies",
        description: "Install project dependencies with your package manager",
        category: "install",
        command: "${PACKAGE_MANAGER} install",
      },
      {
        presetKey: "nextjs-dev",
        name: "Run Dev Server",
        description: "Start the Next.js development server",
        category: "dev",
        command: "${PACKAGE_MANAGER} run dev",
        longRunning: true,
      },
      {
        presetKey: "nextjs-build",
        name: "Build Production",
        description: "Create an optimized production build",
        category: "build",
        command: "${PACKAGE_MANAGER} run build",
      },
    ],
  },
  {
    frameworkSlug: "react",
    displayName: "React",
    pipelines: [
      {
        presetKey: "react-install",
        name: "Install Dependencies",
        description: "Install project dependencies",
        category: "install",
        command: "${PACKAGE_MANAGER} install",
      },
      {
        presetKey: "react-dev",
        name: "Run Dev Server",
        description: "Start the development server",
        category: "dev",
        command: "${PACKAGE_MANAGER} run dev",
        longRunning: true,
      },
      {
        presetKey: "react-build",
        name: "Build Production",
        description: "Create a production build",
        category: "build",
        command: "${PACKAGE_MANAGER} run build",
      },
    ],
  },
  {
    frameworkSlug: "vue",
    displayName: "Vue",
    pipelines: [
      {
        presetKey: "vue-install",
        name: "Install Dependencies",
        description: "Install project dependencies",
        category: "install",
        command: "${PACKAGE_MANAGER} install",
      },
      {
        presetKey: "vue-dev",
        name: "Run Dev Server",
        description: "Start the development server",
        category: "dev",
        command: "${PACKAGE_MANAGER} run dev",
        longRunning: true,
      },
      {
        presetKey: "vue-build",
        name: "Build Production",
        description: "Create a production build",
        category: "build",
        command: "${PACKAGE_MANAGER} run build",
      },
    ],
  },
  {
    frameworkSlug: "svelte",
    displayName: "Svelte",
    pipelines: [
      {
        presetKey: "svelte-install",
        name: "Install Dependencies",
        description: "Install project dependencies",
        category: "install",
        command: "${PACKAGE_MANAGER} install",
      },
      {
        presetKey: "svelte-dev",
        name: "Run Dev Server",
        description: "Start the development server",
        category: "dev",
        command: "${PACKAGE_MANAGER} run dev",
        longRunning: true,
      },
      {
        presetKey: "svelte-build",
        name: "Build Production",
        description: "Create a production build",
        category: "build",
        command: "${PACKAGE_MANAGER} run build",
      },
    ],
  },
];

function defaultExecutionContext(): ExecutionContext {
  return {
    type: "sdk",
    sdkType: "node",
    workingDirectory: "${PROJECT_PATH}",
  };
}

function buildStep(def: StarterPipelineDef, command: string): PipelineStep {
  return {
    id: def.category,
    blockId: `starter-${def.category}`,
    name: def.name,
    config: {
      command,
      longRunning: def.longRunning ?? false,
    },
    dependsOn: [],
  };
}

export function frameworkToSlug(framework: string | undefined): string | undefined {
  if (!framework) return undefined;
  return FRAMEWORK_SLUG_MAP[framework] ?? framework.toLowerCase().replace(/\s+/g, "-");
}

export function getStarterPack(slug: string): FrameworkStarterPack | undefined {
  return STARTER_PACKS.find((p) => p.frameworkSlug === slug);
}

/** Resolve framework names from IDs (icon registry) or legacy metadata */
export function resolveFrameworkNames(project: Project): string[] {
  const fromRegistry = projectIconRegistry
    .resolveFrameworks(project)
    .map((f) => f.name);
  if (fromRegistry.length > 0) return fromRegistry;

  const legacy = getProjectFramework(project);
  return legacy ? [legacy] : [];
}

/** Resolve package manager from IDs (icon registry) or legacy metadata */
export function resolvePackageManagerName(
  project: Project,
): string | undefined {
  const fromRegistry = projectIconRegistry.resolvePackageManagers(project);
  if (fromRegistry.length > 0) return fromRegistry[0].name;
  return getProjectPackageManager(project);
}

/** Normalize package-manager script commands, respecting pnpm/yarn shorthand. */
export function normalizePackageScriptCommand(
  pm: string,
  command: string,
  script: "build" | "dev" | "test" | "install",
): string {
  const trimmed = command.trim();
  if (!trimmed) {
    return formatPackageScriptCommand(pm, script);
  }

  const runPattern = new RegExp(`^(npm|yarn|pnpm)\\s+run\\s+${script}\\b`);
  if (runPattern.test(trimmed)) {
    return trimmed.replace(/^(npm|yarn|pnpm)/, pm);
  }

  const shorthandPattern = new RegExp(`^(npm|yarn|pnpm)\\s+${script}\\b`);
  if (shorthandPattern.test(trimmed)) {
    return trimmed.replace(/^(npm|yarn|pnpm)/, pm);
  }

  if (script === "install" && /^(npm|yarn|pnpm)\s+install\b/.test(trimmed)) {
    return trimmed.replace(/^(npm|yarn|pnpm)/, pm);
  }

  // Raw script body from package.json (e.g. "next build --webpack") → invoke via script name
  if (script !== "install" && !/^(npm|yarn|pnpm)\s/.test(trimmed)) {
    return formatPackageScriptCommand(pm, script);
  }

  return trimmed.replace(/\$\{PACKAGE_MANAGER\}/g, pm);
}

function formatPackageScriptCommand(
  pm: string,
  script: "build" | "dev" | "test" | "install",
): string {
  if (pm === "npm") {
    return `${pm} run ${script}`;
  }
  return `${pm} ${script}`;
}

function findStarterPackForFrameworks(
  frameworkNames: string[],
): FrameworkStarterPack | null {
  const slugs = frameworkNames
    .map((name) => frameworkToSlug(name))
    .filter((slug): slug is string => !!slug);

  for (const prioritySlug of STARTER_FRAMEWORK_PRIORITY) {
    if (slugs.includes(prioritySlug)) {
      const pack = getStarterPack(prioritySlug);
      if (pack) return pack;
    }
  }

  for (const slug of slugs) {
    const pack = getStarterPack(slug);
    if (pack) return pack;
  }

  return null;
}

export function resolveCommands(
  project: Project,
  def: StarterPipelineDef,
): string {
  const pm = resolvePackageManagerName(project)?.toLowerCase() ?? "npm";

  if (def.category === "dev" && project.start_command) {
    return normalizePackageScriptCommand(pm, project.start_command, "dev");
  }
  if (def.category === "build" && project.build_command) {
    return normalizePackageScriptCommand(pm, project.build_command, "build");
  }
  if (def.category === "install") {
    return `${pm} install`;
  }

  return def.command.replace(/\$\{PACKAGE_MANAGER\}/g, pm);
}

export interface StarterPackStatus {
  pack: FrameworkStarterPack;
  frameworkSlug: string;
  provisionedKeys: string[];
  missingKeys: string[];
  isFullyProvisioned: boolean;
}

export function getStarterPackForProject(
  project: Project,
  existingPipelines: Pipeline[],
): StarterPackStatus | null {
  const pack = findStarterPackForFrameworks(resolveFrameworkNames(project));
  if (!pack) return null;

  const provisionedKeys = existingPipelines
    .map((p) => p.presetKey)
    .filter((k): k is string => !!k);

  const missingKeys = pack.pipelines
    .map((p) => p.presetKey)
    .filter((key) => !provisionedKeys.includes(key));

  return {
    pack,
    frameworkSlug: pack.frameworkSlug,
    provisionedKeys,
    missingKeys,
    isFullyProvisioned: missingKeys.length === 0,
  };
}

/** Ensure icon registry is loaded, then provision missing starter pipelines */
export async function autoProvisionStarterPipelines(
  projectId: string,
  project: Project,
  existingPipelines: Pipeline[],
): Promise<Pipeline[]> {
  await projectIconRegistry.ensureLoaded();
  return provisionStarterPipelines(projectId, project, existingPipelines);
}

export async function provisionStarterPipelines(
  projectId: string,
  project: Project,
  existingPipelines: Pipeline[],
): Promise<Pipeline[]> {
  const status = getStarterPackForProject(project, existingPipelines);
  if (!status || status.missingKeys.length === 0) {
    return [];
  }

  const created: Pipeline[] = [];

  for (const def of status.pack.pipelines) {
    if (!status.missingKeys.includes(def.presetKey)) {
      continue;
    }

    const command = resolveCommands(project, def);
    const request: CreatePipelineRequest = {
      name: def.name,
      description: def.description,
      projectId,
      steps: [buildStep(def, command)],
      variables: [],
      secrets: [],
      executionContext: defaultExecutionContext(),
      enabled: true,
      presetKey: def.presetKey,
      category: def.category,
    };

    log.info("Provisioning starter pipeline", {
      presetKey: def.presetKey,
      projectId,
    });
    const pipeline = await pipelineService.createPipeline(request);
    created.push(pipeline);
  }

  return created;
}

export const frameworkStarterService = {
  frameworkToSlug,
  getStarterPack,
  getStarterPackForProject,
  autoProvisionStarterPipelines,
  provisionStarterPipelines,
  resolveCommands,
  resolveFrameworkNames,
  resolvePackageManagerName,
  normalizePackageScriptCommand,
  STARTER_PACKS,
};
