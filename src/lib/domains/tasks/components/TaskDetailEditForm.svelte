<script lang="ts">
  import { Textarea } from '$lib/components/ui/textarea';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import {
    TASK_STATUS_OPTIONS,
    TASK_PRIORITY_OPTIONS,
    type TaskStatus,
    type TaskPriority,
  } from '../types';

  interface Props {
    title: string;
    description: string;
    status: TaskStatus;
    priority: TaskPriority;
    type: string;
    dueDate: string;
  }
  let {
    title = $bindable(),
    description = $bindable(),
    status = $bindable(),
    priority = $bindable(),
    type = $bindable(),
    dueDate = $bindable(),
  }: Props = $props();
</script>

<div class="space-y-4">
  <div>
    <Label for="edit-title">Title</Label>
    <Input id="edit-title" bind:value={title} placeholder="Enter task title..." />
  </div>
  <div>
    <Label for="edit-description">Description</Label>
    <Textarea
      id="edit-description"
      bind:value={description}
      placeholder="Enter task description..."
      rows={4}
    />
  </div>
  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
    <div>
      <Label for="edit-status">Status</Label>
      <Select
        options={TASK_STATUS_OPTIONS}
        defaultValue={status}
        placeholder="Select status..."
        onSelect={(value) => (status = value as TaskStatus)}
      />
    </div>
    <div>
      <Label for="edit-priority">Priority</Label>
      <Select
        options={TASK_PRIORITY_OPTIONS}
        defaultValue={priority}
        placeholder="Select priority..."
        onSelect={(value) => (priority = value as TaskPriority)}
      />
    </div>
  </div>
  <div class="grid grid-cols-1 gap-4 @md:grid-cols-2">
    <div>
      <Label for="edit-type">Type</Label>
      <Input id="edit-type" bind:value={type} placeholder="e.g., Story, Bug, Feature..." />
    </div>
    <div>
      <Label for="edit-due-date">Due Date</Label>
      <Input id="edit-due-date" type="date" bind:value={dueDate} />
    </div>
  </div>
</div>
