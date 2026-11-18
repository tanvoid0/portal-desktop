<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Label } from '$lib/components/ui/label';

  interface Props {
    label: string;
    children: Snippet;
  }
  let { label, children }: Props = $props();
</script>

<div>
  <Label class="text-sm font-medium text-muted-foreground">{label}</Label>
  {@render children()}
</div>
