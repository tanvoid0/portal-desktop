/** Parse a `portalremote://{host}:{port}/{token}` QR payload; null when malformed. */
export function parsePairUrl(raw: string): { host: string; port: number; token: string } | null {
  const m = /^portalremote:\/\/([^/:\s]+):(\d{1,5})\/(\S+)$/.exec(raw.trim());
  if (!m) return null;
  const port = Number(m[2]);
  if (port < 1 || port > 65535) return null;
  return { host: m[1], port, token: m[3] };
}
