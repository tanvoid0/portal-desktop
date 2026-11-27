import FaEye from '~icons/fa6-solid/eye';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaLink from '~icons/fa6-solid/link';
import FaRobot from '~icons/fa6-solid/robot';
import FaTrash from '~icons/fa6-solid/trash';
import FaStar from '~icons/fa6-solid/star';
import FaArrowUp from '~icons/fa6-solid/arrow-up';
import FaArrowDown from '~icons/fa6-solid/arrow-down';
import FaAnglesUp from '~icons/fa6-solid/angles-up';
import FaRotateLeft from '~icons/fa6-solid/rotate-left';
import FaChevronRight from '~icons/fa6-solid/chevron-right';
import FaFolderPlus from '~icons/fa6-solid/folder-plus';
import type { MenuItem } from '$lib/components/menu/types';
import { formatCount } from '$lib/domains/shared/utils';
import type { CoderThread } from '../types';

export interface WorkspaceMenuHandlers {
  starred: boolean;
  expanded: boolean;
  onToggleStar: () => void;
  onToggle: () => void;
  onMoveTop?: () => void;
  onMoveUp?: () => void;
  onMoveDown?: () => void;
  onResetOrder?: () => void;
  onOpenProject?: () => void;
  onAddProject?: () => void;
  onCopyPath?: () => void;
  onDeleteHistory?: () => void;
}

/** Right-click actions for a workspace header. Items without a handler are dropped. */
export function buildWorkspaceMenu(h: WorkspaceMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [
    { label: h.starred ? 'Unstar' : 'Star', icon: FaStar, onSelect: h.onToggleStar },
    { label: h.expanded ? 'Collapse' : 'Expand', icon: FaChevronRight, onSelect: h.onToggle },
  ];
  const group = (
    entries: Array<[string, MenuItem['icon'], (() => void) | undefined]>,
    variant?: MenuItem['variant']
  ) => {
    let first = true;
    for (const [label, icon, onSelect] of entries) {
      if (!onSelect) continue;
      items.push({ label, icon, onSelect, variant, separatorBefore: first });
      first = false;
    }
  };
  group([
    ['Move to top', FaAnglesUp, h.onMoveTop],
    ['Move up', FaArrowUp, h.onMoveUp],
    ['Move down', FaArrowDown, h.onMoveDown],
    ['Reset order', FaRotateLeft, h.onResetOrder],
  ]);
  group([
    ['Open project page', FaEye, h.onOpenProject],
    ['Add as project', FaFolderPlus, h.onAddProject],
    ['Copy path', FaClipboard, h.onCopyPath],
  ]);
  group([['Delete chat history…', FaTrash, h.onDeleteHistory]], 'destructive');
  return items;
}

/** Confirm text for deleting a workspace's sessions; `kept` are the running ones left alone. */
export function deleteHistoryMessage(count: number, label: string, kept: number): string {
  const skipped = kept ? ` ${formatCount(kept, 'running session')} will be kept.` : '';
  return `Delete the whole chat history of "${label}": ${formatCount(count, 'session')} with transcripts, including any hidden by search or filters? Files in the folder are not touched. This can't be undone.${skipped}`;
}

export interface CoderThreadMenuHandlers {
  onOpen?: (thread: CoderThread) => void;
  starred?: boolean;
  onToggleStar?: (thread: CoderThread) => void;
  onCopyReference?: (thread: CoderThread) => void;
  onCopyLink?: (thread: CoderThread) => void;
  onLaunchWorkspace?: (thread: CoderThread) => void;
  onDelete?: (thread: CoderThread) => void;
}

/** Right-click actions for a coder session row. Items without a handler are dropped. */
export function buildCoderThreadMenu(thread: CoderThread, h: CoderThreadMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [];

  if (h.onOpen) items.push({ label: 'Open', icon: FaEye, onSelect: () => h.onOpen!(thread) });
  if (h.onToggleStar) {
    items.push({
      label: h.starred ? 'Unstar' : 'Star',
      icon: FaStar,
      onSelect: () => h.onToggleStar!(thread),
    });
  }
  if (h.onCopyReference) {
    items.push({
      label: 'Copy ID',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: () => h.onCopyReference!(thread),
    });
  }
  if (h.onCopyLink) {
    items.push({
      label: 'Copy link',
      icon: FaLink,
      separatorBefore: !h.onCopyReference,
      onSelect: () => h.onCopyLink!(thread),
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
