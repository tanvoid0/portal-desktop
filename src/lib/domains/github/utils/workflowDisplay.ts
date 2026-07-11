import type { Component } from "svelte";
import {
  CheckCircle2,
  Circle,
  Loader2,
  XCircle,
  Ban,
  Clock,
} from "@lucide/svelte";
import type { GitHubWorkflowRun } from "../types";

export type WorkflowDisplayStatus =
  | "success"
  | "failed"
  | "running"
  | "queued"
  | "cancelled"
  | "skipped"
  | "pending";

export function getWorkflowDisplayStatus(
  status: string,
  conclusion?: string | null,
): WorkflowDisplayStatus {
  if (status === "queued" || status === "waiting" || status === "requested") {
    return "queued";
  }
  if (status === "pending") {
    return "pending";
  }
  if (status === "in_progress") {
    return "running";
  }

  switch (conclusion) {
    case "success":
      return "success";
    case "failure":
    case "timed_out":
    case "action_required":
      return "failed";
    case "cancelled":
      return "cancelled";
    case "skipped":
    case "neutral":
    case "stale":
      return "skipped";
    default:
      return "pending";
  }
}

export function getWorkflowStatusIcon(status: WorkflowDisplayStatus): Component {
  switch (status) {
    case "success":
      return CheckCircle2;
    case "failed":
      return XCircle;
    case "running":
      return Loader2;
    case "cancelled":
      return Ban;
    case "queued":
    case "pending":
      return Clock;
    default:
      return Circle;
  }
}

export function getWorkflowStatusColor(status: WorkflowDisplayStatus): string {
  switch (status) {
    case "success":
      return "text-green-500";
    case "failed":
      return "text-red-500";
    case "running":
      return "text-blue-500";
    case "cancelled":
      return "text-yellow-500";
    case "queued":
    case "pending":
      return "text-muted-foreground";
    default:
      return "text-muted-foreground";
  }
}

export function getWorkflowStatusBadgeVariant(
  status: WorkflowDisplayStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "success":
      return "default";
    case "failed":
      return "destructive";
    case "running":
      return "secondary";
    default:
      return "outline";
  }
}

export function formatWorkflowDuration(
  startedAt?: string | null,
  completedAt?: string | null,
): string {
  if (!startedAt) return "Not started";
  const start = new Date(startedAt);
  if (!completedAt) return "In progress";

  const end = new Date(completedAt);
  const ms = end.getTime() - start.getTime();
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${Math.round(ms / 1000)}s`;
  const minutes = Math.floor(ms / 60_000);
  const seconds = Math.round((ms % 60_000) / 1000);
  return `${minutes}m ${seconds}s`;
}

export function isWorkflowRunActive(run: GitHubWorkflowRun): boolean {
  return (
    run.status === "in_progress" ||
    run.status === "queued" ||
    run.status === "waiting" ||
    run.status === "requested" ||
    run.status === "pending"
  );
}

export function isWorkflowJobActive(status: string): boolean {
  return (
    status === "in_progress" ||
    status === "queued" ||
    status === "waiting" ||
    status === "requested" ||
    status === "pending"
  );
}

export function isWorkflowJobLogsFetchable(status: string): boolean {
  return status === "in_progress" || status === "completed";
}

export const WORKFLOW_LOGS_UNAVAILABLE_PREFIX = "Logs are not available yet";

export function isWorkflowLogsUnavailableMessage(message?: string | null): boolean {
  if (!message) return false;
  return (
    message.startsWith(WORKFLOW_LOGS_UNAVAILABLE_PREFIX) ||
    message.startsWith("Log access expired or is temporarily unavailable")
  );
}
