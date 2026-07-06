/**
 * Tauri API utilities
 *
 * Provides SSR-safe access to Tauri APIs using dynamic imports.
 * This avoids build-time errors when Tauri APIs are not available.
 */

/**
 * Check if running in a Tauri environment
 */
export function isTauriEnvironment(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  // Check for Tauri internals (handle both possible property names)
  return "__TAURI_INTERNALS__" in window || "__TAURI__" in window;
}

/**
 * Lazy-loaded Tauri invoke function
 * Cached after first load to avoid repeated imports
 */
let invokeFn: ((cmd: string, args?: any) => Promise<any>) | null = null;

/**
 * Get the Tauri invoke function (SSR-safe)
 *
 * This function dynamically imports the Tauri API only when needed,
 * avoiding build-time errors when Tauri APIs are not available.
 *
 * @returns The Tauri invoke function
 * @throws Error if not in a Tauri environment
 *
 * @example
 * ```ts
 * const invoke = await getInvoke();
 * const result = await invoke('my_command', { arg: 'value' });
 * ```
 */
export async function getInvoke(): Promise<
  (cmd: string, args?: any) => Promise<any>
> {
  if (!isTauriEnvironment()) {
    throw new Error("Tauri environment required");
  }

  if (!invokeFn) {
    const tauriCore = await import("@tauri-apps/api/core");
    invokeFn = tauriCore.invoke;
  }

  return invokeFn;
}

/**
 * Type-safe wrapper for Tauri invoke
 *
 * @example
 * ```ts
 * const result = await tauriInvoke<string>('my_command', { arg: 'value' });
 * ```
 */
export async function tauriInvoke<T = any>(
  cmd: string,
  args?: any,
): Promise<T> {
  if (typeof window === "undefined") {
    throw new Error("Tauri environment required");
  }
  const invoke = await getInvoke();
  return invoke(cmd, args) as Promise<T>;
}
