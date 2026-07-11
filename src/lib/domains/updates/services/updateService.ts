import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "$lib/domains/shared";
import type {
  UpdateCheckResult,
  UpdateErrorInfo,
  UpdateInfo,
  UpdateInstallResult,
} from "../types";

export type {
  UpdateCheckResult,
  UpdateErrorInfo,
  UpdateInfo,
  UpdateInstallResult,
  UpdateStatus,
} from "../types";

function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string") {
      return message;
    }
  }

  return "An unexpected error occurred";
}

/**
 * Map raw updater errors to user-friendly, actionable messages.
 */
export function parseUpdateError(error: unknown): UpdateErrorInfo {
  const technical = extractErrorMessage(error);
  const normalized = technical.toLowerCase();

  if (
    normalized.includes("signature") ||
    normalized.includes("minisign") ||
    normalized.includes("verify") ||
    normalized.includes("verification failed")
  ) {
    return {
      category: "signature",
      title: "Update verification failed",
      message:
        "The available update could not be verified. For your safety, it was not installed.",
      hint: "Download the latest release manually from GitHub Releases, or try again later.",
      recoverable: true,
      technical,
    };
  }

  if (
    normalized.includes("release json") ||
    normalized.includes("valid release") ||
    normalized.includes("latest.json") ||
    (normalized.includes("could not fetch") &&
      (normalized.includes("json") || normalized.includes("remote")))
  ) {
    return {
      category: "manifest",
      title: "Update feed unavailable",
      message:
        "The app could not read update information from the release server.",
      hint: "The release may not be published yet, or the update feed may require a public download URL. You can still download updates manually from GitHub Releases.",
      recoverable: true,
      technical,
    };
  }

  if (
    normalized.includes("404") ||
    normalized.includes("not found") ||
    normalized.includes("no such file")
  ) {
    return {
      category: "manifest",
      title: "Update feed not found",
      message: "No update information was found at the configured release URL.",
      hint: "If this app was built from source, install updates manually until a release feed is published.",
      recoverable: true,
      technical,
    };
  }

  if (
    normalized.includes("network") ||
    normalized.includes("timeout") ||
    normalized.includes("timed out") ||
    normalized.includes("connection") ||
    normalized.includes("dns") ||
    normalized.includes("offline") ||
    normalized.includes("failed to fetch") ||
    normalized.includes("unable to connect") ||
    normalized.includes("internet")
  ) {
    return {
      category: "network",
      title: "Connection problem",
      message: "Could not reach the update server.",
      hint: "Check your internet connection and try again.",
      recoverable: true,
      technical,
    };
  }

  if (
    normalized.includes("updater") &&
    (normalized.includes("disabled") ||
      normalized.includes("not enabled") ||
      normalized.includes("not configured") ||
      normalized.includes("not available"))
  ) {
    return {
      category: "disabled",
      title: "Updates unavailable in this build",
      message: "Automatic updates are only supported in packaged release builds.",
      hint: "Run a release build or download the latest installer from GitHub Releases.",
      recoverable: false,
      technical,
    };
  }

  if (normalized.includes("download")) {
    return {
      category: "download",
      title: "Download failed",
      message: "The update could not be downloaded.",
      hint: "Check your connection and try again, or download the installer manually.",
      recoverable: true,
      technical,
    };
  }

  if (normalized.includes("install")) {
    return {
      category: "install",
      title: "Installation failed",
      message: "The update downloaded but could not be installed.",
      hint: "Try again, or download and run the latest installer manually.",
      recoverable: true,
      technical,
    };
  }

  return {
    category: "unknown",
    title: "Update check failed",
    message: "Something went wrong while checking for updates.",
    hint: "Try again in a moment, or download updates manually from GitHub Releases.",
    recoverable: true,
    technical,
  };
}

/**
 * Check for available updates without throwing.
 */
export async function checkForUpdates(): Promise<UpdateCheckResult> {
  try {
    logger.info("Checking for updates...");
    const update = await check();

    if (update !== null) {
      logger.info("Update available", { version: update.version });

      return {
        status: "available",
        info: {
          version: update.version || "unknown",
          date: update.date,
          body: update.body,
          available: true,
        },
      };
    }

    logger.info("No updates available");
    const currentVersion = await getCurrentVersion();

    return {
      status: "current",
      info: {
        version: currentVersion ?? "unknown",
        available: false,
      },
    };
  } catch (error) {
    const parsed = parseUpdateError(error);
    logger.error("Failed to check for updates", {
      category: parsed.category,
      message: parsed.message,
      technical: parsed.technical,
    });

    return { status: "error", error: parsed };
  }
}

/**
 * Install the available update without throwing.
 */
export async function installUpdateAndRelaunch(): Promise<UpdateInstallResult | void> {
  try {
    logger.info("Installing update...");

    const update = await check();

    if (update === null) {
      return {
        status: "error",
        error: {
          category: "unknown",
          title: "No update available",
          message: "There is no update ready to install.",
          hint: "Check for updates again before installing.",
          recoverable: true,
        },
      };
    }

    await update.downloadAndInstall((progress) => {
      if (progress.event === "Started") {
        logger.info("Download started");
      } else if (progress.event === "Progress") {
        const chunkLength = progress.data?.chunkLength || 0;
        logger.info(`Download progress: ${chunkLength} bytes downloaded`);
      } else if (progress.event === "Finished") {
        logger.info("Download finished, installing...");
      }
    });

    logger.info("Update installed, relaunching...");
    await relaunch();
  } catch (error) {
    const parsed = parseUpdateError(error);
    logger.error("Failed to install update", {
      category: parsed.category,
      message: parsed.message,
      technical: parsed.technical,
    });

    return { status: "error", error: parsed };
  }
}

/**
 * Get the current application version, or null if unavailable.
 */
export async function getCurrentVersion(): Promise<string | null> {
  try {
    return await invoke<string>("get_app_version_command");
  } catch (error) {
    logger.error("Failed to get app version", { error });
    return null;
  }
}
