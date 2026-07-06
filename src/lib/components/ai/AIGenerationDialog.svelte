<script lang="ts">
  import AIGenerationDialogBase from "$lib/domains/ai/components/shared/AIGenerationDialog.svelte";

  interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess: (result: unknown, originalInput: string) => void;
    config: import("$lib/domains/ai/components/shared/AIGenerationDialog.svelte").AIGenerationConfig;
    previousResult?: unknown;
  }

  let { open, onOpenChange, onSuccess, config, previousResult }: Props =
    $props();
</script>

<AIGenerationDialogBase
  {open}
  {onOpenChange}
  {onSuccess}
  {config}
  {previousResult}
/>
