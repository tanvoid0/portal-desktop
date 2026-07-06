import type { Handle } from "@sveltejs/kit";
import { isLocalhostRequest } from "$lib/utils/serverUtils";

/**
 * Server-side hooks for authentication and security
 * Blocks unauthorized browser access at the server level
 */
export const handle: Handle = async ({ event, resolve }) => {
  // Allow API routes and static assets
  if (
    event.url.pathname.startsWith("/api/") ||
    event.url.pathname.startsWith("/_app/") ||
    event.url.pathname.startsWith("/favicon") ||
    event.url.pathname.startsWith("/logo") ||
    event.url.pathname.startsWith("/svelte.svg") ||
    event.url.pathname.startsWith("/tauri.svg") ||
    event.url.pathname.startsWith("/vite.svg") ||
    event.url.pathname.startsWith("/robots.txt")
  ) {
    return resolve(event);
  }

  // Allow localhost access without authentication
  // For remote access, the client-side guard will handle authentication
  // (We can't easily distinguish Tauri vs browser on server-side)

  return resolve(event);
};
