import { buildReference } from '$lib/domains/shared/utils/entityReference';
import { taskStatusLabel, type Task } from '../types';

/** Paste-ready reference for a task — drop into Claude as "this is the task I mean". */
export function buildTaskReference(task: Task, projectName?: string): string {
  const priority = task.priority.charAt(0).toUpperCase() + task.priority.slice(1);
  return buildReference({
    kind: 'task',
    id: task.id,
    title: task.title,
    fields: {
      Status: taskStatusLabel(task.status),
      Priority: priority,
      Type: task.type || undefined,
      Project: projectName,
      Tags: task.tags?.length ? task.tags.join(', ') : undefined,
    },
    body: task.description?.trim() || '(no description)',
  });
}

/** The reference wrapped as an actionable prompt for a coder session. */
export function buildCoderPrompt(task: Task, projectName?: string): string {
  return `${buildTaskReference(task, projectName)}\n\nImplement this task. Explore the relevant code first, then make the change.`;
}
