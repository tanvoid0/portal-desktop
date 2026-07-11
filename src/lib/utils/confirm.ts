import {
  confirmActions,
  type ConfirmOptions,
} from "$lib/domains/shared/stores/confirmStore";

/**
 * Cross-environment confirmation dialog using shadcn AlertDialog.
 */
export async function confirmAction(
  message: string,
  title = "Confirm",
  options?: ConfirmOptions,
): Promise<boolean> {
  return confirmActions.request(message, title, options);
}

export type { ConfirmOptions };
