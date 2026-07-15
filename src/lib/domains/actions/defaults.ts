/**
 * Smart default local actions + workflows from project metadata.
 * No DB rows required — ephemeral catalog entries.
 */

import {
  normalizePackageScriptCommand,
} from "$lib/domains/projects/pipelines/services/frameworkStarterService";
import type {
  ProjectAutomationProfile,
  UnifiedAction,
  UnifiedWorkflow,
} from "./types";
import { substituteVars, profileVariables } from "./profile";

function formatPmScript(
  pm: string,
  script: "build" | "dev" | "test" | "install" | "lint",
): string {
  if (script === "install") return `${pm} install`;
  if (pm === "npm") return `npm run ${script}`;
  return `${pm} ${script}`;
}

function resolveScriptCommand(
  profile: ProjectAutomationProfile,
  script: "build" | "dev" | "test" | "install" | "lint",
  explicit?: string,
): string {
  const pm = profile.packageManager;
  if (script === "lint") {
    return explicit?.trim() || formatPmScript(pm, "lint");
  }
  if (explicit?.trim()) {
    return normalizePackageScriptCommand(pm, explicit, script);
  }
  return formatPmScript(pm, script);
}

/** Node-like ecosystems that use package.json scripts */
function isNodeLike(profile: ProjectAutomationProfile): boolean {
  const pm = profile.packageManager;
  if (["npm", "yarn", "pnpm", "bun"].includes(pm)) return true;
  const fw = [...profile.frameworks, profile.framework ?? ""].map((f) =>
    f.toLowerCase(),
  );
  return fw.some((f) =>
    [
      "react",
      "next.js",
      "nextjs",
      "vue",
      "vue.js",
      "svelte",
      "angular",
      "nuxt",
      "node.js",
      "nodejs",
      "express",
      "electron",
      "tauri",
    ].includes(f),
  );
}

function isRustLike(profile: ProjectAutomationProfile): boolean {
  const pm = profile.packageManager;
  if (pm === "cargo") return true;
  return [...profile.frameworks, profile.framework ?? ""]
    .map((f) => f.toLowerCase())
    .some((f) => f === "rust" || f === "tauri");
}

/**
 * Build default local actions for a project profile.
 * Prefer build/start/test commands from metadata over generics.
 */
export function buildDefaultActions(
  profile: ProjectAutomationProfile,
): UnifiedAction[] {
  const vars = profileVariables(profile);
  const actions: UnifiedAction[] = [];

  if (isRustLike(profile) && !isNodeLike(profile)) {
    actions.push(
      {
        id: "install",
        name: "Fetch Dependencies",
        description: "cargo fetch",
        source: "local",
        runner: "local",
        category: "install",
        command: "cargo fetch",
      },
      {
        id: "build",
        name: "Build",
        description: "cargo build",
        source: "local",
        runner: "local",
        category: "build",
        command: profile.buildCommand?.trim() || "cargo build",
      },
      {
        id: "test",
        name: "Test",
        description: "cargo test",
        source: "local",
        runner: "local",
        category: "test",
        command: profile.testCommand?.trim() || "cargo test",
      },
      {
        id: "dev",
        name: "Run",
        description: "cargo run",
        source: "local",
        runner: "local",
        category: "dev",
        command: profile.startCommand?.trim() || "cargo run",
        longRunning: true,
      },
    );
  } else {
    // Default: Node / generic package-manager projects
    actions.push({
      id: "install",
      name: "Install Dependencies",
      description: "Install project dependencies",
      source: "local",
      runner: "local",
      category: "install",
      command: resolveScriptCommand(profile, "install"),
    });

    actions.push({
      id: "dev",
      name: "Dev Server",
      description: "Start the development server",
      source: "local",
      runner: "local",
      category: "dev",
      command: resolveScriptCommand(profile, "dev", profile.startCommand),
      longRunning: true,
    });

    actions.push({
      id: "build",
      name: "Build",
      description: "Create a production build",
      source: "local",
      runner: "local",
      category: "build",
      command: resolveScriptCommand(profile, "build", profile.buildCommand),
    });

    actions.push({
      id: "test",
      name: "Test",
      description: "Run the test suite",
      source: "local",
      runner: "local",
      category: "test",
      command: resolveScriptCommand(profile, "test", profile.testCommand),
    });

    actions.push({
      id: "lint",
      name: "Lint",
      description: "Run the linter",
      source: "local",
      runner: "local",
      category: "lint",
      command: resolveScriptCommand(profile, "lint"),
    });
  }

  return actions.map((a) => ({
    ...a,
    command: a.command ? substituteVars(a.command, vars) : a.command,
  }));
}

/** Default local workflows composed from default actions */
export function buildDefaultWorkflows(
  actions: UnifiedAction[],
): UnifiedWorkflow[] {
  const ids = new Set(actions.map((a) => a.id));
  const workflows: UnifiedWorkflow[] = [];

  if (ids.has("install") && ids.has("test") && ids.has("build")) {
    workflows.push({
      id: "ci",
      name: "CI",
      description: "Install → Test → Build",
      source: "local",
      runner: "local",
      steps: [
        { action: "install" },
        { action: "test", needs: ["install"] },
        { action: "build", needs: ["test"] },
      ],
    });
  }

  return workflows;
}
