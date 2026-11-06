/** The backend can't decrypt the stored GitHub token (keychain key changed): user must reconnect. */
// Matches the Rust string `Failed to decrypt GitHub credential: ...` (portal-core github/service.rs:1157).
export function isTokenUnreadableError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error ?? '');
  return message.includes('Failed to decrypt GitHub credential');
}
