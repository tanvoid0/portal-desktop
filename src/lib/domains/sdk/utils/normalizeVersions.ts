/**
 * Normalizers for the SDK version commands.
 *
 * `fetch_available_versions` returns Rust's `SDKVersion` (snake_case, no SDK of its
 * own), so every caller that lists versions for more than one SDK has to carry the
 * SDK along with each row — otherwise the UI shows a bare version number and the
 * developer cannot tell what they are about to install.
 */

/** A row exactly as the backend sends it. */
export interface RawSDKVersion {
  version: string;
  installed: boolean;
  active: boolean;
  size?: string | null;
  release_date?: string | null;
}

/** A row the UI can render: camelCase, and tagged with the SDK it belongs to. */
export interface SDKVersionRow {
  sdkType: string;
  sdkName: string;
  version: string;
  installed: boolean;
  active: boolean;
  size: string | null;
  releaseDate: string | null;
  /** Client-side action state. */
  downloading: boolean;
  error: string | null;
}

export function normalizeVersion(
  raw: RawSDKVersion,
  sdkType: string,
  sdkName: string
): SDKVersionRow {
  return {
    sdkType,
    sdkName,
    version: raw.version,
    installed: !!raw.installed,
    active: !!raw.active,
    // The feeds send "0.0 MB" when they have no size; showing that is worse than nothing.
    size: raw.size && !/^0+(\.0+)?\s/.test(raw.size) ? raw.size : null,
    releaseDate: raw.release_date ?? null,
    downloading: false,
    error: null,
  };
}

/**
 * Strip the chatter out of a version string.
 *
 * `sdk_version` is the raw stdout of `<binary> --version`, so it arrives as
 * "go version go1.26.2 windows/amd64" or "rustc 1.94.1 (e408947bf 2026-03-25)".
 * The cards have room for a version, not a sentence.
 */
export function formatSdkVersion(raw: string | null | undefined): string | null {
  if (!raw) return null;
  const match = raw.match(/\d+(?:\.\d+)*(?:[-+][0-9A-Za-z.-]+)?/);
  return match ? match[0] : raw.trim().replace(/^v/, '') || null;
}
