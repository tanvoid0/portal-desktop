// Both match the Rust string `Failed to decrypt GitHub credential: ...` (portal-core github/service.rs).
function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error ?? '');
}

/** The vault is locked, so the (fine) token can't be decrypted yet: user must unlock, not reconnect. */
export function isVaultLockedError(error: unknown): boolean {
  return /vault is locked/i.test(messageOf(error));
}

/** The backend can't decrypt the stored GitHub token (keychain key changed): user must reconnect. */
export function isTokenUnreadableError(error: unknown): boolean {
  const message = messageOf(error);
  return message.includes('Failed to decrypt GitHub credential') && !isVaultLockedError(message);
}
