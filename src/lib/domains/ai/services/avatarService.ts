/**
 * Avatar Service - Manages avatar behavior, toast interception, and AI suggestion generation
 */

import { get } from "svelte/store";
import { toastStore } from "$lib/utils/toast";
import { avatarStore, avatarActions } from "../stores/avatarStore";
import { aiChatService } from "./aiChatService";
import type { AvatarSuggestion } from "../types/avatar";
import { page } from "$app/stores";
import { logger } from "$lib/domains/shared";

const log = logger.createScoped("AvatarService");

class AvatarService {
  private unsubscribeToast: (() => void) | null = null;
  private suggestionCache = new Map<string, string>();
  private lastSuggestionTime = 0;
  private readonly MIN_SUGGESTION_INTERVAL = 5000; // 5 seconds between suggestions
  private readonly CACHE_DURATION = 60000; // Cache suggestions for 1 minute

  /**
   * Initialize the avatar service
   */
  init() {
    if (this.unsubscribeToast) {
      // Already initialized
      return;
    }

    // Subscribe to toast store
    this.unsubscribeToast = toastStore.subscribe((store) => {
      const config = get(avatarStore).config;

      // Only process if avatar is enabled
      if (!config.enabled) {
        return;
      }

      // Process new toasts
      const latestToast = store.toasts[store.toasts.length - 1];
      if (latestToast) {
        this.handleToast(latestToast);
      }
    });

    log.info("Avatar service initialized");
  }

  /**
   * Cleanup and stop listening
   */
  destroy() {
    if (this.unsubscribeToast) {
      this.unsubscribeToast();
      this.unsubscribeToast = null;
    }
    log.info("Avatar service destroyed");
  }

  /**
   * Handle a new toast message
   */
  private async handleToast(toast: {
    id: string;
    title?: string;
    description?: string;
    type?: "success" | "error" | "warning" | "info";
  }) {
    // Only handle error, warning, and info toasts
    if (!toast.type || !["error", "warning", "info"].includes(toast.type)) {
      return;
    }

    // Rate limiting - don't generate suggestions too frequently
    const now = Date.now();
    if (now - this.lastSuggestionTime < this.MIN_SUGGESTION_INTERVAL) {
      return;
    }

    // Check if we already have a suggestion for this error
    const errorKey = this.getErrorKey(toast);
    const cachedSuggestion = this.suggestionCache.get(errorKey);

    if (cachedSuggestion) {
      this.showSuggestion(cachedSuggestion, toast);
      return;
    }

    // Generate new suggestion
    await this.generateSuggestion(toast);
  }

  /**
   * Generate a unique key for error caching
   */
  private getErrorKey(toast: {
    title?: string;
    description?: string;
    type?: string;
  }): string {
    const errorText = `${toast.type}:${toast.title || ""}:${toast.description || ""}`;
    return errorText.toLowerCase().trim();
  }

  /**
   * Generate AI-powered suggestion
   */
  private async generateSuggestion(toast: {
    title?: string;
    description?: string;
    type?: string;
  }) {
    try {
      // Set thinking state
      avatarActions.setState("thinking");
      avatarActions.setExpression("thinking");

      // Get current page context
      const currentPage = get(page);
      const pagePath = currentPage?.url?.pathname || "unknown";

      // Build prompt
      const errorMessage =
        toast.description || toast.title || "An error occurred";
      const prompt = `User encountered ${toast.type || "an issue"}: "${errorMessage}". Current page: ${pagePath}. Provide a brief, helpful suggestion (max 2 sentences) to help resolve this. Be concise and actionable.`;

      // Generate suggestion using AI
      const suggestion = await aiChatService.sendMessage(prompt, [], {
        temperature: 0.7,
        max_tokens: 150,
      });

      // Cache the suggestion
      const errorKey = this.getErrorKey(toast);
      this.suggestionCache.set(errorKey, suggestion);

      // Clear cache after duration
      setTimeout(() => {
        this.suggestionCache.delete(errorKey);
      }, this.CACHE_DURATION);

      // Show the suggestion
      this.showSuggestion(suggestion, toast);
      this.lastSuggestionTime = Date.now();
    } catch (error) {
      log.error("Failed to generate suggestion", error);

      // Fallback to static suggestion based on error type
      const fallbackSuggestion = this.getFallbackSuggestion(toast);
      this.showSuggestion(fallbackSuggestion, toast);
    } finally {
      // Reset to idle if no suggestion is shown
      setTimeout(() => {
        const currentState = get(avatarStore);
        if (currentState.state === "thinking") {
          avatarActions.setState("idle");
          avatarActions.setExpression("neutral");
        }
      }, 1000);
    }
  }

  /**
   * Get fallback suggestion when AI is unavailable
   */
  private getFallbackSuggestion(toast: {
    title?: string;
    description?: string;
    type?: string;
  }): string {
    const errorType = toast.type || "error";

    switch (errorType) {
      case "error":
        return "Something went wrong. Try checking your connection or refreshing the page.";
      case "warning":
        return "This might cause issues. Consider reviewing your settings or configuration.";
      case "info":
        return "You might find helpful information in the documentation or settings.";
      default:
        return "If this persists, try refreshing the page or checking the logs.";
    }
  }

  /**
   * Show a suggestion to the user
   */
  private showSuggestion(
    message: string,
    toast: { title?: string; description?: string; type?: string },
  ) {
    const suggestion: AvatarSuggestion = {
      id: `suggestion-${Date.now()}`,
      message: message.trim(),
      timestamp: new Date(),
      context: {
        error: toast.description || toast.title,
        page: get(page)?.url?.pathname,
        toastType: toast.type as
          | "error"
          | "warning"
          | "info"
          | "success"
          | undefined,
      },
    };

    // Set expression based on error type
    if (toast.type === "error") {
      avatarActions.setExpression("concerned");
    } else if (toast.type === "success") {
      avatarActions.setExpression("happy");
    } else {
      avatarActions.setExpression("neutral");
    }

    avatarActions.setSuggestion(suggestion);

    // Auto-dismiss after 10 seconds
    setTimeout(() => {
      const currentSuggestion = get(avatarStore).currentSuggestion;
      if (currentSuggestion?.id === suggestion.id) {
        avatarActions.dismissSuggestion();
      }
    }, 10000);
  }

  /**
   * Manually trigger a suggestion (for testing or special cases)
   */
  async triggerSuggestion(
    message: string,
    context?: { error?: string; page?: string },
  ) {
    const suggestion: AvatarSuggestion = {
      id: `suggestion-${Date.now()}`,
      message,
      timestamp: new Date(),
      context,
    };

    avatarActions.setSuggestion(suggestion);
  }

  /**
   * Clear suggestion cache
   */
  clearCache() {
    this.suggestionCache.clear();
  }
}

export const avatarService = new AvatarService();
