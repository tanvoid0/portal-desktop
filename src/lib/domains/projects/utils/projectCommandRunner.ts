/**
 * Run project build/start/test commands via pipelines or shell fallback.
 */

import type { Project } from "$lib/domains/projects/types";
import type { Pipeline } from "$lib/domains/projects/pipelines";
import { invokeClient } from "$lib/utils/invokeClient";

export type ProjectCommandKind = "build" | "start" | "test";

export function getProjectCommand(
  project: Project,
  kind: ProjectCommandKind,
): string | undefined {
  switch (kind) {
    case "build":
      return project.build_command?.trim() || undefined;
    case "start":
      return project.start_command?.trim() || undefined;
    case "test":
      return project.test_command?.trim() || undefined;
  }
}

export function findPipelineForCommand(
  pipelines: Pipeline[],
  kind: ProjectCommandKind,
): Pipeline | undefined {
  switch (kind) {
    case "build":
      return pipelines.find((p) => p.enabled && p.category === "build");
    case "start":
      return pipelines.find((p) => p.enabled && p.category === "dev");
    case "test":
      return (
        pipelines.find((p) => p.enabled && p.category === "build" && /test/i.test(p.name)) ??
        pipelines.find((p) => p.enabled && /test/i.test(p.name))
      );
  }
}

export function commandKindLabel(kind: ProjectCommandKind): string {
  switch (kind) {
    case "build":
      return "Build";
    case "start":
      return "Start";
    case "test":
      return "Test";
  }
}

function isWindows(): boolean {
  return typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
}

export async function runShellCommand(
  workingDirectory: string,
  command: string,
): Promise<{ success: boolean; output: string }> {
  const shell = isWindows()
    ? { command: "cmd", args: ["/C", command] }
    : { command: "sh", args: ["-c", command] };

  try {
    const output = await invokeClient.post<string>("execute_command_in_directory", {
      command: shell.command,
      args: shell.args,
      workingDirectory,
    });
    return { success: true, output };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { success: false, output: message };
  }
}
