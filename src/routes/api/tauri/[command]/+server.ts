import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { getClientIp } from "$lib/utils/serverUtils";

/**
 * API Proxy for Tauri commands
 *
 * This endpoint is a placeholder for Tauri command invocations from external devices.
 * Currently returns 501 (Not Implemented) as Tauri commands can only be executed
 * from within the Tauri app context.
 *
 * InvokeClient handles all routing and authentication logic on the client side.
 * This endpoint exists only to provide a consistent API structure for InvokeClient
 * routing. When InvokeClient makes HTTP requests for Tauri commands, they go here.
 *
 * To enable external device access, implement a Tauri HTTP server plugin that
 * exposes commands as REST endpoints and update this handler to proxy to it.
 */
export const POST: RequestHandler = async ({ params, request }) => {
  try {
    const command = params.command;

    if (!command) {
      return json({ error: "Command name is required" }, { status: 400 });
    }

    // Parse request body for command arguments (for logging/debugging)
    let args: any = {};
    try {
      const body = await request.json();
      args = body.args || body;
    } catch {
      // No body or invalid JSON, use empty args
      args = {};
    }

    // Extract client IP address from request headers
    const clientIp = getClientIp(request, request.headers);

    // For generate_device_passcode, inject IP address into device_info if available
    if (
      command === "generate_device_passcode" &&
      args.request &&
      args.request.device_info
    ) {
      try {
        const deviceInfo =
          typeof args.request.device_info === "string"
            ? JSON.parse(args.request.device_info)
            : args.request.device_info;

        // Add IP address to device_info if not already present or if it was "Unknown"
        if (!deviceInfo.ip || deviceInfo.ip === "Unknown") {
          deviceInfo.ip = clientIp;
          args.request.device_info = JSON.stringify(deviceInfo);
        }
      } catch {
        // If parsing fails, ignore and continue
      }
    }

    // Return 501 - Tauri commands can only be called from Tauri frontend
    // InvokeClient handles authentication and routing on the client side
    return json(
      {
        error:
          "Tauri commands can only be called from the Tauri frontend. For external device access, a Tauri HTTP server plugin is required.",
        command,
        args,
        hint: "This endpoint is a placeholder. To enable external device access, implement a Tauri HTTP server plugin that exposes commands as REST endpoints.",
      },
      { status: 501 }, // Not Implemented
    );
  } catch (error) {
    console.error("Tauri proxy error:", error);
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error occurred";
    return json({ error: errorMessage }, { status: 500 });
  }
};
