/**
 * Resolve a project's build/start/test commands and the pipeline that runs them.
 * Running goes through the actions catalog (`execute_script`); there is no
 * separate shell path.
 */

import type { Project } from '$lib/domains/projects/types';
import type { Pipeline } from '$lib/domains/projects/pipelines';

export type ProjectCommandKind = 'build' | 'start' | 'test';

export function getProjectCommand(project: Project, kind: ProjectCommandKind): string | undefined {
  switch (kind) {
    case 'build':
      return project.build_command?.trim() || undefined;
    case 'start':
      return project.start_command?.trim() || undefined;
    case 'test':
      return project.test_command?.trim() || undefined;
  }
}

export function findPipelineForCommand(
  pipelines: Pipeline[],
  kind: ProjectCommandKind
): Pipeline | undefined {
  switch (kind) {
    case 'build':
      return pipelines.find((p) => p.enabled && p.category === 'build');
    case 'start':
      return pipelines.find((p) => p.enabled && p.category === 'dev');
    case 'test':
      return (
        pipelines.find((p) => p.enabled && p.category === 'build' && /test/i.test(p.name)) ??
        pipelines.find((p) => p.enabled && /test/i.test(p.name))
      );
  }
}

export function commandKindLabel(kind: ProjectCommandKind): string {
  switch (kind) {
    case 'build':
      return 'Build';
    case 'start':
      return 'Start';
    case 'test':
      return 'Test';
  }
}
