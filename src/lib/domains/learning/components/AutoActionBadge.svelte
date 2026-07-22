<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";

  interface Props {
    actionType: string;
    autonomous?: boolean;
    safetyLevel?: "safe" | "low-risk" | "medium-risk" | "high-risk";
    confidence?: number;
  }

  const {
    actionType,
    autonomous = false,
    safetyLevel = "low-risk",
    confidence = 0,
  }: Props = $props();

  const badgeVariant = $derived.by(() => {
    if (!autonomous) return "secondary" as const;
    if (safetyLevel === "high-risk") return "destructive" as const;
    return "outline" as const;
  });

  const safetyLabel = $derived.by(() => {
    if (!autonomous) return "Manual";
    switch (safetyLevel) {
      case "safe":
        return "Auto (Safe)";
      case "low-risk":
        return "Auto (Low Risk)";
      case "medium-risk":
        return "Auto (Medium Risk)";
      case "high-risk":
        return "Auto (High Risk)";
      default:
        return "Auto";
    }
  });
</script>

<Badge
  variant={badgeVariant}
  class="gap-1"
  title="Action type: {actionType}, Confidence: {confidence > 0
    ? (confidence * 100).toFixed(0) + '%'
    : 'N/A'}"
>
  <span>{autonomous ? "🤖" : "⚡"}</span>
  <span>{safetyLabel}</span>
  {#if confidence > 0}
    <span class="opacity-60">({(confidence * 100).toFixed(0)}%)</span>
  {/if}
</Badge>
