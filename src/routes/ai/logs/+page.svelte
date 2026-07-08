<script lang="ts">
  import { PageHeader } from "$lib/components/shell";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { ScrollText } from "@lucide/svelte";
</script>

<div class="space-y-6">
  <PageHeader
    title="AI Logs"
    description="View AI request and response logs from the desktop app."
  />

  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <ScrollText class="h-5 w-5" />
        Logs
      </CardTitle>
      <CardDescription>
        This page was reset after a file encoding issue. Rebuild log UI blocks here if needed.
      </CardDescription>
    </CardHeader>
    <CardContent>
      <p class="text-sm text-muted-foreground">
        Log features are temporarily unavailable in this view, but the route now builds correctly.
      </p>
    </CardContent>
  </Card>
</div>
