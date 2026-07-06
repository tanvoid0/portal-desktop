const STORAGE_KEY = "portal-terminal-widget-rail";

export type WidgetId =
  | "blocks"
  | "ai"
  | "notes"
  | "launcher"
  | "history";

export interface WidgetRailState {
  open: boolean;
  activeWidgets: WidgetId[];
}

const defaultState: WidgetRailState = {
  open: true,
  activeWidgets: ["blocks", "notes"],
};

export function loadWidgetRailState(): WidgetRailState {
  if (typeof window === "undefined") return defaultState;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultState;
    return { ...defaultState, ...JSON.parse(raw) };
  } catch {
    return defaultState;
  }
}

export function saveWidgetRailState(state: WidgetRailState): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // ignore
  }
}

export function toggleWidget(
  state: WidgetRailState,
  widget: WidgetId,
): WidgetRailState {
  const active = state.activeWidgets.includes(widget)
    ? state.activeWidgets.filter((w) => w !== widget)
    : [...state.activeWidgets, widget];
  return { ...state, activeWidgets: active, open: active.length > 0 || state.open };
}
