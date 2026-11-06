import { toast } from '$lib/utils/toast';
import { aiProviderService } from '../services/aiProviderService.js';

/**
 * - `unknown`: no verdict yet, or the health command is not available — never blocks anything.
 * - `unreachable`: the server is not running.
 * - `unauthenticated`: reachable, auth required, our token is rejected.
 */
export type PlatformStatus = 'unknown' | 'ok' | 'unreachable' | 'unauthenticated';

/**
 * Single source of truth for agent-platform health, shared by /ai, /coder and
 * Settings. Refresh on mount and after start/pair; there is no polling loop.
 */
class AiPlatformState {
  checking = $state(false);
  starting = $state(false);
  known = $state(false);
  reachable = $state(true);
  authenticated = $state(true);
  detail = $state<string | null>(null);
  baseUrl = $state('');
  authRequired = $state(false);

  status = $derived.by<PlatformStatus>(() => {
    if (!this.known) return 'unknown';
    if (!this.reachable) return 'unreachable';
    if (this.authRequired && !this.authenticated) return 'unauthenticated';
    return 'ok';
  });
  /** True only when we positively know platform calls will fail. */
  blocked = $derived(this.status === 'unreachable' || this.status === 'unauthenticated');

  async refresh() {
    if (this.checking) return;
    this.checking = true;
    try {
      const h = await aiProviderService.platformHealth();
      this.reachable = h.reachable;
      this.authenticated = h.authenticated;
      this.authRequired = h.auth_required;
      this.detail = h.detail;
      this.baseUrl = h.base_url;
      this.known = true;
    } catch {
      // Command not wired (or IPC failure): degrade to "no verdict", don't block.
      this.known = false;
    } finally {
      this.checking = false;
    }
  }

  async start() {
    if (this.starting) return;
    this.starting = true;
    try {
      const r = await aiProviderService.startPlatform();
      if (r.started || r.already_running) toast.success(r.detail || 'Agent platform is running');
      else toast.error('Could not start agent-platform', r.detail);
    } catch (e) {
      toast.error('Could not start agent-platform', String(e));
    } finally {
      this.starting = false;
    }
    await this.refresh();
  }
}

export const aiPlatform = new AiPlatformState();
