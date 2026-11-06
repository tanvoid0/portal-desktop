<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { PageError } from '$lib/components/shell';

  const is404 = $derived(page.status === 404);
</script>

<div class="mx-auto max-w-2xl space-y-4 p-6">
  <PageError
    title={`${page.status} ${is404 ? 'Page not found' : 'Something went wrong'}`}
    message={is404
      ? `There is nothing at ${page.url.pathname}.`
      : (page.error?.message ?? 'An unexpected error occurred.')}
  />
  <div class="flex justify-center gap-2">
    <Button variant="outline" onclick={() => (history.length > 1 ? history.back() : goto('/'))}
      >Go back</Button
    >
    <Button onclick={() => goto('/')}>Go home</Button>
  </div>
</div>
