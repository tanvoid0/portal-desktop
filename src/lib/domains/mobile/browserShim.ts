/**
 * A browser (or other-desktop) `window.__TAURI_INTERNALS__` so the unmodified
 * mobile frontend (`invoke`/`listen` from `@tauri-apps/api`) runs against the
 * daemon directly, with no Rust shell in between — the web-client counterpart
 * to `mobile/src-tauri/src/lib.rs`. Mirror its behaviour, don't redesign it:
 *
 * - `mobile_status`/`mobile_pair`/`mobile_connect`/`mobile_disconnect` behave
 *   the same as the Rust version, storing `{host, port, token, name}` in
 *   `localStorage` instead of a file (a browser has no filesystem).
 *   `mobile_discover` always returns `[]` — no UDP from a browser; manual
 *   host/port entry on `/connect` already covers it.
 * - Everything else forwards to `POST /rpc/{command}` with a bearer token,
 *   except the two `plugin:event|*` commands `@tauri-apps/api/event`'s
 *   `listen`/`_unlisten` issue, which are served locally against the
 *   `/events` WebSocket bridge below.
 * - Tauri commands reject with a plain string (`Result<_, String>`), not an
 *   `Error` — `$lib/utils/errorMessage.ts` already accounts for this, so the
 *   shim matches it rather than rejecting with `Error` objects.
 */
import { isMobileShell } from './shell';
import { isForwardableCommand } from './commandName';

const STORAGE_KEY = 'portal-client-connection';

interface StoredConn {
  host: string;
  port: number;
  token: string;
  name: string;
}

interface MobileStatus {
  connected: boolean;
  host: string | null;
  port: number | null;
  name: string | null;
  caps: string[];
}

function loadConn(): StoredConn | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as StoredConn) : null;
  } catch {
    return null;
  }
}

function saveConn(conn: StoredConn | null): void {
  try {
    if (conn) localStorage.setItem(STORAGE_KEY, JSON.stringify(conn));
    else localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Private-mode/quota: the connection just won't survive a reload.
  }
}

/** `[host]:port` for an IPv6 literal, `host:port` otherwise — mirrors Rust's `hostport()`. */
function hostport(host: string, port: number): string {
  return host.includes(':') && !host.startsWith('[') ? `[${host}]:${port}` : `${host}:${port}`;
}

/** Non-2xx `/rpc` or `/whoami` reply text — the daemon's own body, verbatim. Mirrors `rpc_error()`. */
function rpcError(status: number, body: string): string {
  return body.trim() ? body : `daemon answered HTTP ${status}`;
}

/** Mirrors Rust's `pair_error()`. */
function pairError(status: number): string {
  switch (status) {
    case 403:
      return 'pairing was denied on the desktop';
    case 408:
      return 'nobody answered the pairing request on the desktop in time';
    case 429:
      return 'too many pairing attempts; wait a minute and retry';
    default:
      return `pairing failed: desktop answered HTTP ${status}`;
  }
}

/** A `/whoami` 401: the token was revoked on the desktop. */
class Revoked extends Error {}

async function whoami(conn: StoredConn): Promise<{ name: string; caps: string[] }> {
  let res: Response;
  try {
    res = await fetch(`http://${hostport(conn.host, conn.port)}/whoami`, {
      headers: { Authorization: `Bearer ${conn.token}` },
    });
  } catch (e) {
    throw new Error(`cannot reach ${conn.host}:${conn.port}: ${String(e)}`);
  }
  const body = await res.text();
  if (!res.ok) {
    const msg = rpcError(res.status, body);
    throw res.status === 401 ? new Revoked(msg) : new Error(msg);
  }
  let v: { name?: string; caps?: string[] };
  try {
    v = JSON.parse(body);
  } catch (e) {
    throw new Error(`bad /whoami reply: ${String(e)}`);
  }
  return { name: v.name ?? 'Portal Desktop', caps: Array.isArray(v.caps) ? v.caps : [] };
}

/** Registers callbacks under the numeric ids `transformCallback` hands out — the
 * `window.__TAURI_INTERNALS__` half of `@tauri-apps/api/core`'s `Channel`/event plumbing. */
class CallbackRegistry {
  private next = 1;
  private readonly callbacks = new Map<number, (payload: unknown) => void>();
  private readonly once = new Set<number>();

  transformCallback(callback: (payload: unknown) => void, once = false): number {
    const id = this.next++;
    this.callbacks.set(id, callback);
    if (once) this.once.add(id);
    return id;
  }

  unregisterCallback(id: number): void {
    this.callbacks.delete(id);
    this.once.delete(id);
  }

  call(id: number, payload: unknown): void {
    const cb = this.callbacks.get(id);
    if (!cb) return;
    cb(payload);
    if (this.once.has(id)) this.unregisterCallback(id);
  }
}

class Bridge {
  private readonly registry = new CallbackRegistry();
  private readonly listenersByEvent = new Map<string, Set<number>>();
  private caps: string[] = [];
  private live = false;
  private generation = 0;
  private socket: WebSocket | null = null;

  /** `transformCallback(response)` → `{event, id, payload}`, same shape the real Tauri
   * webview delivers to a `listen()` handler (see `connection.svelte.ts`'s `e.payload`). */
  private emit(event: string, payload: unknown): void {
    const ids = this.listenersByEvent.get(event);
    if (!ids) return;
    for (const id of [...ids]) this.registry.call(id, { event, id, payload });
  }

  status(): MobileStatus {
    const conn = loadConn();
    return {
      connected: conn !== null && this.live,
      host: conn?.host ?? null,
      port: conn?.port ?? null,
      name: conn?.name ?? null,
      caps: this.caps,
    };
  }

  private setLive(live: boolean): void {
    if (live === this.live) return;
    this.live = live;
    this.emit('mobile://connection', this.status());
  }

  /** Stop the bridge and forget the connection — mirrors Rust's `Mobile::clear`. */
  clear(): void {
    this.generation++;
    this.socket?.close();
    this.socket = null;
    this.caps = [];
    this.live = false;
    saveConn(null);
    this.emit('mobile://connection', this.status());
  }

  /** Store `conn`, persist it, (re)start the events bridge — mirrors Rust's `Mobile::set`. */
  set(conn: StoredConn): void {
    this.generation++;
    this.socket?.close();
    this.socket = null;
    this.live = false;
    saveConn(conn);
    this.emit('mobile://connection', this.status());
    void this.run(conn, this.generation);
  }

  /** whoami + events socket, forever, with capped backoff. Mirrors Rust's `bridge()`/`run_socket()`.
   *
   * One gap vs. the Rust client: a browser `WebSocket` never exposes the handshake's HTTP
   * status, so a token revoked while the socket is open can't be told apart from an ordinary
   * drop at the moment it happens — both just retry. The very next retry's `whoami()` call
   * (still run before every reconnect) catches the revocation and clears, same as Rust, just
   * up to one backoff step later. ponytail: acceptable gap, no browser API closes it. */
  private async run(conn: StoredConn, gen: number): Promise<void> {
    let delay = 1000;
    while (gen === this.generation) {
      try {
        const who = await whoami(conn);
        if (gen !== this.generation) return; // a newer set()/clear() won overlapping this await
        this.caps = who.caps;
      } catch (e) {
        if (gen !== this.generation) return;
        if (e instanceof Revoked) return this.clear();
        this.setLive(false);
        delay = Math.min(delay * 2, 30_000);
        await new Promise((resolve) => setTimeout(resolve, delay));
        continue;
      }

      const closedCleanly = await this.runSocket(conn, gen);
      if (gen !== this.generation) return;
      this.setLive(false);
      delay = closedCleanly ? 1000 : Math.min(delay * 2, 30_000);
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }

  /** One WebSocket lifetime: resolves `true` once it opened at all (a connect failure
   * resolves `false`, so the backoff above only grows on a connection that never opened). */
  private runSocket(conn: StoredConn, gen: number): Promise<boolean> {
    return new Promise((resolve) => {
      const url = `ws://${hostport(conn.host, conn.port)}/events?token=${encodeURIComponent(conn.token)}`;
      let socket: WebSocket;
      try {
        socket = new WebSocket(url);
      } catch {
        return resolve(false); // bad URL / SecurityError — same as Rust's Stop::Other, just retry
      }
      this.socket = socket;
      let opened = false;
      socket.onopen = () => {
        if (gen !== this.generation) return socket.close();
        opened = true;
        this.setLive(true);
      };
      socket.onmessage = (ev) => {
        if (gen !== this.generation) return;
        try {
          const frame = JSON.parse(ev.data as string) as { event?: string; payload?: unknown };
          if (typeof frame.event === 'string') this.emit(frame.event, frame.payload);
        } catch {
          // Not a frame we understand — drop it.
        }
      };
      socket.onclose = () => {
        if (this.socket === socket) this.socket = null;
        resolve(opened);
      };
      socket.onerror = () => {
        /* onclose always follows; nothing to do here. */
      };
    });
  }

  listen(event: string, handlerId: number): number {
    let ids = this.listenersByEvent.get(event);
    if (!ids) {
      ids = new Set();
      this.listenersByEvent.set(event, ids);
    }
    ids.add(handlerId);
    return handlerId; // the eventId `_unlisten` is handed back later
  }

  unlisten(event: string, eventId: number): void {
    this.listenersByEvent.get(event)?.delete(eventId);
    this.registry.unregisterCallback(eventId);
  }

  transformCallback(callback: (payload: unknown) => void, once?: boolean): number {
    return this.registry.transformCallback(callback, once);
  }

  unregisterCallback(id: number): void {
    this.registry.unregisterCallback(id);
  }

  async pair(host: string, port: number, device: string): Promise<MobileStatus> {
    let res: Response;
    try {
      res = await fetch(`http://${hostport(host, port)}/pair/request`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ device }),
      });
    } catch (e) {
      throw new Error(`cannot reach ${host}:${port}: ${String(e)}`);
    }
    if (!res.ok) throw new Error(pairError(res.status));
    let v: { token?: string; name?: string; port?: number };
    try {
      v = JSON.parse(await res.text());
    } catch (e) {
      throw new Error(`bad pairing reply: ${String(e)}`);
    }
    if (!v.token) throw new Error('pairing reply had no token');
    this.set({ host, port: v.port ?? port, token: v.token, name: v.name ?? 'Portal Desktop' });
    return this.status();
  }

  async connect(host: string, port: number, token: string): Promise<MobileStatus> {
    const who = await whoami({ host, port, token, name: '' });
    this.set({ host, port, token, name: who.name });
    return this.status();
  }

  async forward(cmd: string, args: unknown): Promise<unknown> {
    const conn = loadConn();
    if (!conn) throw 'not connected to a Portal Desktop';
    let res: Response;
    try {
      res = await fetch(`http://${hostport(conn.host, conn.port)}/rpc/${cmd}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${conn.token}` },
        body: JSON.stringify(args ?? {}),
      });
    } catch (e) {
      throw `cannot reach ${conn.host}:${conn.port}: ${String(e)}`;
    }
    const body = await res.text();
    if (res.status === 401) this.clear(); // token revoked on the desktop
    if (!res.ok) throw rpcError(res.status, body);
    if (!body) return null;
    try {
      return JSON.parse(body);
    } catch (e) {
      throw `bad reply: ${String(e)}`;
    }
  }
}

const need = (v: unknown, name: string): string => {
  if (typeof v !== 'string' || !v) throw `missing \`${name}\``;
  return v;
};

async function local(bridge: Bridge, cmd: string, args: Record<string, unknown>): Promise<unknown> {
  switch (cmd) {
    case 'mobile_status':
      return bridge.status();
    case 'mobile_disconnect':
      bridge.clear();
      return bridge.status();
    case 'mobile_discover':
      return [];
    case 'mobile_connect': {
      const host = need(args.host, 'host');
      if (typeof args.port !== 'number') throw 'missing `port`';
      const token = need(args.token, 'token');
      return bridge.connect(host, args.port, token).catch((e: unknown) => {
        throw e instanceof Error ? e.message : e;
      });
    }
    case 'mobile_pair': {
      const host = need(args.host, 'host');
      if (typeof args.port !== 'number') throw 'missing `port`';
      const device = need(args.device, 'device');
      return bridge.pair(host, args.port, device).catch((e: unknown) => {
        throw e instanceof Error ? e.message : e;
      });
    }
    default:
      throw `unknown command ${cmd}`;
  }
}

/** True once {@link installBrowserShim} has actually installed the shim — i.e. this is
 * the thin-client build running in a plain browser, not the real Android Tauri runtime.
 * `/connect` reads this to know a daemon's host:port can only ever be its own origin
 * (no CORS layer on the daemon, so any other origin's `fetch` fails before it leaves). */
export let isBrowserClient = false;

/** Installs `window.__TAURI_INTERNALS__` when this is the thin-client build and nothing
 * is there already — i.e. never on the real Android Tauri runtime. Call once, as early
 * as possible (before any `invoke()`/`listen()` can fire). */
export function installBrowserShim(): void {
  if (!isMobileShell) return;
  const w = window as unknown as {
    __TAURI_INTERNALS__?: unknown;
    __TAURI_EVENT_PLUGIN_INTERNALS__?: unknown;
  };
  if (w.__TAURI_INTERNALS__) return;
  isBrowserClient = true;

  const bridge = new Bridge();
  const saved = loadConn();
  if (saved) bridge.set(saved); // mirrors Rust's setup(): read connection.json, reconnect

  // `@tauri-apps/api/event`'s `_unlisten` calls this directly (ahead of the
  // `plugin:event|unlisten` invoke below) rather than going through `invoke` for it.
  w.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: (event: string, eventId: number) => bridge.unlisten(event, eventId),
  };

  w.__TAURI_INTERNALS__ = {
    invoke(cmd: string, args: unknown = {}, _options?: unknown): Promise<unknown> {
      const a = (args ?? {}) as Record<string, unknown>;
      if (cmd === 'plugin:event|listen') {
        const { event, handler } = a as { event: string; handler: number };
        return Promise.resolve(bridge.listen(event, handler));
      }
      if (cmd === 'plugin:event|unlisten') {
        const { event, eventId } = a as { event: string; eventId: number };
        bridge.unlisten(event, eventId);
        return Promise.resolve(null);
      }
      if (cmd.startsWith('mobile_')) return local(bridge, cmd, a);
      if (!isForwardableCommand(cmd)) {
        return Promise.reject(`command "${cmd}" has no browser client equivalent`);
      }
      return bridge.forward(cmd, args);
    },
    transformCallback(callback: (payload: unknown) => void, once?: boolean): number {
      return bridge.transformCallback(callback, once);
    },
    unregisterCallback(id: number): void {
      bridge.unregisterCallback(id);
    },
  };
}

installBrowserShim();
