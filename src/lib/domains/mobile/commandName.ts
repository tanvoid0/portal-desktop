/** Ascii alnum/underscore only — mirrors the `forward()` check in `mobile/src-tauri/src/lib.rs`,
 *  so a browser client rejects the same command names the Android shell would refuse to forward. */
export function isForwardableCommand(cmd: string): boolean {
  return /^[A-Za-z0-9_]+$/.test(cmd);
}
