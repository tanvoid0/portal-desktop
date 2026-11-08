import type { UpdateErrorInfo, UpdateInfo } from '../types';

/** Shared state for the "update available" dialog shown on launch. */
class UpdatePromptState {
  info = $state<UpdateInfo | null>(null);
  open = $state(false);
  installing = $state(false);
  error = $state<UpdateErrorInfo | null>(null);

  show(info: UpdateInfo) {
    this.info = info;
    this.error = null;
    this.open = true;
  }

  dismiss() {
    if (!this.installing) this.open = false;
  }
}

export const updatePrompt = new UpdatePromptState();
