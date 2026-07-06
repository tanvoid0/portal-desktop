/**
 * Server-side utility functions
 * These are safe to use in SvelteKit server routes and hooks
 */

/**
 * Check if a request is from localhost (server-side)
 * Similar to InvokeClient.isLocalhost() but works in server context
 */
export function isLocalhostRequest(url: URL, headers: Headers): boolean {
  const hostname = url.hostname;
  const hostHeader = headers.get("host") || "";

  return (
    hostname === "localhost" ||
    hostname === "127.0.0.1" ||
    hostname === "::1" ||
    hostname === "[::1]" ||
    hostHeader.includes("localhost") ||
    hostHeader.includes("127.0.0.1")
  );
}

/**
 * Extract client IP address from request headers
 * Checks X-Forwarded-For, X-Real-IP, and falls back to connection remote address
 *
 * @param request - The incoming request
 * @param headers - Request headers
 * @returns The client IP address or 'Unknown' if not available
 */
export function getClientIp(request: Request, headers: Headers): string {
  // Check X-Forwarded-For header (first IP in chain if proxied)
  const forwardedFor = headers.get("x-forwarded-for");
  if (forwardedFor) {
    // X-Forwarded-For can contain multiple IPs, take the first one
    const firstIp = forwardedFor.split(",")[0].trim();
    if (firstIp) {
      return firstIp;
    }
  }

  // Check X-Real-IP header (set by some proxies)
  const realIp = headers.get("x-real-ip");
  if (realIp) {
    return realIp.trim();
  }

  // Check CF-Connecting-IP (Cloudflare)
  const cfIp = headers.get("cf-connecting-ip");
  if (cfIp) {
    return cfIp.trim();
  }

  // Fallback: try to get from request URL (for localhost/dev scenarios)
  try {
    const url = new URL(request.url);
    if (
      url.hostname &&
      url.hostname !== "localhost" &&
      url.hostname !== "127.0.0.1"
    ) {
      return url.hostname;
    }
  } catch {
    // Ignore URL parsing errors
  }

  return "Unknown";
}
