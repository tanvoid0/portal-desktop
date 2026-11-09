/** Tauri 2 commands returning `Result<_, String>` reject with a plain string, not an
 *  `Error` — so `err.message` alone misses it. Extracts the message from either shape. */
export function getErrorMessage(error: unknown, fallback = ''): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return fallback;
}
