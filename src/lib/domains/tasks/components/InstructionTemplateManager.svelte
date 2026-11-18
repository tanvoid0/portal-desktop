<script lang="ts">
  import FaFileLines from '~icons/fa6-solid/file-lines';
  import FaChevronDown from '~icons/fa6-solid/chevron-down';
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible/index.js';
  import Select from '$lib/components/ui/select.svelte';
  import Icon from '@iconify/svelte';
  import { toastActions } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import {
    hasStoredTemplates,
    loadTemplatesFromStorage,
    saveTemplatesToStorage,
    type InstructionTemplate,
  } from '../utils/instructionTemplates';
  import InstructionTemplateForm from './InstructionTemplateForm.svelte';
  import InstructionTemplateList from './InstructionTemplateList.svelte';

  interface Props {
    selectedTemplateId?: string | null;
    customInstruction?: string;
    onTemplateSelect?: (templateId: string, instruction: string) => void;
    onInstructionChange?: (instruction: string) => void;
  }

  let {
    selectedTemplateId = $bindable<string | null>(null),
    customInstruction = $bindable(''),
    onTemplateSelect,
    onInstructionChange,
  }: Props = $props();

  let templates = $state<InstructionTemplate[]>(loadTemplatesFromStorage());
  let showTemplateManager = $state(false);
  let showNewTemplateForm = $state(false);
  let newTemplateName = $state('');
  let newTemplateInstruction = $state('');
  let editingTemplateId = $state<string | null>(null);

  onMount(() => {
    // Save defaults if they don't exist in localStorage
    if (!hasStoredTemplates()) {
      saveTemplates();
    }
  });

  function saveTemplates() {
    saveTemplatesToStorage(templates);
  }

  function handleTemplateSelect(templateId: string) {
    const template = templates.find((t) => t.id === templateId);
    if (template) {
      selectedTemplateId = templateId;
      customInstruction = template.instruction;
      onTemplateSelect?.(templateId, template.instruction);
    }
  }

  function handleCreateTemplate() {
    if (!newTemplateName.trim() || !newTemplateInstruction.trim()) {
      toastActions.error('Please fill in both name and instruction');
      return;
    }

    const newTemplate: InstructionTemplate = {
      id: `template-${Date.now()}`,
      name: newTemplateName.trim(),
      instruction: newTemplateInstruction.trim(),
      createdAt: new Date(),
    };

    templates = [...templates, newTemplate];
    saveTemplates();
    toastActions.success('Template created successfully');

    // Select the new template
    handleTemplateSelect(newTemplate.id);

    // Reset form
    newTemplateName = '';
    newTemplateInstruction = '';
    showNewTemplateForm = false;
  }

  async function handleDeleteTemplate(templateId: string) {
    const template = templates.find((t) => t.id === templateId);
    if (template?.isDefault) {
      toastActions.error('Cannot delete default templates');
      return;
    }

    const confirmed = await confirmAction(
      'Are you sure you want to delete this template?',
      'Delete template'
    );
    if (!confirmed) return;

    templates = templates.filter((t) => t.id !== templateId);
    saveTemplates();
    if (selectedTemplateId === templateId) {
      selectedTemplateId = null;
      customInstruction = '';
    }
    toastActions.success('Template deleted');
  }

  function handleEditTemplate(templateId: string) {
    const template = templates.find((t) => t.id === templateId);
    if (template) {
      editingTemplateId = templateId;
      newTemplateName = template.name;
      newTemplateInstruction = template.instruction;
      showNewTemplateForm = true;
    }
  }

  function handleUpdateTemplate() {
    if (!editingTemplateId || !newTemplateName.trim() || !newTemplateInstruction.trim()) {
      return;
    }

    const template = templates.find((t) => t.id === editingTemplateId);
    if (template?.isDefault) {
      toastActions.error('Cannot edit default templates');
      return;
    }

    templates = templates.map((t) =>
      t.id === editingTemplateId
        ? {
            ...t,
            name: newTemplateName.trim(),
            instruction: newTemplateInstruction.trim(),
          }
        : t
    );
    saveTemplates();
    toastActions.success('Template updated');

    // Update selected instruction if this template is selected
    if (selectedTemplateId === editingTemplateId) {
      customInstruction = newTemplateInstruction.trim();
      onInstructionChange?.(customInstruction);
    }

    // Reset form
    editingTemplateId = null;
    newTemplateName = '';
    newTemplateInstruction = '';
    showNewTemplateForm = false;
  }

  function handleCancelForm() {
    editingTemplateId = null;
    newTemplateName = '';
    newTemplateInstruction = '';
    showNewTemplateForm = false;
  }

  const templateOptions = $derived(
    templates.map((t) => ({
      value: t.id,
      label: t.name + (t.isDefault ? ' (Default)' : ''),
    }))
  );

  const selectOptions = $derived([{ value: '', label: 'None (Custom)' }, ...templateOptions]);
</script>

<Collapsible bind:open={showTemplateManager}>
  <div class="space-y-2">
    <CollapsibleTrigger>
      <Button variant="outline" class="w-full justify-between">
        <span class="flex items-center gap-2">
          <FaFileLines class="h-4 w-4" />
          Custom Instructions
        </span>
        <FaChevronDown class="h-4 w-4" />
      </Button>
    </CollapsibleTrigger>
    <CollapsibleContent>
      <div class="space-y-4 pt-2">
        <!-- Template Selection -->
        <div class="space-y-2">
          <Label>Instruction Template</Label>
          <Select
            options={selectOptions}
            defaultValue={selectedTemplateId || ''}
            onSelect={(value) => {
              if (value) {
                handleTemplateSelect(value);
              } else {
                selectedTemplateId = null;
                customInstruction = '';
              }
            }}
          />
        </div>

        <!-- Custom Instruction -->
        <div class="space-y-2">
          <Label for="custom-instruction">Custom Instruction</Label>
          <Textarea
            id="custom-instruction"
            bind:value={customInstruction}
            placeholder="e.g., Focus on API endpoints only, break into smaller subtasks, prioritize security features..."
            rows={3}
            oninput={() => {
              selectedTemplateId = null;
            }}
          />
          <p class="text-xs text-muted-foreground">
            Specify how you want the tasks to be generated (format, focus areas, structure, etc.).
          </p>
        </div>

        <!-- Template Management -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <Label>Manage Templates</Label>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => {
                showNewTemplateForm = !showNewTemplateForm;
                if (!showNewTemplateForm) {
                  handleCancelForm();
                }
              }}
            >
              <Icon
                icon={showNewTemplateForm ? 'lucide:minus' : 'lucide:plus'}
                class="mr-1 h-4 w-4"
              />
              {showNewTemplateForm ? 'Cancel' : 'New Template'}
            </Button>
          </div>

          {#if showNewTemplateForm}
            <InstructionTemplateForm
              bind:name={newTemplateName}
              bind:instruction={newTemplateInstruction}
              editing={!!editingTemplateId}
              onSubmit={editingTemplateId ? handleUpdateTemplate : handleCreateTemplate}
              onCancel={handleCancelForm}
            />
          {/if}

          <InstructionTemplateList
            {templates}
            onEdit={handleEditTemplate}
            onDelete={handleDeleteTemplate}
          />
        </div>
      </div>
    </CollapsibleContent>
  </div>
</Collapsible>
