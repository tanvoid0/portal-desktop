<script lang="ts">
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';

  let {
    checked = $bindable(false),
    id,
    label,
  }: { checked?: boolean; id: string; label: string } = $props();
</script>

<div class="flex items-center gap-1.5">
  <Checkbox bind:checked {id} />
  <Label for={id} class="text-xs font-normal text-muted-foreground">{label}</Label>
</div>
