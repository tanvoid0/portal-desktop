<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Label } from '$lib/components/ui/label';

  interface Props {
    label: string;
    /** Id of the control the label points at. */
    for?: string;
    error?: string;
    children: Snippet;
  }
  let { label, for: htmlFor, error, children }: Props = $props();
</script>

<div class="space-y-2">
  <Label for={htmlFor}>{label}</Label>
  {@render children()}
  {#if error}
    <p class="text-sm text-destructive">{error}</p>
  {/if}
</div>
