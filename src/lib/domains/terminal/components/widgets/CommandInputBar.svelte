<script lang="ts">
  import CommandInput from "../ai/CommandInput.svelte";

  interface Props {
    tabId: string;
    onSubmit: (command: string, isAIMode?: boolean) => void;
    onStop?: () => void;
  }

  let { tabId, onSubmit, onStop }: Props = $props();
</script>

<div class="border-t border-gray-700 bg-gray-800 p-2">
  <CommandInput
    {tabId}
    {onSubmit}
    {onStop}
    placeholder="Enter command or /ai for AI mode..."
  />
</div>
