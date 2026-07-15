/**
 * Build a ProjectAutomationProfile from a Project (or cwd-only context).
 */

import type { Project } from "$lib/domains/projects/types";
import {
  getProjectFramework,
  getProjectPackageManager,
} from "$lib/domains/projects/utils/display";
import { projectIconRegistry } from "$lib/domains/projects/utils/iconRegistry";
import type { ProjectAutomationProfile } from "./types";

export function resolveFrameworkNames(project: Project): string[] {
  const fromRegistry = projectIconRegistry
    .resolveFrameworks(project)
    .map((f) => f.name);
  if (fromRegistry.length > 0) return fromRegistry;

  const legacy = getProjectFramework(project);
  return legacy ? [legacy] : [];
}

export function resolvePackageManagerName(project: Project): string {
  const fromRegistry = projectIconRegistry.resolvePackageManagers(project);
  if (fromRegistry.length > 0) {
    return fromRegistry[0].name.toLowerCase();
  }
  return (
    getProjectPackageManager(project)?.toLowerCase() ??
    (project as Project & { package_manager?: string }).package_manager?.toLowerCase() ??
    "npm"
  );
}

export function profileFromProject(project: Project): ProjectAutomationProfile {
  const frameworks = resolveFrameworkNames(project);
  return {
    id: project.id,
    name: project.name,
    path: project.path,
    packageManager: resolvePackageManagerName(project),
    framework: frameworks[0],
    frameworks,
    buildCommand: project.build_command,
    startCommand: project.start_command,
    testCommand: project.test_command,
  };
}

export function profileFromDirectory(
  cwd: string,
  packageManager = "npm",
): ProjectAutomationProfile {
  const name = cwd.replace(/\\/g, "/").split("/").filter(Boolean).pop() ?? cwd;
  return {
    name,
    path: cwd,
    packageManager,
    frameworks: [],
  };
}

/** Builtin template variables for action command substitution */
export function profileVariables(
  profile: ProjectAutomationProfile,
): Record<string, string> {
  const cwd = profile.path.replace(/\\/g, "/");
  const vars: Record<string, string> = {
    CWD: cwd,
    PROJECT_PATH: cwd,
    WORKING_DIR: cwd,
    PROJECT_NAME: profile.name,
    PACKAGE_MANAGER: profile.packageManager,
  };
  if (profile.id) vars.PROJECT_ID = profile.id;
  if (profile.buildCommand) vars.BUILD_COMMAND = profile.buildCommand;
  if (profile.startCommand) vars.START_COMMAND = profile.startCommand;
  if (profile.testCommand) vars.TEST_COMMAND = profile.testCommand;
  if (profile.framework) vars.FRAMEWORK = profile.framework;
  return vars;
}

export function substituteVars(
  template: string,
  variables: Record<string, string>,
): string {
  return template.replace(/\$\{([A-Z0-9_]+)\}/gi, (match, key: string) => {
    const upper = key.toUpperCase();
    return variables[upper] ?? variables[key] ?? match;
  });
}
