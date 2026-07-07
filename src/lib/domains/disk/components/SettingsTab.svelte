<script lang="ts">
  import { AI_PROVIDER_SETTINGS_PATH } from "$lib/config/ai-nav";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
</script>

<Card>
  <CardContent class="max-w-xl space-y-3 p-4">
    <p class="text-sm text-muted-foreground">
      AI verification uses the desktop app's configured provider — no separate
      disk-utility credentials.
    </p>
    <p class="text-sm text-muted-foreground">
      Set base URL, API token, and model under
      <Button variant="link" href={AI_PROVIDER_SETTINGS_PATH} class="h-auto p-0">
        Settings → AI Providers
      </Button>.
      Backend providers and models are configured on agent-platform
      (<code class="text-muted-foreground">/config</code>), not here.
    </p>
  </CardContent>
</Card>
