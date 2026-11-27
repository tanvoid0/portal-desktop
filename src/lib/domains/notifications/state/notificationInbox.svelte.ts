/** History behind `notify()`: what fired, and whether the user has seen it.
 * ponytail: localStorage, newest 200, per webview — move into portal-core if a
 * phone or the daemon ever has to share the same list. */
import type { Toast } from '$lib/domains/shared/stores/toastStore';

export type NotificationSource = 'agent' | 'pipeline' | 'device';
export type FactKind = 'tool' | 'command' | 'file' | 'step' | 'exit' | 'duration' | 'device';

/** What a notification is about. Ids are looked up when the list renders (so the
 *  folder, project name and "still waiting?" are current); `facts` are snapshots. */
export interface NotificationContext {
  source?: NotificationSource;
  /** Agent session. */
  runId?: string;
  /** Approval this notification asked for. */
  requestId?: string;
  projectId?: string;
  facts?: { kind: FactKind; value: string }[];
}

export interface InboxItem extends NotificationContext {
  id: string;
  title: string;
  body?: string;
  type: NonNullable<Toast['type']>;
  route?: string;
  at: number;
  read: boolean;
}

const STORAGE_KEY = 'portal-notifications';
export const MAX_ITEMS = 200;

function load(): InboxItem[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]');
    // One bad entry (no id) would break the page's keyed list for good.
    return Array.isArray(parsed)
      ? parsed.filter((i) => typeof i?.id === 'string' && typeof i.at === 'number')
      : [];
  } catch {
    return [];
  }
}

class NotificationInbox {
  items = $state<InboxItem[]>(load());

  get unread(): number {
    return this.items.filter((i) => !i.read).length;
  }

  add(n: Omit<InboxItem, 'id' | 'at' | 'read'>): void {
    const item = { ...n, id: Math.random().toString(36).slice(2, 11), at: Date.now(), read: false };
    this.#set([item, ...this.items].slice(0, MAX_ITEMS));
  }

  markRead(id: string): void {
    this.#set(this.items.map((i) => (i.id === id ? { ...i, read: true } : i)));
  }

  /** The user went to `route`, so everything pointing there has been seen. */
  markRouteRead(route: string): void {
    this.#set(this.items.map((i) => (i.route === route ? { ...i, read: true } : i)));
  }

  markAllRead(): void {
    this.#set(this.items.map((i) => ({ ...i, read: true })));
  }

  remove(id: string): void {
    this.#set(this.items.filter((i) => i.id !== id));
  }

  clear(): void {
    this.#set([]);
  }

  #set(items: InboxItem[]): void {
    this.items = items;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
    } catch {
      // Storage full or unavailable: history still works for this session.
    }
  }
}

export const notificationInbox = new NotificationInbox();
