<!-- Clickable severity badge; clicking filters the log list by that severity -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { getSeverityBadgeVariant } from '../../utils/logSeverity';

  interface Props {
    severity: string;
    onclick?: (severity: string) => void;
  }
  let { severity, onclick }: Props = $props();
</script>

<Button
  variant="ghost"
  onclick={() => onclick?.(severity)}
  class="h-auto cursor-pointer p-0 hover:opacity-80"
>
  <Badge variant={getSeverityBadgeVariant(severity)}>{severity}</Badge>
</Button>
