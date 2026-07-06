import type { Component } from "svelte";
import {
  CheckCircle2,
  Circle,
  Loader2,
  XCircle,
  Ban,
  SkipForward,
} from "@lucide/svelte";
import type { ExecutionStatus } from "../types";

export function getExecutionStatusIcon(
  status: ExecutionStatus | string,
): Component {
  switch (status) {
    case "success":
      return CheckCircle2;
    case "failed":
      return XCircle;
    case "running":
      return Loader2;
    case "cancelled":
      return Ban;
    case "skipped":
      return SkipForward;
    default:
      return Circle;
  }
}

export function getExecutionStatusColor(status: ExecutionStatus | string): string {
  switch (status) {
    case "success":
      return "text-green-500";
    case "failed":
      return "text-red-500";
    case "running":
      return "text-blue-500";
    case "cancelled":
      return "text-yellow-500";
    case "skipped":
      return "text-muted-foreground";
    default:
      return "text-muted-foreground";
  }
}

export function getExecutionStatusBadgeVariant(
  status: ExecutionStatus | string,
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

export function formatExecutionDuration(
  startedAt: Date | string,
  finishedAt?: Date | string | null,
): string {
  const start =
    typeof startedAt === "string" ? new Date(startedAt) : startedAt;
  if (!finishedAt) return "In progress";

  const end =
    typeof finishedAt === "string" ? new Date(finishedAt) : finishedAt;
  const ms = end.getTime() - start.getTime();
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${Math.round(ms / 1000)}s`;
  const minutes = Math.floor(ms / 60_000);
  const seconds = Math.round((ms % 60_000) / 1000);
  return `${minutes}m ${seconds}s`;
}
