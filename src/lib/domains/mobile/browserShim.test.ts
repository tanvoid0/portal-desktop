import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const KEY = 'portal-client-connection';
const CONN = { host: '10.0.0.5', port: 18420, token: 'tok_abc', name: 'PC' };

/** The slice of `WebSocket` the shim touches, driven by hand. */
class FakeSocket {
  static readonly OPEN = 1;
  static all: FakeSocket[] = [];
  readyState = 0;
  sent: string[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((ev: { data: string }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;

  constructor(readonly url: string) {
    FakeSocket.all.push(this);
  }
  send(text: string) {
    this.sent.push(text);
  }
  close() {
    this.readyState = 3;
    this.onclose?.();
  }
  open() {
    this.readyState = FakeSocket.OPEN;
    this.onopen?.();
  }
  receive(data: string) {
    this.onmessage?.({ data });
  }
}

interface Internals {
  invoke(cmd: string, args?: unknown): Promise<unknown>;
  transformCallback(cb: (e: { payload: unknown }) => void): number;
}

/** A fresh shim over a fake `localStorage`/`WebSocket`; nothing is paired until a test stores a connection. */
async function install() {
  vi.resetModules();
  vi.stubEnv('VITE_PORTAL_MOBILE', '1');
  vi.stubGlobal('window', {});
  const store = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, v),
    removeItem: (k: string) => void store.delete(k),
  });
  vi.stubGlobal('WebSocket', FakeSocket);
  await import('./browserShim');
  const tauri = (window as unknown as { __TAURI_INTERNALS__: Internals }).__TAURI_INTERNALS__;
  const pair = () => store.set(KEY, JSON.stringify(CONN));
  const send = (frame?: unknown) => tauri.invoke('mobile_control_send', { frame });
  return { tauri, pair, send };
}

describe('mobile_control_send in the browser client', () => {
  beforeEach(() => {
    FakeSocket.all = [];
  });
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
  });

  it('refuses without a connection or a frame, and opens nothing', async () => {
    const { pair, send } = await install();
    await expect(send({ t: 'ping' })).rejects.toBe('not connected to a Portal Desktop');
    pair();
    await expect(send()).rejects.toBe('missing `frame`');
    expect(FakeSocket.all).toHaveLength(0);
  });

  it('opens one /control socket lazily, queues until it is open, then reuses it', async () => {
    const { pair, send } = await install();
    pair();

    // Returns at once with the socket still connecting.
    await expect(send({ t: 'mouse_move', dx: 50, dy: 0 })).resolves.toBeNull();
    await send({ t: 'mouse_move', dx: 1, dy: 1 });
    expect(FakeSocket.all).toHaveLength(1);
    expect(FakeSocket.all[0].url).toBe('ws://10.0.0.5:18420/control?token=tok_abc');
    expect(FakeSocket.all[0].sent).toEqual([]);

    FakeSocket.all[0].open();
    expect(FakeSocket.all[0].sent).toEqual([
      '{"t":"mouse_move","dx":50,"dy":0}',
      '{"t":"mouse_move","dx":1,"dy":1}',
    ]);
    await send({ t: 'tap', key: 'a' });
    expect(FakeSocket.all).toHaveLength(1);
    expect(FakeSocket.all[0].sent).toHaveLength(3);
  });

  it('holds at most 64 frames for a socket that is still opening', async () => {
    const { pair, send } = await install();
    pair();
    for (let i = 0; i < 64; i++) await send({ t: 'mouse_move', dx: i, dy: 0 });
    await expect(send({ t: 'x' })).rejects.toBe('the control socket is backed up');
    FakeSocket.all[0].open();
    expect(FakeSocket.all[0].sent).toHaveLength(64);
  });

  it('re-emits only {t:"error"} frames as mobile://control-error', async () => {
    const { tauri, pair, send } = await install();
    pair();
    const seen: unknown[] = [];
    const handler = tauri.transformCallback((e) => seen.push(e.payload));
    await tauri.invoke('plugin:event|listen', { event: 'mobile://control-error', handler });

    await send({ t: 'ping' });
    const socket = FakeSocket.all[0];
    socket.open();
    socket.receive('{"t":"hello","name":"PC"}');
    socket.receive('{"event":"coder://change","payload":{}}'); // the fanout, already on /events
    socket.receive('{"t":"pong","seq":1}');
    socket.receive('not json');
    socket.receive('null');
    expect(seen).toEqual([]);

    socket.receive('{"t":"error","message":"this device is not permitted to send input"}');
    expect(seen).toEqual([{ t: 'error', message: 'this device is not permitted to send input' }]);
  });

  it('reopens after the daemon closes it, and closes on disconnect', async () => {
    const { tauri, pair, send } = await install();
    pair();
    await send({ t: 'a' });
    FakeSocket.all[0].open();
    FakeSocket.all[0].close(); // dropped: revoked, caps changed or expired

    await send({ t: 'b' });
    expect(FakeSocket.all).toHaveLength(2);
    FakeSocket.all[1].open();
    expect(FakeSocket.all[1].sent).toEqual(['{"t":"b"}']); // nothing stale replayed

    await tauri.invoke('mobile_disconnect');
    expect(FakeSocket.all[1].readyState).toBe(3);
    await expect(send({ t: 'c' })).rejects.toBe('not connected to a Portal Desktop');
    expect(FakeSocket.all).toHaveLength(2);
  });
});
