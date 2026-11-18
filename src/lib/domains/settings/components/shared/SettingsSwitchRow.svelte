<!-- Label (+ optional description) on the left, Switch on the right. -->
<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';

  interface Props {
    id: string;
    label: string;
    description?: string;
    checked: boolean;
    disabled?: boolean;
    onToggle: () => void;
  }

  let { id, label, description, checked, disabled = false, onToggle }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <div class="space-y-0.5">
    <Label for={id}>{label}</Label>
    {#if description}<p class="text-sm text-muted-foreground">{description}</p>{/if}
  </div>
  <Switch {id} {checked} {disabled} onclick={onToggle} />
</div>
