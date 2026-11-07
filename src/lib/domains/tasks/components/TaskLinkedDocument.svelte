<script lang="ts">
  import FaFileLines from '~icons/fa6-solid/file-lines';
  import FaArrowUpRightFromSquare from '~icons/fa6-solid/arrow-up-right-from-square';
  import FaPenToSquare from '~icons/fa6-solid/pen-to-square';
  import FaPlus from '~icons/fa6-solid/plus';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { documentActions, documents } from '$lib/domains/documents';
  import { ResourceType } from '$lib/domains/shared/types/resourceType';
  import type { Task } from '../types';

  interface Props {
    task: Task;
  }

  let { task }: Props = $props();

  onMount(() => {
    documentActions.loadDocuments();
  });

  const linkedDoc = $derived(
    task.resourceType === ResourceType.DOCUMENT && task.resourceId
      ? ($documents.find((d) => d.id === parseInt(task.resourceId!)) ?? null)
      : null
  );
  /** Only a document-linked task whose document is missing hides the card entirely. */
  const showCreate = $derived(task.resourceType !== ResourceType.DOCUMENT || !task.resourceId);
</script>

{#if linkedDoc}
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2 text-lg">
        <FaFileLines class="h-5 w-5" />
        Linked Document
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      <div>
        <h4 class="font-medium">{linkedDoc.title}</h4>
        <p class="mt-1 line-clamp-2 text-sm text-muted-foreground">
          {linkedDoc.content.substring(0, 100)}
          {linkedDoc.content.length > 100 ? '...' : ''}
        </p>
      </div>
      <div class="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          onclick={() => goto(`/documents/${linkedDoc.id}`)}
          class="flex-1"
        >
          <FaArrowUpRightFromSquare class="mr-2 h-4 w-4" />
          Open Document
        </Button>
        <Button variant="outline" size="sm" onclick={() => goto(`/documents/${linkedDoc.id}/edit`)}>
          <FaPenToSquare class="h-4 w-4" />
        </Button>
      </div>
    </CardContent>
  </Card>
{:else if showCreate}
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2 text-lg">
        <FaFileLines class="h-5 w-5" />
        Document
      </CardTitle>
    </CardHeader>
    <CardContent>
      <Button
        variant="outline"
        onclick={() =>
          goto(`/documents/create?taskId=${task.id}&taskTitle=${encodeURIComponent(task.title)}`)}
        class="w-full"
      >
        <FaPlus class="mr-2 h-4 w-4" />
        Create Document for Task
      </Button>
    </CardContent>
  </Card>
{/if}
