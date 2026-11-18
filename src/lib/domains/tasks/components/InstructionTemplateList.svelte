<script lang="ts">
  import FaPencil from '~icons/fa6-solid/pencil';
  import FaTrash from '~icons/fa6-solid/trash';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import type { InstructionTemplate } from '../utils/instructionTemplates';

  interface Props {
    templates: InstructionTemplate[];
    onEdit: (id: string) => void;
    onDelete: (id: string) => void;
  }
  let { templates, onEdit, onDelete }: Props = $props();
</script>

<div class="max-h-48 space-y-2 overflow-y-auto">
  {#each templates as template (template.id)}
    <div class="flex items-center justify-between rounded-lg border p-2">
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <span class="truncate text-sm font-medium">{template.name}</span>
          {#if template.isDefault}
            <Badge variant="secondary" class="text-xs">Default</Badge>
          {/if}
        </div>
        <p class="truncate text-xs text-muted-foreground">
          {template.instruction}
        </p>
      </div>
      <div class="ml-2 flex items-center gap-1">
        {#if !template.isDefault}
          <Button variant="ghost" size="sm" onclick={() => onEdit(template.id)} class="h-7 w-7 p-0">
            <FaPencil class="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onclick={() => onDelete(template.id)}
            class="h-7 w-7 p-0 text-destructive"
          >
            <FaTrash class="h-3 w-3" />
          </Button>
        {/if}
      </div>
    </div>
  {/each}
</div>
