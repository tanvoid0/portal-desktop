<script lang="ts">
  import FaClock from '~icons/fa6-solid/clock';
  import FaTrash from '~icons/fa6-solid/trash';
  import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
  } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import type { Document } from '../types';

  interface Props {
    document: Document;
    onClick?: () => void;
    onDelete?: () => void;
  }

  let { document, onClick, onDelete }: Props = $props();

  const preview = $derived(
    document.content.length > 150 ? document.content.substring(0, 150) + '...' : document.content
  );
</script>

<!-- Stretched-link pattern: the title button covers the card; Delete sits above it. -->
<Card
  elevation="raised"
  class="group relative h-full cursor-pointer overflow-hidden transition-shadow focus-within:ring-2 focus-within:ring-ring hover:border-border hover:shadow-sm"
>
  <CardHeader>
    <CardTitle class="transition-colors group-hover:text-primary">
      <Button
        variant="link"
        class="h-auto p-0 text-left font-[inherit] text-[length:inherit] whitespace-normal text-inherit after:absolute after:inset-0 hover:no-underline"
        onclick={() => onClick?.()}
      >
        {document.title}
      </Button>
    </CardTitle>
    <CardDescription>{preview}</CardDescription>
  </CardHeader>
  <CardContent class="space-y-3">
    <div class="flex items-start justify-between gap-2">
      <div class="flex-1">
        <div class="mb-2 flex items-center gap-2">
          {#if document.isDraft}
            <Badge variant="outline" class="text-xs">Draft</Badge>
          {/if}
          {#if document.isArchived}
            <Badge variant="secondary" class="text-xs">Archived</Badge>
          {/if}
        </div>
        <div class="mb-2 flex flex-wrap gap-1">
          {#if document.tags && document.tags.length > 0}
            {#each document.tags.slice(0, 3) as tag}
              <Badge variant="secondary" class="text-xs">{tag}</Badge>
            {/each}
            {#if document.tags.length > 3}
              <Badge variant="outline" class="text-xs">+{document.tags.length - 3}</Badge>
            {/if}
          {/if}
        </div>
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <FaClock class="h-3 w-3" />
          {new Date(document.updatedAt).toLocaleDateString()}
        </div>
      </div>
      {#if onDelete}
        <Button
          variant="ghost"
          size="sm"
          class="relative z-10 h-8 w-8 p-0"
          aria-label={`Delete document: ${document.title}`}
          onclick={() => onDelete?.()}
        >
          <FaTrash class="h-4 w-4" />
        </Button>
      {/if}
    </div>
  </CardContent>
</Card>
