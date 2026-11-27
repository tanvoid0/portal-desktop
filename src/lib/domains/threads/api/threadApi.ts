import { invokeClient } from '$lib/utils/invokeClient';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ThreadDeltaFrame,
  ThreadEventFrame,
  ThreadEventsPage,
  ThreadSnapshot,
  ThreadSummary,
} from '../generated';

/** Live-only; `seq` is the last committed seq, like a delta's. */
export interface ThreadHeartbeatFrame {
  thread_id: string;
  seq: number;
  waited_seconds: number;
}

/** Read side of the thread log (`coder_thread_*`); every provider's threads, one cap. Writes are S4. */
export const threadApi = {
  list(): Promise<ThreadSummary[]> {
    return invokeClient.post<ThreadSummary[]>('coder_thread_list');
  },

  /** `null`: no such thread. */
  snapshot(threadId: string): Promise<ThreadSnapshot | null> {
    return invokeClient.post<ThreadSnapshot | null>('coder_thread_snapshot', { threadId });
  },

  eventsSince(threadId: string, seq: number): Promise<ThreadEventsPage> {
    return invokeClient.post<ThreadEventsPage>('coder_thread_events_since', { threadId, seq });
  },

  onEvent(cb: (f: ThreadEventFrame) => void): Promise<UnlistenFn> {
    return listen<ThreadEventFrame>('coder://thread-event', (e) => cb(e.payload));
  },
  onDelta(cb: (f: ThreadDeltaFrame) => void): Promise<UnlistenFn> {
    return listen<ThreadDeltaFrame>('coder://thread-delta', (e) => cb(e.payload));
  },
  onHeartbeat(cb: (f: ThreadHeartbeatFrame) => void): Promise<UnlistenFn> {
    return listen<ThreadHeartbeatFrame>('coder://thread-heartbeat', (e) => cb(e.payload));
  },
  /** Socket-wide: this client missed frames, so every open thread must catch up. */
  onLagged(cb: () => void): Promise<UnlistenFn> {
    return listen('events://lagged', () => cb());
  },
  /** The bridge's status; a `connected` one is a reconnect to catch up after. */
  onConnection(cb: (connected: boolean) => void): Promise<UnlistenFn> {
    return listen<{ connected: boolean }>('mobile://connection', (e) => cb(e.payload.connected));
  },
};
