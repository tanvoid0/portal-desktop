/** Canonical route for a project's terminal workspace (cwd = project path). */
export function projectTerminalHref(projectId: string): string {
  return `/projects/${projectId}?tab=terminal`;
}
