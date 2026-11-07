<!--
  "How to run this": start / build / test commands with Run buttons and inline editing.
-->

<script lang="ts">
  import Check from '~icons/fa6-solid/check';
  import Edit from '~icons/fa6-solid/pen-to-square';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Play from '~icons/fa6-solid/play';
  import Plus from '~icons/fa6-solid/plus';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Terminal from '~icons/fa6-solid/terminal';
  import X from '~icons/fa6-solid/xmark';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { projectService } from '$lib/domains/projects';
  import { logger } from '$lib/domains/shared/services/logger';
  import { toast } from '$lib/utils/toast';
  import type { Project } from '$lib/domains/projects/types';
  import type { ProjectCommandKind } from '$lib/domains/projects/utils/projectCommandRunner';

  type Field = 'start_command' | 'build_command' | 'test_command' | 'output_directory';

  interface Props {
    project: Project;
    runningCommand: ProjectCommandKind | null;
    refreshing?: boolean;
    onRun: (kind: ProjectCommandKind) => void;
    onRedetect: () => void;
    onSaved: () => void | Promise<unknown>;
  }

  let { project, runningCommand, refreshing = false, onRun, onRedetect, onSaved }: Props = $props();

  const log = logger.createScoped('ProjectCommands');

  const ROWS: {
    field: Field;
    kind?: ProjectCommandKind;
    label: string;
    help: string;
    placeholder: string;
  }[] = [
    {
      field: 'start_command',
      kind: 'start',
      label: 'Start',
      help: 'runs the app on your machine',
      placeholder: 'npm run dev',
    },
    {
      field: 'build_command',
      kind: 'build',
      label: 'Build',
      help: 'packages the app so it can be shipped',
      placeholder: 'npm run build',
    },
    {
      field: 'test_command',
      kind: 'test',
      label: 'Test',
      help: 'checks that the code still works',
      placeholder: 'npm test',
    },
    {
      field: 'output_directory',
      label: 'Build output folder',
      help: 'where the build puts its files',
      placeholder: 'dist',
    },
  ];

  let editingField = $state<Field | null>(null);
  let editValue = $state('');
  let saving = $state(false);

  const hasRunnable = $derived(
    Boolean(project.start_command || project.build_command || project.test_command)
  );

  function startEditing(field: Field) {
    editingField = field;
    editValue = project[field] ?? '';
  }

  async function save() {
    if (!editingField) return;
    saving = true;
    try {
      await projectService.updateProject(project.id, {
        // '' (not undefined) so clearing a command actually saves; undefined is dropped.
        [editingField]: editValue.trim(),
      });
      editingField = null;
      await onSaved();
    } catch (err) {
      log.error('Failed to save command', err);
      toast.error('Failed to save. Please try again.');
    } finally {
      saving = false;
    }
  }
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Terminal class="h-5 w-5" />
      How to run this
    </CardTitle>
  </CardHeader>
  <CardContent class="space-y-4">
    {#if !hasRunnable && editingField === null}
      <div class="rounded-md border border-dashed p-6 text-center">
        <p class="mb-3 text-sm text-muted-foreground">
          No commands were detected for this project. Re-detect from the project folder, or add the
          command you normally type in a terminal.
        </p>
        <div class="flex flex-wrap justify-center gap-2">
          <Button size="sm" onclick={() => startEditing('start_command')} class="gap-2">
            <Plus class="h-4 w-4" />
            Add command
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={onRedetect}
            disabled={refreshing}
            class="gap-2"
          >
            <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
            Re-detect
          </Button>
        </div>
      </div>
    {:else}
      {#each ROWS as row (row.field)}
        {@const value = project[row.field]}
        <div class="space-y-1">
          <div class="flex items-center justify-between gap-2">
            <div class="min-w-0">
              <span class="text-sm font-medium">{row.label}</span>
              <span class="text-sm text-muted-foreground"> — {row.help}</span>
              {#if row.kind === 'start' && project.dev_port}
                <Badge variant="outline" class="ml-1">Port {project.dev_port}</Badge>
              {/if}
            </div>
            {#if editingField !== row.field}
              <div class="flex shrink-0 items-center gap-1">
                {#if row.kind && value}
                  {@const kind = row.kind}
                  <Button
                    size="sm"
                    onclick={() => onRun(kind)}
                    disabled={runningCommand !== null}
                    class="h-7 gap-1 px-2"
                  >
                    {#if runningCommand === kind}
                      <Loader2 class="h-3 w-3 animate-spin" />
                    {:else}
                      <Play class="h-3 w-3" />
                    {/if}
                    Run
                  </Button>
                {/if}
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => startEditing(row.field)}
                  class="h-7 px-2"
                  aria-label={value ? `Edit ${row.label}` : `Add ${row.label}`}
                >
                  {#if value}<Edit class="h-3 w-3" />{:else}<Plus class="h-3 w-3" />{/if}
                </Button>
              </div>
            {/if}
          </div>
          {#if editingField === row.field}
            <div class="flex items-center gap-2">
              <Input
                bind:value={editValue}
                class="font-mono text-sm"
                placeholder={row.placeholder}
                onkeydown={(e) => {
                  if (e.key === 'Enter') save();
                  if (e.key === 'Escape') editingField = null;
                }}
              />
              <Button
                variant="ghost"
                size="sm"
                onclick={save}
                disabled={saving}
                class="h-8"
                aria-label="Save"
              >
                <Check class="h-4 w-4 text-green-600" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => (editingField = null)}
                disabled={saving}
                class="h-8"
                aria-label="Cancel"
              >
                <X class="h-4 w-4 text-red-600" />
              </Button>
            </div>
          {:else if value}
            <p class="rounded bg-muted p-2 font-mono text-sm break-all">{value}</p>
          {:else}
            <p class="text-sm text-muted-foreground">Not set</p>
          {/if}
        </div>
      {/each}
    {/if}
  </CardContent>
</Card>
