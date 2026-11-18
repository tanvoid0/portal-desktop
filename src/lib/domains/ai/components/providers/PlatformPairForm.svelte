<!-- One-shot pairing with the platform master key (never held in state, never stored). -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    pairing: boolean;
    onPair: (key: string) => Promise<void>;
  }

  let { pairing, onPair }: Props = $props();

  // The master key is only ever read from the DOM node inside pair(); it is
  // never held in component state.
  let masterKeyInput = $state<HTMLInputElement | null>(null);

  async function pair() {
    const key = masterKeyInput?.value ?? '';
    if (!key.trim()) return;
    try {
      await onPair(key);
    } finally {
      if (masterKeyInput) masterKeyInput.value = '';
    }
  }
</script>

<div class="flex gap-2">
  <Input
    bind:ref={masterKeyInput}
    type="password"
    autocomplete="off"
    spellcheck={false}
    class="font-mono text-sm"
    placeholder="Platform master key (used once, never stored)"
    aria-label="Platform master key"
    onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && pair()}
  />
  <Button size="sm" type="button" disabled={pairing} onclick={pair}>
    {#if pairing}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
    Pair
  </Button>
</div>
