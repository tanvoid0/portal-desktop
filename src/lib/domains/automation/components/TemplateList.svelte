<!--
	Pipeline template grid — search filtering, the card, and the empty state.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import Download from '~icons/fa6-solid/download';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Trash2 from '~icons/fa6-solid/trash';
  import FileText from '~icons/fa6-solid/file-lines';
  import type { PipelineTemplate } from '$lib/domains/projects/pipelines';
  import { PageEmpty } from '$lib/components/shell';

  interface Props {
    templates: PipelineTemplate[];
    search: string;
    onImport?: () => void;
    onClearFilters?: () => void;
    onEdit?: (template: PipelineTemplate) => void;
    onExport?: (template: PipelineTemplate) => void;
    onDelete?: (template: PipelineTemplate) => void;
  }

  let {
    templates,
    search,
    onImport = () => {},
    onClearFilters = () => {},
    onEdit = () => {},
    onExport = () => {},
    onDelete = () => {},
  }: Props = $props();

  const filteredTemplates = $derived.by(() => {
    if (!search) return templates;
    const query = search.toLowerCase();
    return templates.filter(
      (t) =>
        t.name.toLowerCase().includes(query) ||
        t.description.toLowerCase().includes(query) ||
        t.key.toLowerCase().includes(query) ||
        (t.framework && t.framework.toLowerCase().includes(query))
    );
  });
</script>

{#if templates.length === 0}
  <PageEmpty
    title="No templates yet"
    description="Import a pipeline template to get started."
    icon={FileText}
    actionLabel="Import Template"
    onAction={() => onImport()}
  />
{:else if filteredTemplates.length === 0}
  <!--
    Templates exist — the search box (shared with the Blocks tab) hid them all.
    Name the query rather than claiming there are no templates.
  -->
  <PageEmpty
    title="No matching templates"
    description={`No template matches “${search.trim()}”. There ${
      templates.length === 1 ? 'is 1 template' : `are ${templates.length} templates`
    } available.`}
    icon={FileText}
    actionLabel="Clear search"
    onAction={() => onClearFilters()}
  />
{:else}
  <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
    {#each filteredTemplates as template (template.key)}
      <Card class="gap-0 p-3">
        <div class="flex items-start gap-2">
          <div class="min-w-0 flex-1">
            <div class="flex min-w-0 items-center gap-2">
              <h3 class="truncate text-sm font-medium">{template.name}</h3>
              {#if !template.id}
                <Badge variant="outline" class="shrink-0 text-xs">Built-in</Badge>
              {/if}
            </div>
            <p class="line-clamp-1 text-xs text-muted-foreground">{template.description}</p>
          </div>
          <div class="flex shrink-0 items-center gap-0.5">
            <Button
              size="icon-sm"
              variant="ghost"
              aria-label={`Export template ${template.name}`}
              onclick={() => onExport(template)}
            >
              <Download class="h-3.5 w-3.5" />
            </Button>
            {#if template.id}
              <Button
                size="icon-sm"
                variant="ghost"
                aria-label={`Edit template ${template.name}`}
                onclick={() => onEdit(template)}
              >
                <Edit class="h-3.5 w-3.5" />
              </Button>
              <Button
                size="icon-sm"
                variant="ghost"
                class="text-destructive hover:text-destructive"
                aria-label={`Delete template ${template.name}`}
                onclick={() => onDelete(template)}
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            {/if}
          </div>
        </div>

        <div class="mt-2 flex flex-wrap items-center gap-1">
          <span class="text-xs text-muted-foreground">
            {template.steps.length} step{template.steps.length === 1 ? '' : 's'}
          </span>
          {#if template.framework}
            <Badge variant="outline" class="text-xs">{template.framework}</Badge>
          {/if}
          {#each (template.tags || []).slice(0, 2) as tag}
            <Badge variant="outline" class="text-xs">{tag}</Badge>
          {/each}
          {#if (template.tags || []).length > 2}
            <Badge variant="outline" class="text-xs">+{(template.tags || []).length - 2}</Badge>
          {/if}
        </div>
      </Card>
    {/each}
  </div>
{/if}
