<script lang="ts">
  import FaWandMagicSparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import InstructionTemplateManager from './InstructionTemplateManager.svelte';

  interface Props {
    storyText: string;
    selectedTemplateId: string | null;
    customInstruction: string;
    isGenerating: boolean;
    onTemplateSelect: (templateId: string, instruction: string) => void;
    onGenerate: () => void;
  }
  let {
    storyText = $bindable(),
    selectedTemplateId = $bindable(),
    customInstruction = $bindable(),
    isGenerating,
    onTemplateSelect,
    onGenerate,
  }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle>Story / Description</CardTitle>
    <CardDescription>Paste your story, issue description, or requirements here</CardDescription>
  </CardHeader>
  <CardContent class="space-y-4">
    <div class="space-y-2">
      <Label for="story-text">Story Text</Label>
      <Textarea
        id="story-text"
        bind:value={storyText}
        placeholder="Paste your story, issue description, or requirements here..."
        rows={8}
        disabled={isGenerating}
      />
    </div>

    <InstructionTemplateManager bind:selectedTemplateId bind:customInstruction {onTemplateSelect} />

    <Button onclick={onGenerate} disabled={!storyText.trim() || isGenerating} class="w-full">
      <FaWandMagicSparkles class="mr-2 h-4 w-4" />
      Generate Tasks
    </Button>
  </CardContent>
</Card>
