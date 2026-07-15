/**
 * Automation context — standard variables for any directory or project.
 * Prefer ProjectAutomationProfile when available (correct package manager).
 */

import type { Project } from "$lib/domains/projects/types";
import type { ResolveContext } from "./blockResolver";
import type { ProjectAutomationProfile } from "$lib/domains/actions/types";
import {
  profileFromProject,
  profileVariables,
} from "$lib/domains/actions/profile";

/** Loose project fields accepted by the legacy automation API */
export type AutomationProjectRef = {
  id?: string;
  name: string;
  path: string;
  package_manager?: string;
  build_command?: string;
  start_command?: string;
  test_command?: string;
  metadata?: Project["metadata"];
  package_manager_ids?: Project["package_manager_ids"];
  framework_ids?: Project["framework_ids"];
};

export interface AutomationContextOptions {
  /** Working directory (required for execution) */
  cwd: string;
  /** Extra template variables (override builtins) */
  variables?: Record<string, string | number | boolean>;
  secrets?: Record<string, string>;
  /** Optional linked project — full Project preferred */
  project?: Project | AutomationProjectRef;
  /** Pre-resolved automation profile (wins over project fields) */
  profile?: ProjectAutomationProfile;
}

function toStringRecord(
  vars: Record<string, string | number | boolean> = {},
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(vars).map(([k, v]) => [
      k,
      typeof v === "boolean" ? (v ? "true" : "false") : String(v),
    ]),
  );
}

function resolveProfile(
  options: AutomationContextOptions,
): ProjectAutomationProfile | null {
  if (options.profile) return options.profile;
  if (options.project && "status" in options.project) {
    return profileFromProject(options.project as Project);
  }
  if (options.project) {
    const p = options.project;
    return {
      id: p.id,
      name: p.name,
      path: p.path,
      packageManager: p.package_manager?.toLowerCase() ?? "npm",
      frameworks: [],
      buildCommand: p.build_command,
      startCommand: p.start_command,
      testCommand: p.test_command,
    };
  }
  return null;
}

/** Built-in variables available in every automation run */
export function createAutomationContext(
  options: AutomationContextOptions,
): ResolveContext {
  const cwd = options.cwd.replace(/\\/g, "/");
  const profile = resolveProfile(options);

  const builtins: Record<string, string> = profile
    ? profileVariables({ ...profile, path: profile.path || cwd })
    : {
        CWD: cwd,
        PROJECT_PATH: cwd,
        WORKING_DIR: cwd,
        PROJECT_NAME: "",
        PACKAGE_MANAGER: "npm",
      };

  builtins.CWD = cwd;
  builtins.WORKING_DIR = cwd;
  if (!builtins.PROJECT_PATH) builtins.PROJECT_PATH = cwd;

  return {
    variables: {
      ...builtins,
      ...toStringRecord(options.variables),
    },
    secrets: options.secrets,
  };
}

export function contextFromProject(
  project: Project | AutomationProjectRef,
  extra?: Record<string, string | number | boolean>,
): ResolveContext {
  return createAutomationContext({
    cwd: project.path,
    project,
    variables: extra,
  });
}

export function contextFromDirectory(
  cwd: string,
  extra?: Record<string, string | number | boolean>,
): ResolveContext {
  return createAutomationContext({ cwd, variables: extra });
}

export function contextFromProfile(
  profile: ProjectAutomationProfile,
  extra?: Record<string, string | number | boolean>,
): ResolveContext {
  return createAutomationContext({
    cwd: profile.path,
    profile,
    variables: extra,
  });
}
