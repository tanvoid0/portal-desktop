import { writable } from 'svelte/store';

export interface ConfirmOptions {
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

export interface ConfirmRequest {
  message: string;
  title: string;
  confirmLabel: string;
  cancelLabel: string;
  destructive: boolean;
  resolve: (confirmed: boolean) => void;
}

export const confirmStore = writable<ConfirmRequest | null>(null);

const DESTRUCTIVE_TITLE_PATTERN = /\b(delete|remove|uninstall|discard|rollback|reset)\b/i;

function isDestructiveTitle(title: string): boolean {
  return DESTRUCTIVE_TITLE_PATTERN.test(title);
}

/**
 * Most call sites name the action in the title ("Delete IDE", "Uninstall version") and never
 * set `confirmLabel`, leaving the button to read "Continue" regardless of what it does. Use the
 * title's leading verb as the button label instead; only the generic default title falls back to
 * "Continue".
 */
function defaultConfirmLabel(title: string): string {
  if (title === 'Confirm') return 'Continue';
  return title.trim().match(/^[A-Za-z]+/)?.[0] ?? 'Continue';
}

export const confirmActions = {
  request(message: string, title = 'Confirm', options: ConfirmOptions = {}): Promise<boolean> {
    return new Promise((resolve) => {
      confirmStore.set({
        message,
        title,
        confirmLabel: options.confirmLabel ?? defaultConfirmLabel(title),
        cancelLabel: options.cancelLabel ?? 'Cancel',
        destructive: options.destructive ?? isDestructiveTitle(title),
        resolve,
      });
    });
  },

  confirm() {
    confirmStore.update((state) => {
      state?.resolve(true);
      return null;
    });
  },

  cancel() {
    confirmStore.update((state) => {
      state?.resolve(false);
      return null;
    });
  },
};
