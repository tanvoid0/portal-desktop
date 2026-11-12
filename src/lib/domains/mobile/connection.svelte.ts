import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface MobileStatus {
  connected: boolean;
  host: string | null;
  port: number | null;
  name: string | null;
  caps: string[];
}

export interface DiscoveredDesktop {
  host: string;
  port: number;
  name: string;
  version: string;
}

class MobileConnection {
  status = $state<MobileStatus | null>(null);
  readonly loaded = $derived(this.status !== null);
  readonly connected = $derived(this.status?.connected ?? false);
  private unlisten: UnlistenFn | undefined;

  /** Fetch current status and follow `mobile://connection`. Call once from the root layout. */
  async init(): Promise<void> {
    if (!this.unlisten) {
      this.unlisten = await listen<MobileStatus>('mobile://connection', (e) => {
        this.status = e.payload;
      });
    }
    try {
      this.status = await invoke<MobileStatus>('mobile_status');
    } catch {
      this.status = { connected: false, host: null, port: null, name: null, caps: [] };
    }
  }

  pair(host: string, port: number, device: string) {
    return invoke<MobileStatus>('mobile_pair', { host, port, device });
  }

  connect(host: string, port: number, token: string) {
    return invoke<MobileStatus>('mobile_connect', { host, port, token });
  }

  disconnect() {
    return invoke<void>('mobile_disconnect');
  }

  discover() {
    return invoke<DiscoveredDesktop[]>('mobile_discover');
  }
}

export const mobileConnection = new MobileConnection();
