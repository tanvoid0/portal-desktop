import { logger } from '$lib/domains/shared';
import { toast } from '$lib/utils/toast';
import { isTauriEnvironment } from '$lib/utils/tauri';
import { settingsService } from '$lib/domains/settings/services/settingsService';
import { settingsActions } from '$lib/domains/settings/stores/settingsStore';
import type { UpdateSettings } from '$lib/domains/settings/types';
import { updatePrompt } from '../state/updatePrompt.svelte';
import { checkForUpdates, installUpdateAndRelaunch } from './updateService';

/** Install the pending update; on failure surface the error in the prompt and a toast. */
export async function installFromPrompt(): Promise<void> {
  updatePrompt.installing = true;
  updatePrompt.error = null;
  const result = await installUpdateAndRelaunch();
  // On success the app relaunches and never gets here.
  if (result?.status === 'error') {
    updatePrompt.error = result.error;
    toast.error(result.error.title, { description: result.error.message });
  }
  updatePrompt.installing = false;
}

/**
 * Persist a change to the update preferences. The backend replaces the whole `app`
 * section, so merge into the current one rather than sending a fragment.
 */
export async function saveUpdatePrefs(patch: Partial<UpdateSettings>): Promise<UpdateSettings> {
  const current = await settingsService.getSettings();
  const updates = { ...current.app.updates, ...patch };
  await settingsActions.updateSettings({ app: { ...current.app, updates } });
  return updates;
}

/**
 * Launch-time update check. Silent unless an update exists: prompts the user, or installs
 * straight away when "automatic install" is on.
 */
export async function runStartupUpdateCheck(): Promise<void> {
  if (!isTauriEnvironment()) return;

  try {
    const { app } = await settingsService.getSettings();
    if (!app.updates.autoCheck) return;

    const result = await checkForUpdates();
    if (result.status !== 'available') return; // errors are already logged; don't nag on launch

    if (app.updates.autoInstall) {
      toast.info(`Installing Portal Desktop ${result.info.version}…`, {
        description: 'The app will restart when the update is ready.',
      });
      updatePrompt.info = result.info;
      await installFromPrompt();
      if (!updatePrompt.error) return;
      // Auto-install failed: fall through so the user can retry from the prompt.
    }

    updatePrompt.show(result.info);
  } catch (error) {
    logger.error('Startup update check failed', { error });
  }
}
