import { toast } from '$lib/utils/toast';
import { aiProviderService } from '../services/aiProviderService.js';

/** Where to get the Agent Platform app when it isn't installed. */
export const AGENT_PLATFORM_RELEASES_URL =
  'https://github.com/tanvoid0/agent-platform/releases/latest';

/**
 * - `unknown`: no verdict yet, or the health command is not available — never blocks anything.
 * - `not_installed`: the Agent Platform app isn't on this machine.
 * - `unreachable`: it's installed but not running.
 * - `unauthenticated`: reachable, auth required, our token is rejected.
 */
export type PlatformStatus = 'unknown' | 'ok' | 'not_installed' | 'unreachable' | 'unauthenticated';

/**
 * Single source of truth for agent-platform health, shared by /ai, /coder and
 * Settings. Refresh on mount and after start/pair; there is no polling loop.
 */
class AiPlatformState {
  checking = $state(false);
  starting = $state(false);
  known = $state(false);
  reachable = $state(true);
  installed = $state(true);
  authenticated = $state(true);
  detail = $state<string | null>(null);
  baseUrl = $state('');
  authRequired = $state(false);

  status = $derived.by<PlatformStatus>(() => {
    if (!this.known) return 'unknown';
    if (!this.reachable) return this.installed ? 'unreachable' : 'not_installed';
    if (this.authRequired && !this.authenticated) return 'unauthenticated';
    return 'ok';
  });
  /** True only when we positively know platform calls will fail. */
  blocked = $derived(this.status !== 'unknown' && this.status !== 'ok');

  async refresh() {
    if (this.checking) return;
    this.checking = true;
    try {
      const h = await aiProviderService.platformHealth();
      this.reachable = h.reachable;
      this.installed = h.installed;
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

  /** Launch the installed app. Call only when `status === 'unreachable'`. */
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
