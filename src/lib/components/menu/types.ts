import type { Component } from 'svelte';

/** One entry in a {@link ContextMenuList}. A domain builds an array of these to
 *  describe the actions for an item; the component renders them. */
export interface MenuItem {
  label: string;
  /** unplugin-icons component, e.g. `~icons/fa6-solid/pen`. */
  icon?: Component;
  onSelect?: () => void;
  variant?: 'default' | 'destructive';
  disabled?: boolean;
  /** Draw a separator above this item (ignored for the first visible item). */
  separatorBefore?: boolean;
  /** One level of nesting — e.g. a "Set status ▸" submenu. */
  submenu?: MenuItem[];
}
