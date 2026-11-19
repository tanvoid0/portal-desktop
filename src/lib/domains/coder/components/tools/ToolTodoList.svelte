<script lang="ts">
  import Check from '~icons/fa6-solid/check';
  import Spinner from '~icons/fa6-solid/spinner';
  import type { ToolBodyProps } from './types.js';

  let { args }: ToolBodyProps = $props();

  const todos = $derived(
    (Array.isArray(args.todos) ? args.todos : []) as Array<Record<string, unknown>>
  );
</script>

<ul class="space-y-0.5 text-xs">
  {#each todos as todo, i (i)}
    {@const status = String(todo.status ?? 'pending')}
    <li class="flex items-start gap-1.5 {status === 'completed' ? 'text-muted-foreground' : ''}">
      {#if status === 'completed'}
        <Check class="mt-0.5 h-3 w-3 shrink-0" />
      {:else if status === 'in_progress'}
        <Spinner class="mt-0.5 h-3 w-3 shrink-0 animate-spin text-primary" />
      {:else}
        <span
          class="mt-0.5 inline-block h-3 w-3 shrink-0 rounded-full border border-muted-foreground/60"
          aria-hidden="true"
        ></span>
      {/if}
      <span class={status === 'completed' ? 'line-through' : ''}>
        {String((status === 'in_progress' && todo.activeForm) || todo.content || '')}
      </span>
    </li>
  {/each}
</ul>
