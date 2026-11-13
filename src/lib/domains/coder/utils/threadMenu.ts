import FaEye from '~icons/fa6-solid/eye';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaRobot from '~icons/fa6-solid/robot';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';
import type { CoderThread } from '../types';

export interface CoderThreadMenuHandlers {
  onOpen?: (thread: CoderThread) => void;
  onCopyReference?: (thread: CoderThread) => void;
  onLaunchWorkspace?: (thread: CoderThread) => void;
  onDelete?: (thread: CoderThread) => void;
}

/** Right-click actions for a coder session row. Items without a handler are dropped. */
export function buildCoderThreadMenu(thread: CoderThread, h: CoderThreadMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [];

  if (h.onOpen) items.push({ label: 'Open', icon: FaEye, onSelect: () => h.onOpen!(thread) });
  if (h.onCopyReference) {
    items.push({
      label: 'Copy reference',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: () => h.onCopyReference!(thread),
    });
  }
  if (h.onLaunchWorkspace) {
    items.push({
      label: 'Open workspace in Coder',
      icon: FaRobot,
      onSelect: () => h.onLaunchWorkspace!(thread),
    });
  }
  if (h.onDelete) {
    items.push({
      label: 'Delete',
      icon: FaTrash,
      variant: 'destructive',
      separatorBefore: true,
      onSelect: () => h.onDelete!(thread),
    });
  }

  return items;
}
