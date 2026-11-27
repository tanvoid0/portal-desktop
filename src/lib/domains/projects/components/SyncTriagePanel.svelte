<!-- The AI review of one repository's uncommitted work: questions, or a plan to apply. -->
<script lang="ts">
  import * as Collapsible from '$lib/components/ui/collapsible';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';
  import Spinner from '~icons/fa6-solid/spinner';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import { syncTriage } from '../state/syncTriage.svelte';
  import { hasWork } from '../utils/syncTriage';

  interface Props {
    projectId: number;
    disabled?: boolean;
  }
  let { projectId, disabled = false }: Props = $props();

  const item = $derived(syncTriage.items[projectId]);
  const answerable = $derived(!!item && item.questions.every((q) => q.answer.trim()));

  function pick(index: number, option: string) {
    if (!item) return;
    item.questions[index].answer = option;
    // A single question needs no second click.
    if (item.questions.length === 1) syncTriage.answer(projectId);
  }
</script>

<div class="space-y-2 rounded-md bg-muted/40 p-2" aria-live="polite">
  {#if !item}
    <div class="flex flex-wrap gap-2">
      <Button size="sm" {disabled} onclick={() => syncTriage.review(projectId)}>
        <Sparkles class="mr-1 h-3 w-3" /> Review with AI
      </Button>
      <Button
        size="sm"
        variant="ghost"
        {disabled}
        onclick={() => syncTriage.planEverything(projectId)}
      >
        Commit everything…
      </Button>
    </div>
  {:else if item.status === 'queued' || item.status === 'reviewing' || item.status === 'applying'}
    <div class="flex items-center gap-2 text-xs text-muted-foreground">
      <Spinner class="h-3 w-3 animate-spin" />
      {item.status === 'queued'
        ? 'Waiting for review'
        : item.status === 'reviewing'
          ? 'AI is reviewing the changes'
          : 'Applying'}
      {#if item.status !== 'applying'}
        <Button size="sm" variant="ghost" onclick={() => syncTriage.dismiss(projectId)}>Skip</Button
        >
      {/if}
    </div>
  {:else if item.status === 'input'}
    {#if item.summary}<p class="text-sm wrap-anywhere">{item.summary}</p>{/if}
    {#each item.questions as q, i (i)}
      <fieldset class="space-y-1">
        <legend class="text-sm font-medium wrap-anywhere">{q.question}</legend>
        <div class="flex flex-wrap gap-1">
          {#each q.options as option (option)}
            <Button
              size="sm"
              variant={q.answer === option ? 'default' : 'outline'}
              class="h-auto min-h-8 max-w-full whitespace-normal"
              aria-pressed={q.answer === option}
              {disabled}
              onclick={() => pick(i, option)}
            >
              {option}
            </Button>
          {/each}
        </div>
        <Input
          bind:value={q.answer}
          class="max-w-xl"
          placeholder="Or type your own answer"
          aria-label="Your answer: {q.question}"
          onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && syncTriage.answer(projectId)}
        />
      </fieldset>
    {/each}
    <div class="flex flex-wrap gap-2">
      <Button
        size="sm"
        disabled={disabled || !answerable}
        onclick={() => syncTriage.answer(projectId)}
      >
        Continue
      </Button>
      <Button size="sm" variant="ghost" onclick={() => syncTriage.dismiss(projectId)}>
        Leave it
      </Button>
    </div>
  {:else if item.status === 'ready'}
    {#if item.summary}<p class="text-sm wrap-anywhere">{item.summary}</p>{/if}
    {#if hasWork(item.plan)}
      {#each item.plan.commits as commit, i (i)}
        <div class="space-y-1">
          <Input
            bind:value={commit.message}
            aria-label="Commit message {i + 1}"
            class="max-w-3xl"
          />
          <Collapsible.Root class="text-xs text-muted-foreground">
            <Collapsible.Trigger class="group/files flex cursor-pointer items-center gap-1.5">
              <ChevronRight
                class="h-3 w-3 transition-transform group-data-[state=open]/files:rotate-90"
              />
              {commit.files.length}
              {commit.files.length === 1 ? 'file' : 'files'} in this commit
            </Collapsible.Trigger>
            <Collapsible.Content>
              <ul class="max-h-40 overflow-y-auto pl-4 font-mono">
                {#each commit.files as file (file)}<li class="wrap-anywhere">{file}</li>{/each}
              </ul>
            </Collapsible.Content>
          </Collapsible.Root>
        </div>
      {/each}
      {#if item.plan.ignore.length}
        <p class="text-xs wrap-anywhere text-muted-foreground">
          Add to .gitignore: <span class="font-mono">{item.plan.ignore.join(', ')}</span>
        </p>
      {/if}
      <div class="flex flex-wrap items-center gap-3">
        <Button size="sm" {disabled} onclick={() => syncTriage.apply(projectId)}>Apply</Button>
        <div class="flex items-center gap-1.5">
          <Checkbox id="push-{projectId}" bind:checked={item.plan.push} />
          <Label for="push-{projectId}" class="text-xs">Push afterwards</Label>
        </div>
        <Button size="sm" variant="ghost" onclick={() => syncTriage.review(projectId)}>
          Review again
        </Button>
        <Button size="sm" variant="ghost" onclick={() => syncTriage.dismiss(projectId)}>
          Leave it
        </Button>
      </div>
    {:else}
      <div class="flex flex-wrap items-center gap-2">
        <p class="text-xs text-muted-foreground">Nothing to commit or ignore.</p>
        <Button size="sm" variant="ghost" onclick={() => syncTriage.dismiss(projectId)}>
          Dismiss
        </Button>
      </div>
    {/if}
  {:else}
    <p
      class="text-xs wrap-anywhere {item.status === 'error'
        ? 'text-destructive'
        : 'text-muted-foreground'}"
      role={item.status === 'error' ? 'alert' : undefined}
    >
      {item.message}
    </p>
    <div class="flex flex-wrap gap-2">
      <Button size="sm" variant="outline" {disabled} onclick={() => syncTriage.review(projectId)}>
        Review again
      </Button>
      {#if item.status === 'error'}
        <Button
          size="sm"
          variant="ghost"
          {disabled}
          onclick={() => syncTriage.planEverything(projectId)}
        >
          Commit everything…
        </Button>
      {/if}
    </div>
  {/if}
</div>
