import type { GeneratedTaskStructure } from '../services/aiTaskService';
import type { TaskPriority, TaskStatus } from '../types';

export interface OriginalTaskState {
  status?: TaskStatus;
  priority?: TaskPriority;
  tags?: string[];
  type?: string;
  estimatedTime?: number;
}

/** When refining an existing task, fall back to its original fields wherever the AI left them empty. */
export function mergeWithOriginal(
  result: GeneratedTaskStructure,
  original: OriginalTaskState | null
): GeneratedTaskStructure {
  if (!original) return result;
  return {
    ...result,
    main_task: {
      ...result.main_task,
      priority: result.main_task.priority || original.priority || 'medium',
      type_: result.main_task.type_ || original.type || '',
      estimated_time: result.main_task.estimated_time || original.estimatedTime || null,
    },
    suggested_labels:
      result.suggested_labels.length > 0 ? result.suggested_labels : original.tags || [],
  };
}
