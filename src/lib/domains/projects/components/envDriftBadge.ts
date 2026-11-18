import type { BadgeVariant } from '$lib/components/ui/badge';
import type { EnvLinkView } from '../generated';

const WARNING_BADGE = 'border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-400';

export function driftBadge(state: EnvLinkView['state']): {
  label: string;
  variant: BadgeVariant;
  class?: string;
} {
  switch (state) {
    case 'inSync':
      return { label: 'Linked', variant: 'outline' };
    case 'fileChanged':
      return { label: 'Changed on disk', variant: 'outline', class: WARNING_BADGE };
    case 'credentialChanged':
      return { label: 'Shared value updated', variant: 'outline', class: WARNING_BADGE };
    case 'differs':
      return { label: 'Differs from shared', variant: 'destructive' };
    case 'notInFile':
      return { label: 'Not in file', variant: 'outline', class: 'text-muted-foreground' };
    case 'unknown':
      return { label: 'Unlock to compare', variant: 'outline', class: 'text-muted-foreground' };
  }
}
