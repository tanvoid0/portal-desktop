import FaPlay from '~icons/fa6-solid/play';
import FaStop from '~icons/fa6-solid/stop';
import FaArrowUpRightFromSquare from '~icons/fa6-solid/arrow-up-right-from-square';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';

export interface DeploymentMenuHandlers {
  onStart?: () => void;
  onStop?: () => void;
  onOpenUrl?: () => void;
  onCopyReference?: () => void;
  onDelete?: () => void;
}

/** Right-click actions for a deployment/workload. Items without a handler are dropped. */
export function buildDeploymentMenu(h: DeploymentMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [];
  if (h.onStart) items.push({ label: 'Start', icon: FaPlay, onSelect: h.onStart });
  if (h.onStop) items.push({ label: 'Stop', icon: FaStop, onSelect: h.onStop });
  if (h.onOpenUrl) {
    items.push({ label: 'Open URL', icon: FaArrowUpRightFromSquare, onSelect: h.onOpenUrl });
  }
  if (h.onCopyReference) {
    items.push({
      label: 'Copy reference',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: h.onCopyReference,
    });
  }
  if (h.onDelete) {
    items.push({
      label: 'Delete',
      icon: FaTrash,
      variant: 'destructive',
      separatorBefore: true,
      onSelect: h.onDelete,
    });
  }
  return items;
}
