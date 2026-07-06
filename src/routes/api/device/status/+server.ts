import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";

/**
 * Device Status Endpoint
 * Allows devices to check their approval status and retrieve access token
 * This is a public endpoint that requires device_id
 */
export const POST: RequestHandler = async ({ request }) => {
  try {
    const { device_id } = await request.json();

    if (!device_id) {
      return json({ error: "device_id is required" }, { status: 400 });
    }

    // This endpoint would need to query the Tauri backend
    // For now, return a placeholder
    // In a full implementation, this would call a Tauri command via HTTP server
    return json(
      {
        error:
          "Device status check requires Tauri HTTP server. This is a placeholder endpoint.",
        device_id,
        hint: "The device should poll for approval status. In a full implementation, this endpoint would query the database for device approval status and return the access_token if approved.",
      },
      { status: 501 },
    );
  } catch (error) {
    console.error("Device status error:", error);
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error occurred";
    return json({ error: errorMessage }, { status: 500 });
  }
};
