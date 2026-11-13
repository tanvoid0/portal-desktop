import FaEye from '~icons/fa6-solid/eye';
import FaPen from '~icons/fa6-solid/pen-to-square';
import FaDownload from '~icons/fa6-solid/download';
import FaCopy from '~icons/fa6-solid/copy';
import FaClipboard from '~icons/fa6-solid/clipboard';
import FaTrash from '~icons/fa6-solid/trash';
import type { MenuItem } from '$lib/components/menu/types';
import type { Block } from '$lib/domains/projects/pipelines';

export interface ScriptMenuHandlers {
  onView?: (script: Block) => void;
  onEdit?: (script: Block) => void;
  onExport?: (script: Block) => void;
  onCopyCommand?: (script: Block) => void;
  onCopyReference?: (script: Block) => void;
  onDelete?: (script: Block) => void;
}

/** Right-click actions for a script card. Items without a handler are dropped. */
export function buildScriptMenu(script: Block, h: ScriptMenuHandlers): MenuItem[] {
  const items: MenuItem[] = [];
  if (h.onView) items.push({ label: 'View', icon: FaEye, onSelect: () => h.onView!(script) });
  if (h.onEdit) items.push({ label: 'Edit', icon: FaPen, onSelect: () => h.onEdit!(script) });
  if (h.onExport) {
    items.push({ label: 'Export', icon: FaDownload, onSelect: () => h.onExport!(script) });
  }
  if (h.onCopyCommand) {
    items.push({ label: 'Copy command', icon: FaCopy, onSelect: () => h.onCopyCommand!(script) });
  }
  if (h.onCopyReference) {
    items.push({
      label: 'Copy reference',
      icon: FaClipboard,
      separatorBefore: true,
      onSelect: () => h.onCopyReference!(script),
    });
  }
  if (h.onDelete) {
    items.push({
      label: 'Delete',
      icon: FaTrash,
      variant: 'destructive',
      separatorBefore: true,
      onSelect: () => h.onDelete!(script),
    });
  }
  return items;
}
