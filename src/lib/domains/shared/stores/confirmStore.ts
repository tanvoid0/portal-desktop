import { writable } from "svelte/store";

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

const DESTRUCTIVE_TITLE_PATTERN =
  /\b(delete|remove|uninstall|discard|rollback|reset)\b/i;

function isDestructiveTitle(title: string): boolean {
  return DESTRUCTIVE_TITLE_PATTERN.test(title);
}

export const confirmActions = {
  request(
    message: string,
    title = "Confirm",
    options: ConfirmOptions = {},
  ): Promise<boolean> {
    return new Promise((resolve) => {
      confirmStore.set({
        message,
        title,
        confirmLabel: options.confirmLabel ?? "Continue",
        cancelLabel: options.cancelLabel ?? "Cancel",
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
