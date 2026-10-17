<!--
	Modules of a monorepo project grouped by role, first few shown with a toggle for the rest
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Box } from '@lucide/svelte';
  import type { ProjectModule } from '$lib/domains/projects/types';
  import { stackIcon, stackLabel } from '$lib/domains/projects/utils/moduleStacks';

  interface Props {
    modules: ProjectModule[];
    limit?: number;
  }

  let { modules, limit = 6 }: Props = $props();

  const ROLES: { role: ProjectModule['role']; label: string }[] = [
    { role: 'app', label: 'Apps' },
    { role: 'service', label: 'Services' },
    { role: 'package', label: 'Packages' },
  ];

  let showAll = $state(false);

  const visible = $derived(showAll ? modules : modules.slice(0, limit));
  const groups = $derived(
    ROLES.map(({ role, label }) => ({
      label,
      items: visible.filter((m) => m.role === role),
    })).filter((g) => g.items.length > 0)
  );
</script>

<div class="space-y-2">
  <div class="max-h-56 space-y-2 overflow-y-auto pr-1">
    {#each groups as group (group.label)}
      <div>
        <p class="mb-1 text-xs font-medium uppercase tracking-wide text-muted-foreground">
          {group.label}
        </p>
        <ul class="space-y-0.5">
          {#each group.items as m (m.id)}
            <li class="flex items-center gap-2 text-sm" title={m.rel_path}>
              {#if stackIcon(m.stack)}
                <i class="{stackIcon(m.stack)} w-4 shrink-0 text-center"></i>
              {:else}
                <Box class="h-4 w-4 shrink-0 text-muted-foreground" />
              {/if}
              <span class="truncate">{m.name}</span>
              <span class="ml-auto shrink-0 text-xs text-muted-foreground">
                {stackLabel(m.stack)}
              </span>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>
  {#if modules.length > limit}
    <Button
      variant="link"
      size="sm"
      class="h-auto p-0 text-xs text-muted-foreground"
      onclick={() => (showAll = !showAll)}
    >
      {showAll ? 'Show fewer' : `Show all ${modules.length}`}
    </Button>
  {/if}
</div>
