/**
 * State and handlers for the AI task generation page (create-from-story, or
 * refine an existing task when `getTaskId()` returns an id). Construct it
 * during component init — it registers onMount.
 */
import { onMount } from 'svelte';
import { goto } from '$app/navigation';
import {
  aiTaskService,
  type GeneratedTaskStructure,
  type ConversationMessage,
  type TaskContext,
} from '../services/aiTaskService';
import { taskActions } from '../stores/taskStore';
import { toastActions } from '$lib/utils/toast';
import { tauriTaskService } from '../services/tauriTaskService';
import { mergeWithOriginal, type OriginalTaskState } from '../utils/mergeGenerated';
import type {
  Task,
  CreateTaskRequest,
  UpdateTaskRequest,
  TaskPriority,
  TaskStatus,
} from '../types';
import type { ChatMessage } from '$lib/domains/ai/types';

export class TaskGenerationState {
  storyText = $state('');
  customInstruction = $state('');
  selectedTemplateId = $state<string | null>(null);
  isGenerating = $state(false);
  generatedData = $state<GeneratedTaskStructure | null>(null);
  chatMessages = $state<ChatMessage[]>([]);
  taskContext = $state<TaskContext | undefined>(undefined);
  existingTask = $state<Task | null>(null);
  isLoadingTask = $state(false);
  createdMainTaskId = $state<string | null>(null);
  originalTaskState = $state<OriginalTaskState | null>(null);
  existingSubtasks = $state<Map<number, Task>>(new Map()); // Map of order/index to existing subtask

  /** Props are reactive, so the id is read through a getter. */
  constructor(private getTaskId: () => string | undefined) {
    // Load existing task if taskId is provided
    onMount(async () => {
      if (this.taskId) {
        this.isLoadingTask = true;
        try {
          this.existingTask = await tauriTaskService.getTask(this.taskId);
          if (this.existingTask) {
            // Pre-fill story text with task description
            this.storyText = this.existingTask.description || '';

            // Preload existing task as preview
            await this.preloadExistingTaskAsPreview();
          }
          // Build context for updating
          this.taskContext = await this.buildTaskContext();
        } catch (error) {
          toastActions.error(
            'Failed to load task',
            error instanceof Error ? error.message : 'Unknown error'
          );
        } finally {
          this.isLoadingTask = false;
        }
      }
    });
  }

  get taskId() {
    return this.getTaskId();
  }

  async preloadExistingTaskAsPreview() {
    if (!this.existingTask || !this.taskId) return;

    try {
      // Get all tasks to find subtasks
      const allTasks = await tauriTaskService.getTasks();
      const subtasks = allTasks.filter((t) => t.parentId === this.taskId);

      // Store existing subtasks mapped by their order/index
      this.existingSubtasks = new Map();
      subtasks.forEach((st, index) => {
        this.existingSubtasks.set(index, st);
      });

      // Convert existing task to GeneratedTaskStructure format
      this.generatedData = {
        main_task: {
          title: this.existingTask.title,
          description: this.existingTask.description || '',
          priority: this.existingTask.priority,
          type_: this.existingTask.type || '',
          estimated_time: this.existingTask.estimatedTime || null,
          tags: this.existingTask.tags || [],
        },
        subtasks: subtasks.map((st, index) => ({
          title: st.title,
          description: st.description || '',
          estimated_time: st.estimatedTime || null,
          dependencies: [],
          order: index + 1,
        })),
        suggested_project: null,
        suggested_labels: this.existingTask.tags || [],
        confidence: 1.0,
        model_used: 'existing-task',
      };

      // Store original state for preservation
      this.originalTaskState = {
        status: this.existingTask.status,
        priority: this.existingTask.priority,
        tags: this.existingTask.tags || [],
        type: this.existingTask.type,
        estimatedTime: this.existingTask.estimatedTime,
      };

      this.chatMessages = [
        {
          role: 'assistant',
          content: `I've loaded your existing task "${this.existingTask.title}" with ${subtasks.length} subtask(s). You can refine it using AI or edit manually.`,
          timestamp: new Date(),
        },
      ];
    } catch (error) {
      console.error('Failed to preload task as preview:', error);
    }
  }

  async buildTaskContext(): Promise<TaskContext | undefined> {
    if (!this.taskId) return undefined;

    const context: TaskContext = {};
    try {
      const task = await tauriTaskService.getTask(this.taskId);
      if (task) {
        context.parentTask = {
          id: task.id,
          title: task.title,
          description: task.description,
          priority: task.priority,
          type: task.type,
          tags: task.tags,
        };

        // Get existing children
        const allTasks = await tauriTaskService.getTasks();
        const children = allTasks
          .filter((t) => t.parentId === this.taskId)
          .map((t) => ({
            id: t.id,
            title: t.title,
            description: t.description,
            status: t.status,
          }));

        if (children.length > 0) {
          context.existingChildren = children;
        }
      }
    } catch (error) {
      console.error('Failed to build task context:', error);
    }

    return Object.keys(context).length > 0 ? context : undefined;
  }

  handleGenerate = async () => {
    if (!this.storyText.trim()) {
      toastActions.error('Please enter story text');
      return;
    }

    this.isGenerating = true;
    try {
      const result = await aiTaskService.generateTasksFromStory({
        story_text: this.storyText,
        provider_type: 'AgentPlatform',
        instruction: this.customInstruction.trim() || undefined,
        context: this.taskContext,
      });

      // If updating an existing task, preserve its original state in the generated data
      this.generatedData = mergeWithOriginal(result, this.taskId ? this.originalTaskState : null);

      this.chatMessages = [
        {
          role: 'assistant',
          content: `I've ${this.taskId ? 'refined' : 'generated'} a task structure with ${result.subtasks.length} subtask(s). You can review and edit the preview, or chat with me to make adjustments.`,
          timestamp: new Date(),
        },
      ];
    } catch (error) {
      toastActions.error(
        'Failed to generate tasks',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    } finally {
      this.isGenerating = false;
    }
  };

  handleChatMessage = async (message: string, history: ChatMessage[]) => {
    if (!this.generatedData) {
      toastActions.error('Please generate tasks first');
      return;
    }

    // Convert chat messages to conversation format
    // Include the original generation result in the history
    const conversationHistory: ConversationMessage[] = [
      {
        role: 'assistant',
        content: JSON.stringify(this.generatedData, null, 2),
      },
      ...history
        .filter(
          (m) => m.role === 'user' || (m.role === 'assistant' && !m.content.includes('generated'))
        )
        .map((m) => ({
          role: m.role,
          content: m.content,
        })),
      {
        role: 'user',
        content: message,
      },
    ];

    try {
      const result = await aiTaskService.generateTasksFromStory({
        story_text: this.storyText,
        provider_type: 'AgentPlatform',
        history: conversationHistory,
        instruction: this.customInstruction.trim() || undefined,
        context: this.taskContext,
      });

      this.generatedData = mergeWithOriginal(result, this.taskId ? this.originalTaskState : null);

      this.chatMessages = [
        ...this.chatMessages,
        {
          role: 'assistant',
          content: `I've updated the task structure based on your feedback. The preview has been refreshed.`,
          timestamp: new Date(),
        },
      ];
    } catch (error) {
      toastActions.error(
        'Failed to update tasks',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    }
  };

  handleAddTask = async (task: any): Promise<void> => {
    try {
      // If updating existing task, use update instead of create
      if (this.taskId && this.existingTask) {
        const updateData: UpdateTaskRequest = {
          title: task.title,
          description: task.description,
          // Preserve original status and other fields unless explicitly changed
          status: (this.originalTaskState?.status || this.existingTask.status) as TaskStatus,
          priority:
            (task.priority as TaskPriority) ||
            this.originalTaskState?.priority ||
            this.existingTask.priority,
          type: task.type_ || this.originalTaskState?.type || this.existingTask.type,
          estimatedTime:
            task.estimated_time ||
            this.originalTaskState?.estimatedTime ||
            this.existingTask.estimatedTime,
          tags:
            this.generatedData && this.generatedData.suggested_labels.length > 0
              ? this.generatedData.suggested_labels
              : this.originalTaskState?.tags || this.existingTask.tags,
        };

        await taskActions.updateTask(this.taskId, updateData);
        toastActions.success(`Task "${task.title}" updated successfully`);
        this.createdMainTaskId = this.taskId;
      } else {
        // Creating new task
        const taskData: CreateTaskRequest = {
          title: task.title,
          description: task.description,
          status: 'pending',
          priority: task.priority as TaskPriority,
          type: task.type_ || undefined,
          estimatedTime: task.estimated_time || undefined,
          tags: this.generatedData?.suggested_labels || undefined,
        };

        const createdTask = await taskActions.createTask(taskData);
        toastActions.success(`Task "${createdTask.title}" created successfully`);
        this.createdMainTaskId = createdTask.id;
      }
    } catch (error) {
      toastActions.error(
        `Failed to ${this.taskId ? 'update' : 'create'} task`,
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
      throw error;
    }
  };

  handleAddSubtask = async (index: number, subtask: any) => {
    if (!this.generatedData) return;

    try {
      // If main task hasn't been created yet, create it first
      let parentTaskId = this.taskId || this.createdMainTaskId;
      if (!parentTaskId && this.generatedData.main_task) {
        await this.handleAddTask(this.generatedData.main_task);
        parentTaskId = this.createdMainTaskId;
      }

      if (!parentTaskId) {
        toastActions.error('Please create the main task first');
        return;
      }

      const subtaskData: CreateTaskRequest = {
        title: subtask.title,
        description: subtask.description || undefined,
        status: 'pending',
        priority: 'medium',
        parentId: parentTaskId,
        estimatedTime: subtask.estimated_time || undefined,
      };

      await taskActions.createTask(subtaskData);
      toastActions.success(`Subtask "${subtask.title}" created successfully`);
    } catch (error) {
      toastActions.error(
        'Failed to create subtask',
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    }
  };

  handleAddAll = async () => {
    if (!this.generatedData) return;

    try {
      // Update or create main task first
      await this.handleAddTask(this.generatedData.main_task);
      const mainTaskId = this.createdMainTaskId || this.taskId;

      if (!mainTaskId) {
        toastActions.error('Failed to create main task');
        return;
      }

      let updatedCount = 0;
      let createdCount = 0;

      // Update existing subtasks or create new ones
      for (let index = 0; index < this.generatedData.subtasks.length; index++) {
        const subtask = this.generatedData.subtasks[index];
        if (!subtask.title.trim()) continue;

        const existingSubtask = this.existingSubtasks.get(index);

        if (existingSubtask && this.taskId) {
          // Update existing subtask
          const updateData: UpdateTaskRequest = {
            title: subtask.title,
            description: subtask.description || undefined,
            status: existingSubtask.status, // Preserve status
            priority: existingSubtask.priority, // Preserve priority
            estimatedTime: subtask.estimated_time || existingSubtask.estimatedTime,
          };
          await taskActions.updateTask(existingSubtask.id, updateData);
          updatedCount++;
        } else {
          // Create new subtask
          const subtaskData: CreateTaskRequest = {
            title: subtask.title,
            description: subtask.description || undefined,
            status: 'pending',
            priority: 'medium',
            parentId: mainTaskId,
            estimatedTime: subtask.estimated_time || undefined,
          };
          await taskActions.createTask(subtaskData);
          createdCount++;
        }
      }

      const action = this.taskId ? 'Updated' : 'Created';
      const message = this.taskId
        ? `Updated task with ${updatedCount} updated and ${createdCount} new subtask(s)`
        : `Created task with ${this.generatedData.subtasks.length} subtask(s)`;

      toastActions.success(message);

      // Navigate to the task
      goto(`/tasks/${mainTaskId}`);
    } catch (error) {
      toastActions.error(
        `Failed to ${this.taskId ? 'update' : 'create'} tasks`,
        error instanceof Error ? error.message : 'An unexpected error occurred'
      );
    }
  };

  handleTemplateSelect = (templateId: string, instruction: string) => {
    this.selectedTemplateId = templateId;
    this.customInstruction = instruction;
  };

  handleUpdateMainTask = (task: any) => {
    if (this.generatedData) {
      this.generatedData = {
        ...this.generatedData,
        main_task: task,
      };
    }
  };

  handleUpdateSubtask = (index: number, subtask: any) => {
    if (this.generatedData) {
      this.generatedData = {
        ...this.generatedData,
        subtasks: this.generatedData.subtasks.map((s, i) => (i === index ? subtask : s)),
      };
    }
  };
}
