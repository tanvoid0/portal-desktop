<!-- One .env variable: masked/revealed value, drift badge for its shared-credential link, row actions. -->
<script lang="ts">
  import { TableCell, TableRow } from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import Eye from '~icons/fa6-solid/eye';
  import EyeOff from '~icons/fa6-solid/eye-slash';
  import Ellipsis from '~icons/fa6-solid/ellipsis';
  import Copy from '~icons/fa6-solid/copy';
  import LinkIcon from '~icons/fa6-solid/link';
  import UnlinkIcon from '~icons/fa6-solid/link-slash';
  import Plus from '~icons/fa6-solid/plus';
  import ArrowRight from '~icons/fa6-solid/arrow-right';
  import Check from '~icons/fa6-solid/check';
  import PenToSquare from '~icons/fa6-solid/pen-to-square';
  import type { EnvLinkView, EnvVarView } from '../generated';
  import { driftBadge } from './envDriftBadge';

  interface Props {
    envVar: EnvVarView;
    fileName: string;
    otherFiles?: string[];
    onCopyTo?: (target: string) => void;
    onMoveTo?: (target: string) => void;
    onEdit?: () => void;
    isRevealed: boolean;
    onToggleReveal: () => void;
    onUnlockToCompare: () => void;
    onCopyShared: (link: EnvLinkView) => void;
    onUpdateShared: (envVar: EnvVarView) => void;
    onResync: (link: EnvLinkView) => void;
    onUnlink: (link: EnvLinkView, envKey: string) => void;
    onLink: (target: { envKey: string; fileName: string }) => void;
    onCreate: (target: { envKey: string; fileName: string; value: string }) => void;
  }

  let {
    envVar,
    fileName,
    otherFiles = [],
    onCopyTo = () => {},
    onMoveTo = () => {},
    onEdit = () => {},
    isRevealed,
    onToggleReveal,
    onUnlockToCompare,
    onCopyShared,
    onUpdateShared,
    onResync,
    onUnlink,
    onLink,
    onCreate,
  }: Props = $props();
</script>

<TableRow>
  <TableCell class="truncate font-mono" title={envVar.key}>{envVar.key}</TableCell>
  <TableCell>
    <div class="flex min-w-0 items-center gap-2">
      <div class="min-w-0 flex-1">
        {#if envVar.value === ''}
          <Badge variant="outline" class="font-normal text-muted-foreground italic">empty</Badge>
        {:else if isRevealed}
          <span class="block truncate font-mono tabular-nums" title={envVar.value}>
            {envVar.value}
          </span>
        {:else}
          <span class="font-mono text-muted-foreground">••••••••</span>
        {/if}
      </div>
      {#if envVar.value !== ''}
        <Button
          variant="ghost"
          size="icon-sm"
          class="shrink-0"
          aria-label={isRevealed ? `Hide value of ${envVar.key}` : `Reveal value of ${envVar.key}`}
          onclick={() => onToggleReveal()}
        >
          {#if isRevealed}
            <EyeOff class="h-3.5 w-3.5" />
          {:else}
            <Eye class="h-3.5 w-3.5" />
          {/if}
        </Button>
      {/if}
    </div>
  </TableCell>
  <TableCell>
    {#if envVar.link}
      {@const badge = driftBadge(envVar.link.state)}
      <div class="flex flex-wrap items-center gap-1.5">
        <Badge variant="secondary">→ {envVar.link.credentialName}</Badge>
        {#if envVar.link.state === 'unknown'}
          <Button variant="ghost" size="sm" class="h-auto gap-1 p-0" onclick={onUnlockToCompare}>
            <Badge variant={badge.variant} class={badge.class}>
              {badge.label}
            </Badge>
          </Button>
        {:else}
          <Badge variant={badge.variant} class={badge.class}>{badge.label}</Badge>
        {/if}
      </div>
    {/if}
  </TableCell>
  <TableCell>
    <DropdownMenu>
      <DropdownMenuTrigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="ghost"
            size="icon-lg"
            aria-label={`Actions for ${envVar.key}`}
          >
            <Ellipsis class="h-4 w-4" />
          </Button>
        {/snippet}
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onclick={() => onEdit()}>
          <PenToSquare class="mr-2 h-4 w-4" />
          Edit value…
        </DropdownMenuItem>
        {#each otherFiles as target (target)}
          <DropdownMenuItem onclick={() => onCopyTo(target)}>
            <Copy class="mr-2 h-4 w-4" />
            Copy to {target}
          </DropdownMenuItem>
        {/each}
        {#each otherFiles as target (target)}
          <DropdownMenuItem onclick={() => onMoveTo(target)}>
            <ArrowRight class="mr-2 h-4 w-4" />
            Move to {target}
          </DropdownMenuItem>
        {/each}
        <DropdownMenuSeparator />
        {#if envVar.link}
          <DropdownMenuItem onclick={() => onCopyShared(envVar.link!)}>
            <Copy class="mr-2 h-4 w-4" />
            Copy shared value
          </DropdownMenuItem>
          <DropdownMenuItem onclick={() => onUpdateShared(envVar)}>
            <PenToSquare class="mr-2 h-4 w-4" />
            Update shared credential from this file value
          </DropdownMenuItem>
          <DropdownMenuItem onclick={() => onResync(envVar.link!)}>
            <Check class="mr-2 h-4 w-4" />
            Acknowledge (re-sync)
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            variant="destructive"
            onclick={() => onUnlink(envVar.link!, envVar.key)}
          >
            <UnlinkIcon class="mr-2 h-4 w-4" />
            Unlink
          </DropdownMenuItem>
        {:else}
          <DropdownMenuItem onclick={() => onLink({ envKey: envVar.key, fileName })}>
            <LinkIcon class="mr-2 h-4 w-4" />
            Link to shared credential…
          </DropdownMenuItem>
          <DropdownMenuItem
            onclick={() => onCreate({ envKey: envVar.key, fileName, value: envVar.value })}
          >
            <Plus class="mr-2 h-4 w-4" />
            Create shared credential from this value…
          </DropdownMenuItem>
        {/if}
      </DropdownMenuContent>
    </DropdownMenu>
  </TableCell>
</TableRow>
