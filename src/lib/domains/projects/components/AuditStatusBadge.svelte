<!-- Dormant and stale were both plain outline badges; the dash separates the
	two without spending a second hue on it. -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { ageLabel } from '$lib/domains/projects/utils/auditFormat';

  interface Props {
    status: string;
    /** Omit to show the status alone. */
    ageDays?: number | null;
  }
  let { status, ageDays }: Props = $props();

  const variant = $derived(
    status === 'active' ? 'default' : status === 'recent' ? 'secondary' : 'outline'
  );
</script>

<Badge {variant} class={status === 'stale' ? 'border-dashed opacity-70' : ''}>
  {status}{ageDays === undefined ? '' : ` · ${ageLabel(ageDays)}`}
</Badge>
