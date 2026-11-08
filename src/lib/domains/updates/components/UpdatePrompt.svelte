<!--
	Launch-time "update available" dialog. Mounted once in the app shell.
-->

<script lang="ts">
  import {
    AlertDialog,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Progress } from '$lib/components/ui/progress';
  import { updatePrompt } from '../state/updatePrompt.svelte';
  import { installFromPrompt } from '../services/startupUpdate';
</script>

<AlertDialog
  open={updatePrompt.open}
  onOpenChange={(isOpen) => {
    if (!isOpen) updatePrompt.dismiss();
  }}
>
  {#if updatePrompt.info}
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Update available — v{updatePrompt.info.version}</AlertDialogTitle>
        <AlertDialogDescription>
          A new version of Portal Desktop is ready. Installing restarts the app.
        </AlertDialogDescription>
      </AlertDialogHeader>

      {#if updatePrompt.info.body}
        <div class="max-h-40 overflow-y-auto text-sm whitespace-pre-wrap text-muted-foreground">
          {updatePrompt.info.body}
        </div>
      {/if}

      {#if updatePrompt.error}
        <p class="text-sm text-destructive">
          {updatePrompt.error.message}
          {#if updatePrompt.error.hint}{updatePrompt.error.hint}{/if}
        </p>
      {/if}

      {#if updatePrompt.installing}
        <Progress value={undefined} class="w-full" />
      {/if}

      <AlertDialogFooter>
        <Button
          variant="outline"
          disabled={updatePrompt.installing}
          onclick={() => updatePrompt.dismiss()}
        >
          Later
        </Button>
        <Button disabled={updatePrompt.installing} onclick={installFromPrompt}>
          {updatePrompt.installing ? 'Installing…' : 'Install & restart'}
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  {/if}
</AlertDialog>
