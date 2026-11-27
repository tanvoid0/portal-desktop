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
      const mine = s;
      this.listen().then(
        () => this.resnapshot(id),
        (e) => {
          console.error('threads: listen failed', e);
          if (this.subs.get(id) === mine) this.subs.delete(id);
        }
      );
    }
    const mine = s;
    return () => {
      if (this.subs.get(id) !== mine || --mine.refs > 0) return;
      this.subs.delete(id);
      const { [id]: _gone, ...rest } = this.views;
      this.views = rest;
    };
  }

  // ponytail: listeners live for the app's lifetime (like agentRuns); frames of unsubscribed
  // threads are dropped at `subs.get`. Tear down when the last thread closes if that ever costs.
  private listen(): Promise<void> {
    this.listening ??= (async () => {
      const results = await Promise.allSettled([
        threadApi.onEvent((event) => this.feed(event.thread_id, { event })),
        threadApi.onDelta((delta) => this.feed(delta.thread_id, { delta })),
        threadApi.onHeartbeat((hb) => this.heartbeat(hb)),
        threadApi.onReset((id) => void this.resnapshot(id)), // no-op when not subscribed
        threadApi.onLagged(() => this.catchUpAll()),
        threadApi.onConnection((connected) => connected && this.catchUpAll()),
      ]);
      const failed = results.find((r) => r.status === 'rejected');
      if (failed) {
        // Drop the ones that did register, or a retry would double them.
        for (const r of results) if (r.status === 'fulfilled') r.value();
        throw (failed as PromiseRejectedResult).reason;
      }
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

  /** Writes only into the subscription `s` that started the work, not a newer one under the same id. */
  private set(id: string, s: Sub, v: ThreadView) {
    if (this.subs.get(id) === s) this.views = { ...this.views, [id]: v };
  }

  private feed(id: string, item: Item) {
    const s = this.subs.get(id);
    if (!s) return;
    if (s.ready) this.handle(id, s, item);
    else s.buffer.push(item);
  }

  private handle(id: string, s: Sub, item: Item) {
    const v = this.views[id];
    if (!v) {
      // No snapshot yet (it failed, or the thread is not created). Never fold from empty: an
      // imported thread has a projection at seq 0 with no events. A live frame retries the snapshot.
      if ('event' in item && !s.snapping) void this.resnapshot(id);
      return;
    }
    if ('event' in item) {
      const n = applyEvent(v, item.event);
      if (n === null) {
        s.buffer.push(item);
        void this.catchUp(id);
      } else if (n !== v) this.set(id, s, n);
    } else {
      const n = applyDelta(v, item.delta);
      if (n === null) void this.resnapshot(id);
      else if (n !== v) this.set(id, s, n);
    }
  }

  private heartbeat(hb: ThreadHeartbeatFrame) {
    const s = this.subs.get(hb.thread_id);
    const v = this.views[hb.thread_id];
    if (s?.ready && v && hb.seq === v.seq)
      this.set(hb.thread_id, s, { ...v, waitingSeconds: hb.waited_seconds });
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
      let missing = false; // the hub answered "no such thread", as opposed to an error
      try {
        const snap = await threadApi.snapshot(id);
        missing = !snap;
        if (snap) this.set(id, s, fromSnapshot(snap)); // buffered frames <= snap.seq drop in `applyEvent`
      } catch (e) {
        console.error('threads: snapshot failed', id, e);
      } finally {
        s.snapping = null;
        s.ready = true;
        if (this.views[id]) this.replay(id, s);
        else {
          // The next snapshot is requested after these frames arrived, so it covers them. A
          // `created` that raced a "not found" answer gets one retry.
          const raced = s.buffer.some((i) => 'event' in i);
          s.buffer = [];
          if (missing && raced) void this.resnapshot(id);
        }
      }
    })();
    return s.snapping;
  }

  private async catchUp(id: string): Promise<void> {
    const s = this.subs.get(id);
    if (!s || s.catching || s.snapping) return;
    if (!this.views[id]) return void this.resnapshot(id);
    s.catching = true;
    try {
      const page = await threadApi.eventsSince(id, this.views[id].seq);
      // A snapshot started meanwhile owns the buffer and will overwrite the view; stand down.
      if (s.snapping || this.subs.get(id) !== s) return;
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
    for (const [id, s] of this.subs)
      if (s.ready) void (this.views[id] ? this.catchUp(id) : this.resnapshot(id));
    void queryClient.invalidateQueries({ queryKey: queryKeys.threads.list });
  }
}

export const threadStore = new ThreadStore();
