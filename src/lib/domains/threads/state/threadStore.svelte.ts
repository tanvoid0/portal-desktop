/**
 * Live thread views, one per subscribed thread, for every provider. The view is the pure
 * fold in `utils/reducer.ts`; this class only decides which frame to feed it:
 * listen and buffer, snapshot, replay the buffer, then per frame
 * `seq == cursor+1` applies, `<= cursor` drops, `> cursor+1` buffers and catches up
 * (single-flight `events_since`). Resync on `events://lagged`, bridge reconnect and
 * `visibilitychange`. Design: docs/development/CODER_THREAD_LOG_DESIGN.md section 4.
 */
import { threadApi, type ThreadHeartbeatFrame } from '../api/threadApi';
import { applyDelta, applyEvent, fromSnapshot, type ThreadView } from '../utils/reducer';
import type { ThreadDeltaFrame, ThreadEventFrame } from '../generated';
import { queryClient } from '$lib/domains/shared/query/queryClient';
import { queryKeys } from '$lib/domains/shared/query/keys';

type Item = { event: ThreadEventFrame } | { delta: ThreadDeltaFrame };

interface Sub {
  refs: number;
  /** False from subscribe until the snapshot is in: frames wait in `buffer`. */
  ready: boolean;
  buffer: Item[];
  snapping: Promise<void> | null;
  catching: boolean;
}

class ThreadStore {
  /** Replaced, never mutated, so `$state.raw`: no deep proxy over long transcripts. */
  private views = $state.raw<Record<string, ThreadView>>({});
  private subs = new Map<string, Sub>();
  private listening: Promise<void> | null = null;

  /** Null until the first snapshot lands (or the thread does not exist). */
  view(id: string): ThreadView | null {
    return this.views[id] ?? null;
  }

  /** Ref-counted; returns the unsubscribe. The last unsubscribe drops the view. */
  subscribe(id: string): () => void {
    let s = this.subs.get(id);
    if (s) s.refs++;
    else {
      s = { refs: 1, ready: false, buffer: [], snapping: null, catching: false };
      this.subs.set(id, s);
      void this.listen().then(() => this.resnapshot(id));
    }
    return () => {
      const cur = this.subs.get(id);
      if (!cur || --cur.refs > 0) return;
      this.subs.delete(id);
      const { [id]: _gone, ...rest } = this.views;
      this.views = rest;
    };
  }

  // ponytail: listeners live for the app's lifetime (like agentRuns); frames of unsubscribed
  // threads are dropped at `subs.get`. Tear down when the last thread closes if that ever costs.
  private listen(): Promise<void> {
    this.listening ??= (async () => {
      await Promise.all([
        threadApi.onEvent((event) => this.feed(event.thread_id, { event })),
        threadApi.onDelta((delta) => this.feed(delta.thread_id, { delta })),
        threadApi.onHeartbeat((hb) => this.heartbeat(hb)),
        threadApi.onLagged(() => this.catchUpAll()),
        threadApi.onConnection((connected) => connected && this.catchUpAll()),
      ]);
      if (typeof document !== 'undefined') {
        document.addEventListener('visibilitychange', () => {
          if (document.visibilityState === 'visible') this.catchUpAll();
        });
      }
    })().catch((e) => {
      this.listening = null;
      throw e;
    });
    return this.listening;
  }

  private set(id: string, v: ThreadView) {
    if (this.subs.has(id)) this.views = { ...this.views, [id]: v };
  }

  private feed(id: string, item: Item) {
    const s = this.subs.get(id);
    if (!s) return;
    if (s.ready) this.handle(id, s, item);
    else s.buffer.push(item);
  }

  private handle(id: string, s: Sub, item: Item) {
    const v = this.views[id];
    if (!v) return;
    if ('event' in item) {
      const n = applyEvent(v, item.event);
      if (n === null) {
        s.buffer.push(item);
        void this.catchUp(id);
      } else if (n !== v) this.set(id, n);
    } else {
      const n = applyDelta(v, item.delta);
      if (n === null) void this.resnapshot(id);
      else if (n !== v) this.set(id, n);
    }
  }

  private heartbeat(hb: ThreadHeartbeatFrame) {
    const s = this.subs.get(hb.thread_id);
    const v = this.views[hb.thread_id];
    if (s?.ready && v) this.set(hb.thread_id, { ...v, waitingSeconds: hb.waited_seconds });
  }

  /** Re-feeds the buffer; what is still a gap lands back in it. */
  private replay(id: string, s: Sub) {
    const b = s.buffer;
    s.buffer = [];
    for (const item of b) this.handle(id, s, item);
  }

  /** Single-flight: a second caller shares the first one's result. */
  private resnapshot(id: string): Promise<void> {
    const s = this.subs.get(id);
    if (!s) return Promise.resolve();
    s.ready = false;
    s.snapping ??= (async () => {
      try {
        const snap = await threadApi.snapshot(id);
        if (snap) this.set(id, fromSnapshot(snap)); // buffered frames <= snap.seq drop in `applyEvent`
      } catch (e) {
        console.error('threads: snapshot failed', id, e);
      } finally {
        s.snapping = null;
        s.ready = true;
        this.replay(id, s);
      }
    })();
    return s.snapping;
  }

  private async catchUp(id: string): Promise<void> {
    const s = this.subs.get(id);
    const v = this.views[id];
    if (!s || !v || s.catching || s.snapping) return;
    s.catching = true;
    try {
      const page = await threadApi.eventsSince(id, v.seq);
      if (page.reset) return await this.resnapshot(id);
      for (const event of page.events) this.handle(id, s, { event });
      this.replay(id, s);
      // Everything the log holds is in; a frame still ahead of the cursor means we are lost.
      if (s.buffer.some((i) => 'event' in i)) await this.resnapshot(id);
    } catch (e) {
      console.error('threads: catch-up failed', id, e);
    } finally {
      s.catching = false;
    }
  }

  private catchUpAll() {
    for (const [id, s] of this.subs) if (s.ready) void this.catchUp(id);
    void queryClient.invalidateQueries({ queryKey: queryKeys.threads.list });
  }
}

export const threadStore = new ThreadStore();
