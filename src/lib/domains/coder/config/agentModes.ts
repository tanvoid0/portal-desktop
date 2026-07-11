import type { CoderAgentMode } from "../types.js";

export interface AgentModeConfig {
  value: CoderAgentMode;
  label: string;
  hint: string;
  dotClass: string;
  pillClass: string;
  itemClass: string;
}

export const AGENT_MODE_CYCLE: CoderAgentMode[] = [
  "plan",
  "debug",
  "multitask",
  "ask",
  "auto",
];

export const AGENT_MODES: AgentModeConfig[] = [
  {
    value: "plan",
    label: "Plan",
    hint: "Explore and write a .plan.md",
    dotClass: "bg-violet-500",
    pillClass: "border-violet-500/30 bg-violet-500/10 text-violet-700 dark:text-violet-300",
    itemClass: "focus:bg-violet-500/10 data-[state=checked]:text-violet-700 dark:data-[state=checked]:text-violet-300",
  },
  {
    value: "debug",
    label: "Debug",
    hint: "Hypothesis-driven bug fixing",
    dotClass: "bg-amber-500",
    pillClass: "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300",
    itemClass: "focus:bg-amber-500/10 data-[state=checked]:text-amber-700 dark:data-[state=checked]:text-amber-300",
  },
  {
    value: "multitask",
    label: "Multitask",
    hint: "Parallel worktree agents",
    dotClass: "bg-purple-500",
    pillClass: "border-purple-500/30 bg-purple-500/10 text-purple-700 dark:text-purple-300",
    itemClass: "focus:bg-purple-500/10 data-[state=checked]:text-purple-700 dark:data-[state=checked]:text-purple-300",
  },
  {
    value: "ask",
    label: "Ask",
    hint: "Chat only, read-only",
    dotClass: "bg-teal-500",
    pillClass: "border-teal-500/30 bg-teal-500/10 text-teal-700 dark:text-teal-300",
    itemClass: "focus:bg-teal-500/10 data-[state=checked]:text-teal-700 dark:data-[state=checked]:text-teal-300",
  },
  {
    value: "auto",
    label: "Auto",
    hint: "Decides and adapts per task",
    dotClass: "bg-primary",
    pillClass: "border-primary/30 bg-primary/10 text-foreground",
    itemClass: "focus:bg-primary/10 data-[state=checked]:text-foreground",
  },
];

export function getAgentModeConfig(mode: CoderAgentMode): AgentModeConfig {
  return AGENT_MODES.find((m) => m.value === mode) ?? AGENT_MODES[4];
}

export function nextAgentMode(current: CoderAgentMode): CoderAgentMode {
  const idx = AGENT_MODE_CYCLE.indexOf(current);
  const next = idx < 0 ? 0 : (idx + 1) % AGENT_MODE_CYCLE.length;
  return AGENT_MODE_CYCLE[next];
}

const AUTO_EFFECTIVE_LABELS: Partial<Record<CoderAgentMode, string>> = {
  plan: "planning",
  debug: "debugging",
  ask: "asking",
  multitask: "multitasking",
};

export function autoModeSubtitle(effective: CoderAgentMode | null): string | null {
  if (!effective || effective === "auto") return null;
  return AUTO_EFFECTIVE_LABELS[effective] ?? null;
}

/** Lightweight heuristic for Auto mode UI subtitle (v1). */
export function inferEffectiveMode(
  text: string,
  agentMode: CoderAgentMode,
): CoderAgentMode | null {
  if (agentMode !== "auto") return null;
  const issueUrls = text.match(/https?:\/\/github\.com\/[^\s]+\/issues\/\d+/gi) ?? [];
  if (issueUrls.length >= 2) return "multitask";
  const lower = text.toLowerCase();
  if (/\b(plan|design|architect|roadmap|implementation plan)\b/.test(lower)) return "plan";
  if (/\b(debug|bug|fix|error|crash|broken|failing|repro|stack trace)\b/.test(lower)) {
    return "debug";
  }
  if (
    /\b(what|how|explain|why|describe|tell me)\b/.test(lower) &&
    !/\b(fix|implement|add|create|build|write|refactor)\b/.test(lower)
  ) {
    return "ask";
  }
  return null;
}
