import { beforeEach, describe, expect, it } from 'vitest';
import { MAX_ITEMS, notificationInbox } from './notificationInbox.svelte';

describe('notificationInbox', () => {
  beforeEach(() => notificationInbox.clear());

  it('counts unread newest-first and marks read by id, route, or all', () => {
    notificationInbox.add({ title: 'a', type: 'info', route: '/x' });
    notificationInbox.add({ title: 'b', type: 'info', route: '/x' });
    notificationInbox.add({ title: 'c', type: 'error' });
    expect(notificationInbox.items.map((i) => i.title)).toEqual(['c', 'b', 'a']);
    expect(notificationInbox.unread).toBe(3);

    notificationInbox.markRouteRead('/x');
    expect(notificationInbox.unread).toBe(1);

    notificationInbox.markRead(notificationInbox.items[0]!.id);
    expect(notificationInbox.unread).toBe(0);

    notificationInbox.add({ title: 'd', type: 'info' });
    notificationInbox.markAllRead();
    expect(notificationInbox.unread).toBe(0);
  });

  it('drops the oldest past the cap', () => {
    for (let n = 0; n <= MAX_ITEMS; n++) notificationInbox.add({ title: `${n}`, type: 'info' });
    expect(notificationInbox.items).toHaveLength(MAX_ITEMS);
    expect(notificationInbox.items.at(-1)!.title).toBe('1');
  });
});
