<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Check from '~icons/fa6-solid/check';
  import X from '~icons/fa6-solid/xmark';
  import type { Hunk } from '../types.js';

  interface Props {
    changeId: string;
    hunk: Hunk;
    isPending: boolean;
    reviewed: boolean;
    focused: boolean;
    onFocus: () => void;
    onAccept: () => void;
    onReject: () => void;
  }

  let {
    changeId,
    hunk: h,
    isPending,
    reviewed,
    focused,
    onFocus,
    onAccept,
    onReject,
  }: Props = $props();

  function display(lines: string[]): string {
    return lines.map((l) => l.replace(/\n$/, '')).join('\n');
  }
</script>

<!--
  A labelled group, NOT role="button". This div wraps the real
  Accept/Reject Button elements, and a nested button role is both
  invalid ARIA and unusable: the outer role's accessible name is
  computed from its descendants, so it announced the whole diff plus both
  button labels, and `getByRole('button', { name: 'Reject hunk' })`
  resolved to this wrapper instead of the button — the click then
  landed on the focus handler and silently did nothing (QA-083).
  `aria-label` keeps the name off the children either way.
-->
<div
  data-hunk-id="{changeId}:{h.index}"
  role="group"
  aria-label="Hunk {h.index + 1}, line {h.before_start + 1}"
  aria-current={focused ? 'true' : undefined}
  tabindex="0"
  class="p-2 transition-colors {focused
    ? 'bg-primary/5 ring-2 ring-primary/30 ring-inset'
    : reviewed
      ? 'opacity-70'
      : ''}"
  onclick={() => {
    if (isPending) onFocus();
  }}
  onkeydown={(e) => {
    if (isPending && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault();
      onFocus();
    }
  }}
>
  <div class="mb-1 flex items-center justify-between gap-2">
    <div class="flex items-center gap-2">
      <span class="text-xs text-muted-foreground">
        hunk {h.index + 1} · line {h.before_start + 1}
      </span>
      {#if reviewed}
        <Badge variant={h.accepted ? 'secondary' : 'destructive'} class="text-xs">
          {h.accepted ? 'accepted' : 'rejected'}
        </Badge>
      {/if}
    </div>

    {#if isPending}
      <div class="flex items-center gap-1">
        <Button
          size="sm"
          variant={focused && !reviewed ? 'secondary' : 'ghost'}
          class="h-6 text-xs"
          onclick={(e) => {
            e.stopPropagation();
            onAccept();
          }}
        >
          <Check class="mr-1 h-3 w-3" />
          {reviewed ? 'Accept' : 'Accept hunk'}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          class="h-6 text-xs text-destructive hover:text-destructive"
          onclick={(e) => {
            e.stopPropagation();
            onReject();
          }}
        >
          <X class="mr-1 h-3 w-3" />
          {reviewed ? 'Reject' : 'Reject hunk'}
        </Button>
      </div>
    {/if}
  </div>
  {#if h.before_lines.length}
    <pre
      class="overflow-auto rounded bg-diff-del/10 px-2 py-1 text-xs {h.accepted
        ? ''
        : 'opacity-50'}"><code>{display(h.before_lines).replace(/^/gm, '- ')}</code></pre>
  {/if}
  {#if h.after_lines.length}
    <pre
      class="overflow-auto rounded bg-diff-add/10 px-2 py-1 text-xs {h.accepted
        ? ''
        : 'line-through opacity-50'}"><code>{display(h.after_lines).replace(/^/gm, '+ ')}</code
      ></pre>
  {/if}
</div>
