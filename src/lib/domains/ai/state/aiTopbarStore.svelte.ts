import type { Snippet } from "svelte";

/**
 * Controls a page hands to the AI layout so they render beside the tab bar
 * instead of crowding the page's own header row. Pages register in an
 * `$effect` and clear on teardown.
 */
export const aiTopbar = $state<{ actions: Snippet | null }>({ actions: null });
