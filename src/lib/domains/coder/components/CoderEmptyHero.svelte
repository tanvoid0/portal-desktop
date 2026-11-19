<!-- "What should we build?" heading shown above the composer in an empty session. -->
<script lang="ts">
  import Bot from '~icons/fa6-solid/robot';

  interface Props {
    title?: string;
    description?: string;
  }

  let {
    title = 'What should we build?',
    description = 'Describe a task. The agent explores, edits, and runs commands — mode controls how it operates.',
  }: Props = $props();
</script>

<div class="mb-6 text-center [@media(max-height:700px)]:mb-2">
  <div
    class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 [@media(max-height:700px)]:hidden"
  >
    <Bot class="h-6 w-6 text-primary" />
  </div>
  <h3 class="text-lg font-semibold">{title}</h3>
  <p class="mt-1 max-w-md text-sm text-muted-foreground [@media(max-height:700px)]:hidden">
    {description}
  </p>
</div>
