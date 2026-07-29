const STORAGE_KEY = 'portal.ai.chatFolders';

/**
 * Folders are a local grouping aid, not thread data — `ai_conversations` has no
 * column for them, so this lives in localStorage rather than forcing a
 * migration. Clearing site data loses the grouping, not the conversations.
 */
export interface ChatFolders {
  /** Folder names, in display order. */
  names: string[];
  /** conversation id → folder name. Ids in no folder are simply absent. */
  assignments: Record<string, string>;
}

export const EMPTY_CHAT_FOLDERS: ChatFolders = { names: [], assignments: {} };

export function loadChatFolders(): ChatFolders {
  if (typeof localStorage === 'undefined') return { ...EMPTY_CHAT_FOLDERS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...EMPTY_CHAT_FOLDERS };
    const parsed = JSON.parse(raw) as Partial<ChatFolders>;
    return {
      names: Array.isArray(parsed.names) ? parsed.names : [],
      assignments:
        parsed.assignments && typeof parsed.assignments === 'object' ? parsed.assignments : {},
    };
  } catch {
    return { ...EMPTY_CHAT_FOLDERS };
  }
}

export function saveChatFolders(folders: ChatFolders): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(folders));
  } catch {
    // ignore quota / private mode
  }
}

/** Appends a folder, de-duplicating by name. Returns a new value. */
export function addFolder(folders: ChatFolders, name: string): ChatFolders {
  const trimmed = name.trim();
  if (!trimmed || folders.names.includes(trimmed)) return folders;
  return { ...folders, names: [...folders.names, trimmed] };
}

/** Drops a folder; its conversations fall back to the ungrouped list. */
export function removeFolder(folders: ChatFolders, name: string): ChatFolders {
  const assignments = Object.fromEntries(
    Object.entries(folders.assignments).filter(([, folder]) => folder !== name)
  );
  return { names: folders.names.filter((n) => n !== name), assignments };
}

/** Moves a conversation into `name`, or out of any folder when null. */
export function assignFolder(
  folders: ChatFolders,
  conversationId: string,
  name: string | null
): ChatFolders {
  const assignments = { ...folders.assignments };
  if (name) {
    assignments[conversationId] = name;
  } else {
    delete assignments[conversationId];
  }
  return { ...folders, assignments };
}

export interface FolderGroup<T> {
  /** null for the ungrouped bucket rendered after the folders. */
  name: string | null;
  items: T[];
}

/**
 * Buckets conversations by folder, preserving folder order and keeping empty
 * folders so they can render their own empty state.
 */
export function groupByFolder<T extends { id: string }>(
  items: T[],
  folders: ChatFolders
): FolderGroup<T>[] {
  const groups: FolderGroup<T>[] = folders.names.map((name) => ({
    name,
    items: [],
  }));
  const byName = new Map(groups.map((g) => [g.name as string, g]));
  const ungrouped: T[] = [];

  for (const item of items) {
    const group = byName.get(folders.assignments[item.id] ?? '');
    if (group) group.items.push(item);
    else ungrouped.push(item);
  }

  return [...groups, { name: null, items: ungrouped }];
}
