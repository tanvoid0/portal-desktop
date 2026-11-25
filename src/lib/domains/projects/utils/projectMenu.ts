import { goto } from '$app/navigation';
import FaEye from '~icons/fa6-solid/eye';
import FaStar from '~icons/fa6-solid/star';
import FaBoxArchive from '~icons/fa6-solid/box-archive';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaRobot from '~icons/fa6-solid/robot';
import FaBolt from '~icons/fa6-solid/bolt';
import FaUpload from '~icons/fa6-solid/upload';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';
import { copyText } from '$lib/domains/shared/utils/clipboard';
import type { Project } from '../types';

export interface ProjectMenuHandlers {
  onToggleStar?: (project: Project) => void;
  onSetArchived?: (project: Project, archived: boolean) => void;
  onSetPushOk?: (project: Project, pushOk: boolean) => void;
  onCopyReference?: (project: Project) => void;
  onLaunchCoder?: (project: Project) => void;
  onLaunchAgents?: (project: Project) => void;
  onDelete?: (project: Project) => void;
}

/** Copies the project's id; shared by the card and the row. */
export function copyProjectReference(project: Project): void {
  copyText(String(project.id), 'Project ID copied');
}

/** Right-click actions for a project card/row. Items without a handler are dropped. */
export function buildProjectMenu(project: Project, h: ProjectMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [
    { label: 'Open', icon: FaEye, onSelect: () => goto(`/projects/${project.id}`) },
  ];

  if (h.onToggleStar) {
    items.push({
      label: project.starred ? 'Unstar' : 'Star',
      icon: FaStar,
      onSelect: () => h.onToggleStar!(project),
    });
  }
  if (h.onSetArchived) {
    items.push({
      label: project.archived ? 'Unarchive' : 'Archive',
      icon: FaBoxArchive,
      onSelect: () => h.onSetArchived!(project, !project.archived),
    });
  }
  if (h.onSetPushOk) {
    items.push({
      label: project.push_ok ? 'Block push in Sync all' : 'Allow push in Sync all',
      icon: FaUpload,
      onSelect: () => h.onSetPushOk!(project, !project.push_ok),
    });
  }

  if (h.onCopyReference) {
    items.push({
      label: 'Copy ID',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: () => h.onCopyReference!(project),
    });
  }
  if (h.onLaunchCoder) {
    items.push({
      label: 'Open in Coder',
      icon: FaRobot,
      separatorBefore: !h.onCopyReference,
      onSelect: () => h.onLaunchCoder!(project),
    });
  }
  if (h.onLaunchAgents) {
    items.push({
      label: 'Run agent here',
      icon: FaBolt,
      separatorBefore: !h.onCopyReference && !h.onLaunchCoder,
      onSelect: () => h.onLaunchAgents!(project),
    });
  }

  if (h.onDelete) {
    items.push({
      label: 'Delete',
      icon: FaTrash,
      variant: 'destructive',
      separatorBefore: true,
      onSelect: () => h.onDelete!(project),
    });
  }

  return items;
}
