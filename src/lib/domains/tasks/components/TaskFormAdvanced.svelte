<script lang="ts">
  import FaGear from '~icons/fa6-solid/gear';
  import FaChevronDown from '~icons/fa6-solid/chevron-down';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Separator } from '$lib/components/ui/separator';
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible';
  import Select from '$lib/components/ui/select.svelte';
  import { RecurringPatternEnum } from '../types';
  import TaskFormField from './TaskFormField.svelte';

  interface Props {
    recurringPattern: string;
    recurringInterval: number;
    recurringEndDate: string;
    blockedBy: string[];
    blocks: string[];
  }
  let {
    recurringPattern = $bindable(),
    recurringInterval = $bindable(),
    recurringEndDate = $bindable(),
    blockedBy = $bindable(),
    blocks = $bindable(),
  }: Props = $props();

  const RECURRING_PATTERNS = Object.values(RecurringPatternEnum);
  let open = $state(false);

  const parseIds = (value: string) =>
    value
      .split(',')
      .map((id) => id.trim())
      .filter((id) => id);
</script>

<Collapsible bind:open>
  <CollapsibleTrigger>
    <Button variant="outline" class="w-full">
      <FaGear class="mr-2 h-4 w-4" />
      Advanced Settings
      <FaChevronDown class="ml-auto h-4 w-4" />
    </Button>
  </CollapsibleTrigger>
  <CollapsibleContent class="mt-4 space-y-4">
    <Separator />

    <div class="space-y-4">
      <h4 class="text-sm font-medium">Recurring Task</h4>
      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <TaskFormField label="Pattern" for="recurringPattern">
          <Select
            options={['', ...RECURRING_PATTERNS]}
            defaultValue={recurringPattern}
            placeholder="Select pattern..."
            onSelect={(value) => (recurringPattern = value)}
          />
        </TaskFormField>
        <TaskFormField label="Interval" for="recurringInterval">
          <Input
            id="recurringInterval"
            type="number"
            min="1"
            bind:value={recurringInterval}
            placeholder="1"
          />
        </TaskFormField>
        <TaskFormField label="End Date" for="recurringEndDate">
          <Input id="recurringEndDate" type="date" bind:value={recurringEndDate} />
        </TaskFormField>
      </div>
    </div>

    <div class="space-y-4">
      <h4 class="text-sm font-medium">Dependencies</h4>
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <TaskFormField label="Blocked By (Task IDs)">
          <Textarea
            placeholder="Enter task IDs separated by commas..."
            rows={3}
            value={blockedBy.join(', ')}
            oninput={(e) => (blockedBy = parseIds(e.currentTarget.value))}
          />
        </TaskFormField>
        <TaskFormField label="Blocks (Task IDs)">
          <Textarea
            placeholder="Enter task IDs separated by commas..."
            rows={3}
            value={blocks.join(', ')}
            oninput={(e) => (blocks = parseIds(e.currentTarget.value))}
          />
        </TaskFormField>
      </div>
    </div>
  </CollapsibleContent>
</Collapsible>
