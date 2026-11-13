import { goto } from '$app/navigation';
import FaEye from '~icons/fa6-solid/eye';
import FaStar from '~icons/fa6-solid/star';
import FaBoxArchive from '~icons/fa6-solid/box-archive';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaRobot from '~icons/fa6-solid/robot';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';
import { buildReference } from '$lib/domains/shared/utils/entityReference';
import { copyText } from '$lib/domains/shared/utils/clipboard';
import type { Project } from '../types';

export interface ProjectMenuHandlers {
  onToggleStar?: (project: Project) => void;
  onSetArchived?: (project: Project, archived: boolean) => void;
  onCopyReference?: (project: Project) => void;
  onLaunchCoder?: (project: Project) => void;
  onDelete?: (project: Project) => void;
}

/** Copies a markdown reference to the project; shared by the card and the row. */
export function copyProjectReference(project: Project, branch?: string | null): void {
  copyText(
    buildReference({
      kind: 'project',
      id: project.id,
      title: project.name,
      fields: {
        Stage: project.stage ?? undefined,
        Branch: branch ?? undefined,
        Path: project.path,
      },
      body: project.description ?? undefined,
    }),
    'Project reference copied'
  );
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

  if (h.onCopyReference) {
    items.push({
      label: 'Copy reference',
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
