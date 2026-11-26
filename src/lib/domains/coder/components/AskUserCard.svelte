<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as RadioGroup from '$lib/components/ui/radio-group';
  import { toast } from '$lib/utils/toast';
  import { coderService } from '../services/coderService.js';
  import type { ToolCall } from '../types.js';
  import { formatAnswers, parseQuestions } from '../utils/askUser.js';

  interface Props {
    call: ToolCall;
    result?: string | null;
    threadId?: string;
    /** The run is still waiting on this call. */
    active?: boolean;
  }

  let { call, result = null, threadId, active = false }: Props = $props();

  const questions = $derived(parseQuestions(call.function.arguments));

  let step = $state(0);
  let picked = $state<string[][]>([]);
  let other = $state<string[]>([]);
  let sent = $state<string | null>(null);

  const q = $derived(questions[step]);
  const settled = $derived(result ?? sent);
  const disabled = $derived(!active || !threadId);
  /** One single-choice question: the option itself is the submit button. */
  const oneClick = $derived(questions.length === 1 && !questions[0]?.multiSelect);

  function answerFor(i: number): string[] {
    const typed = other[i]?.trim();
    const chosen = picked[i] ?? [];
    // Typed text replaces a radio pick, and adds to checkbox picks.
    if (!typed) return chosen;
    return questions[i].multiSelect ? [...chosen, typed] : [typed];
  }

  function toggle(option: string, on: boolean) {
    const chosen = picked[step] ?? [];
    picked[step] = on ? [...chosen, option] : chosen.filter((o) => o !== option);
  }

  async function submit(answers: string[][]) {
    if (!threadId) return;
    const text = formatAnswers(questions, answers);
    sent = text;
    try {
      await coderService.submitCommandResult(threadId, call.id, text);
    } catch (e) {
      sent = null;
      toast.error(`Could not send the answer: ${e}`);
    }
  }

  function next() {
    if (step < questions.length - 1) step += 1;
    else void submit(questions.map((_, i) => answerFor(i)));
  }
</script>

<div class="rounded-lg border border-primary/30 bg-primary/5 p-3 text-sm">
  {#if settled != null}
    <div class="whitespace-pre-wrap text-muted-foreground">{settled}</div>
  {:else if !q}
    <div class="text-muted-foreground">The model asked a question with no readable content.</div>
  {:else}
    {#if questions.length > 1}
      <div class="mb-1 text-xs text-muted-foreground">Step {step + 1} of {questions.length}</div>
    {/if}
    <div class="mb-3 font-medium">{q.question}</div>

    {#if oneClick}
      <div class="mb-3 flex flex-wrap gap-2">
        {#each q.options as option, i (option)}
          <Button
            size="sm"
            variant={i === 0 ? 'default' : 'secondary'}
            class="h-auto min-h-8 text-left whitespace-normal"
            {disabled}
            onclick={() => submit([[option]])}
          >
            {option}
          </Button>
        {/each}
      </div>
    {:else if q.multiSelect}
      <div class="mb-3 space-y-2">
        {#each q.options as option, i (option)}
          <div class="flex items-center gap-2">
            <Checkbox
              id="{call.id}-{step}-{i}"
              checked={picked[step]?.includes(option) ?? false}
              onCheckedChange={(on) => toggle(option, on === true)}
              {disabled}
            />
            <Label for="{call.id}-{step}-{i}" class="font-normal">{option}</Label>
          </div>
        {/each}
      </div>
    {:else}
      {#key step}
        <RadioGroup.Root
          class="mb-3"
          value={picked[step]?.[0] ?? ''}
          onValueChange={(v) => (picked[step] = [v])}
          {disabled}
        >
          {#each q.options as option, i (option)}
            <div class="flex items-center gap-2">
              <RadioGroup.Item id="{call.id}-{step}-{i}" value={option} />
              <Label for="{call.id}-{step}-{i}" class="font-normal">{option}</Label>
            </div>
          {/each}
        </RadioGroup.Root>
      {/key}
    {/if}

    <form
      class="flex flex-wrap items-center gap-2"
      onsubmit={(e) => {
        e.preventDefault();
        if (answerFor(step).length) next();
      }}
    >
      <Input
        bind:value={() => other[step] ?? '', (v) => (other[step] = v)}
        class="h-8 min-w-40 flex-1"
        placeholder="Or type your own answer"
        aria-label="Your own answer"
        {disabled}
      />
      {#if step > 0}
        <Button type="button" size="sm" variant="ghost" onclick={() => (step -= 1)}>Back</Button>
      {/if}
      <Button type="submit" size="sm" disabled={disabled || !answerFor(step).length}>
        {step < questions.length - 1 ? 'Next' : 'Send'}
      </Button>
    </form>
    {#if !active}
      <div class="mt-2 text-xs text-muted-foreground">This run ended before it was answered.</div>
    {/if}
  {/if}
</div>
