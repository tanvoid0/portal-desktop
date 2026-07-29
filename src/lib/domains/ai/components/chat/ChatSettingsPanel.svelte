<script lang="ts">
  import type { Snippet } from 'svelte';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Slider } from '$lib/components/ui/slider';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Button } from '$lib/components/ui/button';
  import { ChevronDown, RotateCcw, X } from '@lucide/svelte';
  import {
    DEFAULT_CHAT_SETTINGS,
    saveChatSettings,
    type ChatSettings,
  } from '../../utils/chatSettings.js';

  interface Props {
    settings: ChatSettings;
    /** Rough token count of the system prompt, shown like LM Studio. */
    systemPromptTokens?: number;
  }

  let { settings = $bindable(), systemPromptTokens }: Props = $props();

  let stopDraft = $state('');

  // ~4 chars per token is close enough for a composer-side hint.
  const promptTokens = $derived(systemPromptTokens ?? Math.ceil(settings.systemPrompt.length / 4));

  $effect(() => {
    saveChatSettings($state.snapshot(settings));
  });

  function addStop(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    const value = stopDraft.trim();
    if (!value || settings.stopStrings.includes(value)) return;
    settings.stopStrings = [...settings.stopStrings, value];
    stopDraft = '';
  }
</script>

{#snippet numberRow(
  label: string,
  value: number,
  set: (v: number) => void,
  min: number,
  max: number,
  step: number,
  hint?: string
)}
  <div class="space-y-1.5">
    <div class="flex items-center justify-between gap-2">
      <span class="text-xs">{label}</span>
      <Input
        type="number"
        {min}
        {max}
        {step}
        {value}
        oninput={(e) => {
          const next = Number((e.currentTarget as HTMLInputElement).value);
          if (!Number.isNaN(next)) set(next);
        }}
        class="h-7 w-20 px-2 text-right text-xs tabular-nums"
      />
    </div>
    <Slider type="single" {value} onValueChange={set} {min} {max} {step} />
    {#if hint}
      <p class="text-[10px] leading-snug text-muted-foreground">{hint}</p>
    {/if}
  </div>
{/snippet}

<!-- A sampling knob that is off until its checkbox is ticked (LM Studio style). -->
{#snippet gatedRow(
  label: string,
  value: number | null,
  set: (v: number | null) => void,
  fallback: number,
  min: number,
  max: number,
  step: number
)}
  <div class="space-y-1.5">
    <div class="flex items-center justify-between gap-2">
      <label class="flex items-center gap-2 text-xs">
        <Checkbox
          checked={value != null}
          onCheckedChange={(on) => set(on ? fallback : null)}
          class="h-3.5 w-3.5"
        />
        {label}
      </label>
      <Input
        type="number"
        {min}
        {max}
        {step}
        disabled={value == null}
        value={value ?? fallback}
        oninput={(e) => {
          const next = Number((e.currentTarget as HTMLInputElement).value);
          if (!Number.isNaN(next)) set(next);
        }}
        class="h-7 w-20 px-2 text-right text-xs tabular-nums"
      />
    </div>
    {#if value != null}
      <Slider type="single" {value} onValueChange={set} {min} {max} {step} />
    {/if}
  </div>
{/snippet}

{#snippet section(title: string, body: Snippet)}
  <details class="group/sec border-b border-border/60 last:border-0" open>
    <summary
      class="flex cursor-pointer list-none items-center justify-between py-2.5 text-xs font-medium text-muted-foreground marker:content-none hover:text-foreground"
    >
      {title}
      <ChevronDown class="h-3.5 w-3.5 transition-transform group-open/sec:rotate-180" />
    </summary>
    <div class="space-y-4 pb-4">
      {@render body()}
    </div>
  </details>
{/snippet}

<ScrollArea class="min-h-0 flex-1">
  <div class="px-3 pb-4">
    {#snippet systemPromptBody()}
      <div class="space-y-1.5">
        <Textarea
          id="chat-system-prompt"
          bind:value={settings.systemPrompt}
          rows={5}
          placeholder={'Example, "Only answer in rhymes"'}
          class="resize-y text-xs"
        />
        <p class="text-right text-[10px] text-muted-foreground">
          Token count: {promptTokens}
        </p>
      </div>
    {/snippet}
    {@render section('System Prompt', systemPromptBody)}

    {#snippet settingsBody()}
      {@render numberRow(
        'Temperature',
        settings.temperature,
        (v) => (settings.temperature = v),
        0,
        2,
        0.05
      )}

      <div class="space-y-1.5">
        <div class="flex items-center justify-between gap-2">
          <label class="flex items-center gap-2 text-xs">
            <Checkbox bind:checked={settings.limitResponseLength} class="h-3.5 w-3.5" />
            Limit Response Length
          </label>
          <Input
            type="number"
            min={64}
            max={131072}
            step={64}
            disabled={!settings.limitResponseLength}
            bind:value={settings.maxTokens}
            class="h-7 w-20 px-2 text-right text-xs tabular-nums"
          />
        </div>
      </div>

      <div class="space-y-1.5">
        <Label class="text-xs">Stop Strings</Label>
        <Input
          bind:value={stopDraft}
          onkeydown={addStop}
          placeholder="Enter a string and press ⏎"
          class="h-7 text-xs"
        />
        {#if settings.stopStrings.length}
          <div class="flex flex-wrap gap-1">
            {#each settings.stopStrings as stop}
              <span
                class="inline-flex items-center gap-1 rounded-md bg-muted px-1.5 py-0.5 text-[11px]"
              >
                <code class="font-mono">{stop}</code>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  class="h-4 w-4 text-muted-foreground hover:text-destructive"
                  aria-label="Remove stop string {stop}"
                  onclick={() =>
                    (settings.stopStrings = settings.stopStrings.filter((s) => s !== stop))}
                >
                  <X class="h-3 w-3" />
                </Button>
              </span>
            {/each}
          </div>
        {/if}
      </div>
    {/snippet}
    {@render section('Settings', settingsBody)}

    {#snippet samplingBody()}
      {@render gatedRow('Top K Sampling', settings.topK, (v) => (settings.topK = v), 40, 1, 200, 1)}
      {@render gatedRow(
        'Repeat Penalty',
        settings.repeatPenalty,
        (v) => (settings.repeatPenalty = v),
        1.1,
        1,
        2,
        0.01
      )}
      {@render gatedRow(
        'Presence Penalty',
        settings.presencePenalty,
        (v) => (settings.presencePenalty = v),
        0,
        -2,
        2,
        0.05
      )}
      {@render gatedRow(
        'Top P Sampling',
        settings.topP,
        (v) => (settings.topP = v),
        0.95,
        0,
        1,
        0.01
      )}
      {@render gatedRow(
        'Min P Sampling',
        settings.minP,
        (v) => (settings.minP = v),
        0.05,
        0,
        1,
        0.01
      )}
    {/snippet}
    {@render section('Sampling', samplingBody)}

    <Button
      variant="outline"
      size="sm"
      class="mt-4 w-full gap-1.5 text-xs"
      onclick={() => (settings = { ...DEFAULT_CHAT_SETTINGS })}
    >
      <RotateCcw class="h-3 w-3" />
      Reset to defaults
    </Button>
  </div>
</ScrollArea>
