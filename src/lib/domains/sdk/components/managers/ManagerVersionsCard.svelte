<!-- Card shell for a manager's version list: title, Refresh, spinner / empty / error / rows. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import RefreshCw from '~icons/fa6-solid/rotate';

  interface Props {
    title: string;
    loading: boolean;
    error?: string | null;
    empty: boolean;
    emptyText: string;
    onrefresh: () => void;
    children: Snippet;
  }
  let { title, loading, error = null, empty, emptyText, onrefresh, children }: Props = $props();
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between">
      <CardTitle>{title}</CardTitle>
      <Button variant="outline" size="sm" onclick={onrefresh} disabled={loading}>
        <RefreshCw class="mr-2 h-4 w-4 {loading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>
  </CardHeader>
  <CardContent>
    {#if loading}
      <div class="flex items-center justify-center py-8">
        <RefreshCw class="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    {:else if error}
      <p class="py-8 text-center text-destructive">{error}</p>
    {:else if empty}
      <p class="py-8 text-center text-muted-foreground">{emptyText}</p>
    {:else}
      {@render children()}
    {/if}
  </CardContent>
</Card>
