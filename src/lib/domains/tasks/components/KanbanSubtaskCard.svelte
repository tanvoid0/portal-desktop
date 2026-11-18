<script lang="ts">
  import FaTurnDown from '~icons/fa6-solid/turn-down';
  import FaFlag from '~icons/fa6-solid/flag';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Icon from '@iconify/svelte';
  import ContextMenuList from '$lib/components/menu/ContextMenuList.svelte';
  import type { TaskDisplay } from '../utils/taskDisplay';
  import { taskStatusLabel, type Task } from '../types';
  import type { buildTaskMenu } from '../utils/taskMenu';
  import { stripMarkdown } from '../utils/stripMarkdown';

  interface Props {
    subtask: Task;
    menuItems: ReturnType<typeof buildTaskMenu>;
    display: TaskDisplay;
    readonly: boolean;
    onOpen: () => void;
    onToggleStatus: () => void;
  }
  let { subtask, menuItems, display, readonly, onOpen, onToggleStatus }: Props = $props();
</script>

<ContextMenuList items={menuItems}>
  {#snippet trigger(menuProps)}
    <Card
      {...menuProps}
      onclick={onOpen}
      onkeydown={(e: KeyboardEvent) => {
        if (e.target !== e.currentTarget) return;
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onOpen();
        }
      }}
      role="group"
      tabindex={0}
      aria-label="Subtask: {subtask.title}"
      borderAccent="left"
      class="mt-2 ml-4 cursor-pointer gap-0 py-0 hover:shadow-md"
    >
      <CardContent class="space-y-2 p-3">
        <div class="flex items-start gap-2">
          <FaTurnDown class="mt-1 h-3 w-3 shrink-0 text-primary" />
          <h4 class="line-clamp-2 min-w-0 flex-1 text-sm font-medium break-words text-foreground">
            {subtask.title}
          </h4>
          <span
            class="shrink-0 pt-0.5"
            title="{subtask.priority} priority"
            aria-label="{subtask.priority} priority"
          >
            <FaFlag class="h-3 w-3 {display.priorityColor(subtask.priority)}" />
          </span>
        </div>

        {#if subtask.description}
          <p class="line-clamp-2 text-xs text-muted-foreground">
            {stripMarkdown(subtask.description)}
          </p>
        {/if}

        <div class="flex items-center justify-between gap-2">
          <Badge class={display.statusBadge(subtask.status)}>
            {taskStatusLabel(subtask.status)}
          </Badge>
          {#if !readonly}
            <Button
              onclick={(e) => {
                e.stopPropagation();
                onToggleStatus();
              }}
              variant="ghost"
              size="sm"
              class="h-8 w-8 shrink-0 p-0"
              aria-label="Toggle subtask status"
              title="Toggle subtask status"
            >
              <Icon
                icon={display.icon(subtask)}
                class="h-4 w-4 {display.statusColor(subtask.status)}"
              />
            </Button>
          {/if}
        </div>
      </CardContent>
    </Card>
  {/snippet}
</ContextMenuList>
