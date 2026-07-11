<!--
	Global confirmation dialog backed by shadcn AlertDialog.
-->

<script lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from "$lib/components/ui/alert-dialog";
  import { buttonVariants } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import {
    confirmActions,
    confirmStore,
  } from "$lib/domains/shared/stores/confirmStore";

  let request = $derived($confirmStore);
  let open = $state(false);

  $effect(() => {
    open = request !== null;
  });

  function handleOpenChange(isOpen: boolean) {
    if (!isOpen) {
      confirmActions.cancel();
    }
  }
</script>

<AlertDialog {open} onOpenChange={handleOpenChange}>
  {#if request}
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>{request.title}</AlertDialogTitle>
        <AlertDialogDescription>{request.message}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>{request.cancelLabel}</AlertDialogCancel>
        <AlertDialogAction
          class={request.destructive
            ? cn(buttonVariants({ variant: "destructive" }))
            : undefined}
          onclick={() => confirmActions.confirm()}
        >
          {request.confirmLabel}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  {/if}
</AlertDialog>
