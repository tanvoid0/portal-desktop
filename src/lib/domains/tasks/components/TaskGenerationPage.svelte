<script lang="ts">
  import FaArrowLeft from '~icons/fa6-solid/arrow-left';

  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { TaskGenerationState } from '../state/taskGeneration.svelte';
  import TaskStoryInputCard from './TaskStoryInputCard.svelte';
  import TaskGenerationPreviewPanel from './TaskGenerationPreviewPanel.svelte';
  import AIChatPanel from '$lib/domains/ai/components/chat/AIChatPanel.svelte';
  import LoadingSpinner from '$lib/components/ui/loading-spinner.svelte';

  interface Props {
    taskId?: string; // For updating existing task
  }

  let { taskId }: Props = $props();

  const g = new TaskGenerationState(() => taskId);
</script>

<div class="min-h-screen bg-background">
  <div>
    <!-- Header -->
    <div class="mb-6">
      <div class="mb-4 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold">
            {taskId ? 'Update Task with AI' : 'Generate Tasks with AI'}
          </h1>
          <p class="text-sm text-muted-foreground">
            {taskId
              ? 'Use AI to generate subtasks or update the current task'
              : 'Describe your work and let AI create structured tasks for you'}
          </p>
        </div>
        <Button variant="ghost" onclick={() => goto(taskId ? `/tasks/${taskId}` : '/tasks')}>
          <FaArrowLeft class="mr-2 h-4 w-4" />
          Back
        </Button>
      </div>
    </div>

    {#if g.isLoadingTask}
      <div class="flex items-center justify-center py-12">
        <LoadingSpinner size="lg" text="Loading task..." />
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <!-- Left Column: Input and Preview -->
        <div class="space-y-6 lg:col-span-2">
          <!-- Story Input -->
          <TaskStoryInputCard
            bind:storyText={g.storyText}
            bind:selectedTemplateId={g.selectedTemplateId}
            bind:customInstruction={g.customInstruction}
            isGenerating={g.isGenerating}
            onTemplateSelect={g.handleTemplateSelect}
            onGenerate={g.handleGenerate}
          />

          <!-- Preview -->
          {#if g.generatedData}
            <TaskGenerationPreviewPanel
              bind:data={g.generatedData}
              originalStatus={g.originalTaskState?.status}
              isUpdateMode={!!taskId}
              onUpdateMain={g.handleUpdateMainTask}
              onUpdateSubtask={g.handleUpdateSubtask}
              onAdd={g.handleAddTask}
              onAddSubtask={g.handleAddSubtask}
              onAddAll={g.handleAddAll}
            />
          {/if}
        </div>

        <!-- Right Column: Chat Panel -->
        <div class="lg:col-span-1">
          <AIChatPanel
            bind:messages={g.chatMessages}
            onSendMessageWithHistory={g.handleChatMessage}
            bind:isLoading={g.isGenerating}
            placeholder="Ask me to adjust the tasks, add more subtasks, change priorities..."
            title="Refine Tasks"
            class="h-[calc(100vh-12rem)]"
          />
        </div>
      </div>
    {/if}
  </div>
</div>
