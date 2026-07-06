/**
 * Automation context — standard variables for any directory or project.
 * Use this so blocks work the same whether you pass a project path or any folder.
 */

import type { Project } from "$lib/domains/projects/types";
import type { ResolveContext } from "./blockResolver";

export interface AutomationContextOptions {
  /** Working directory (required for execution) */
  cwd: string;
  /** Extra template variables (override builtins) */
  variables?: Record<string, string | number | boolean>;
  secrets?: Record<string, string>;
  /** Optional linked project metadata */
  project?: Pick<
    Project,
    "id" | "name" | "path" | "package_manager" | "build_command" | "start_command"
  >;
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

/** Built-in variables available in every automation run */
export function createAutomationContext(
  options: AutomationContextOptions,
): ResolveContext {
  const cwd = options.cwd.replace(/\\/g, "/");
  const project = options.project;

  const builtins: Record<string, string> = {
    CWD: cwd,
    PROJECT_PATH: project?.path?.replace(/\\/g, "/") ?? cwd,
    WORKING_DIR: cwd,
    PROJECT_NAME: project?.name ?? "",
    PACKAGE_MANAGER: project?.package_manager ?? "npm",
  };

  if (project?.build_command) {
    builtins.BUILD_COMMAND = project.build_command;
  }
  if (project?.start_command) {
    builtins.START_COMMAND = project.start_command;
  }
  if (project?.id) {
    builtins.PROJECT_ID = project.id;
  }

  return {
    variables: {
      ...builtins,
      ...toStringRecord(options.variables),
    },
    secrets: options.secrets,
  };
}

export function contextFromProject(
  project: Pick<
    Project,
    "id" | "name" | "path" | "package_manager" | "build_command" | "start_command"
  >,
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
