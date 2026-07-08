import { MediaQuery } from "svelte/reactivity";
import {
  LAYOUT_BREAKPOINTS,
  type ViewportTier,
} from "$lib/config/layout-breakpoints.js";

/**
 * Reactive viewport tier for layout decisions.
 * - compact: <768px
 * - standard: 768px–1919px
 * - ultrawide: ≥1920px
 */
export class ViewportTierQuery {
  readonly #compact: MediaQuery;
  readonly #ultrawide: MediaQuery;

  constructor() {
    this.#compact = new MediaQuery(
      `max-width: ${LAYOUT_BREAKPOINTS.md - 1}px`,
    );
    this.#ultrawide = new MediaQuery(
      `(min-width: ${LAYOUT_BREAKPOINTS.ultrawide}px)`,
    );
  }

  get tier(): ViewportTier {
    if (this.#compact.current) return "compact";
    if (this.#ultrawide.current) return "ultrawide";
    return "standard";
  }

  get isCompact(): boolean {
    return this.#compact.current;
  }

  get isUltrawide(): boolean {
    return this.#ultrawide.current;
  }
}
