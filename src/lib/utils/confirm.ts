import { isTauriEnvironment } from "./tauri";

/**
 * Cross-environment confirmation dialog.
 * Tauri 2 replaces window.confirm with an async dialog API.
 */
export async function confirmAction(
  message: string,
  title = "Confirm",
): Promise<boolean> {
  if (isTauriEnvironment()) {
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    return confirm(message, { title, kind: "warning" });
  }

  return window.confirm(message);
}
