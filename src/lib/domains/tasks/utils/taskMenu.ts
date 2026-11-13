import FaEye from '~icons/fa6-solid/eye';
import FaListCheck from '~icons/fa6-solid/list-check';
import FaPlus from '~icons/fa6-solid/plus';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaRobot from '~icons/fa6-solid/robot';
import FaPen from '~icons/fa6-solid/pen';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';
import { TASK_STATUS_OPTIONS, type Task, type TaskStatus } from '../types';

export interface TaskMenuHandlers {
  onOpen?: (task: Task) => void;
  onSetStatus?: (taskId: string, status: TaskStatus) => void;
  onCreateSubtask?: (task: Task) => void;
  onCopyReference?: (task: Task) => void;
  onLaunchCoder?: (task: Task) => void;
  onEdit?: (task: Task) => void;
  onDelete?: (task: Task) => void;
}

/** Right-click actions for a task card/row. Items without a handler are dropped,
 *  so a board can offer the full set and another only a subset. */
export function buildTaskMenu(task: Task, h: TaskMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [];

  if (h.onOpen) items.push({ label: 'Open', icon: FaEye, onSelect: () => h.onOpen!(task) });

  if (h.onSetStatus) {
    items.push({
      label: 'Set status',
      icon: FaListCheck,
      submenu: TASK_STATUS_OPTIONS.map((o) => ({
        label: o.label,
        disabled: task.status === o.value,
        onSelect: () => h.onSetStatus!(task.id, o.value),
      })),
    });
  }

  if (h.onCreateSubtask && !task.parentId) {
    items.push({ label: 'Create subtask', icon: FaPlus, onSelect: () => h.onCreateSubtask!(task) });
  }

  if (h.onCopyReference) {
    items.push({
      label: 'Copy reference',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: () => h.onCopyReference!(task),
    });
  }
  if (h.onLaunchCoder) {
    items.push({
      label: 'Open in Coder',
      icon: FaRobot,
      separatorBefore: !h.onCopyReference,
      onSelect: () => h.onLaunchCoder!(task),
    });
  }

  if (h.onEdit) {
    items.push({
      label: 'Edit',
      icon: FaPen,
      separatorBefore: true,
      onSelect: () => h.onEdit!(task),
    });
  }
  if (h.onDelete) {
    items.push({
      label: 'Delete',
      icon: FaTrash,
      variant: 'destructive',
      separatorBefore: !h.onEdit,
      onSelect: () => h.onDelete!(task),
    });
  }

  return items;
}
