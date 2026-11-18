<!-- Live terminal output for a running custom script, with an input line for interactive prompts. -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import TerminalOutputBox from '$lib/components/shared/TerminalOutputBox.svelte';

  interface Props {
    output: string;
    isRunning: boolean;
    onSendInput: (input: string) => void;
  }

  let { output, isRunning, onSendInput }: Props = $props();

  let outputContainer = $state<HTMLDivElement | null>(null);

  // Auto-scroll output to bottom when new content arrives
  $effect(() => {
    if (outputContainer && output) {
      // Use requestAnimationFrame to ensure DOM is updated
      requestAnimationFrame(() => {
        if (outputContainer) {
          outputContainer.scrollTop = outputContainer.scrollHeight;
        }
      });
    }
  });

  // Auto-focus input when script starts running
  $effect(() => {
    if (isRunning) {
      // Small delay to ensure the input is rendered
      setTimeout(() => {
        const input = document.getElementById('interactive-input') as HTMLInputElement;
        if (input) {
          input.focus();
        }
      }, 100);
    }
  });
</script>

<SectionCard title="Output" titleClass="" contentClass="space-y-4">
  {#snippet actions()}
    {#if isRunning}
      <Badge variant="outline">Interactive - Type below to send input</Badge>
    {/if}
  {/snippet}
  <TerminalOutputBox bind:el={outputContainer} class="h-64 rounded-md">
    {output || 'Waiting for output...'}
  </TerminalOutputBox>

  {#if isRunning}
    <div class="flex gap-2">
      <Input
        id="interactive-input"
        type="text"
        placeholder="Type input and press Enter (e.g., password for sudo)..."
        class="flex-1"
        autofocus
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            const input = (e.target as HTMLInputElement).value + '\n';
            onSendInput(input);
            (e.target as HTMLInputElement).value = '';
          }
        }}
      />
    </div>
  {/if}
</SectionCard>
