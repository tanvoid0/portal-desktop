<!-- Ollama page header: back link, title, running/port badges. -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';

  let { serviceInfo }: { serviceInfo: any } = $props();
</script>

<div
  class="divider-edge-b divider-edge-full bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
>
  <div class="flex h-14 w-full items-center px-4">
    <div class="mr-4 hidden md:flex">
      <a class="mr-6 flex items-center space-x-2" href="/sdk">
        <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M10 19l-7-7m0 0l7-7m-7 7h18"
          />
        </svg>
        <span class="hidden font-bold lg:inline-block">Back to SDK</span>
      </a>
    </div>
    <div class="flex flex-1 items-center justify-between space-x-2 md:justify-end">
      <div class="w-full flex-1 md:w-auto md:flex-none">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-3">
            <div
              class="flex size-8 items-center justify-center rounded-md bg-primary/10 text-sm font-semibold text-primary"
            >
              <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
                <path
                  d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.94-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"
                />
              </svg>
            </div>
            <div>
              <h1 class="text-xl font-semibold">Ollama</h1>
              <p class="text-sm text-muted-foreground">Local AI model runner</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            {#if serviceInfo?.running}
              <Badge variant="default" class="bg-green-100 text-green-800">
                <svg class="mr-1 h-3 w-3" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M5 13l4 4L19 7" />
                </svg>
                Running
              </Badge>
            {:else}
              <Badge variant="outline" class="text-gray-500">
                {serviceInfo?.status || 'Stopped'}
              </Badge>
            {/if}
            {#if serviceInfo?.port}
              <Badge variant="secondary">Port {serviceInfo.port}</Badge>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
