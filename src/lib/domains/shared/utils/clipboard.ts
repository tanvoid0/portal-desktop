import { toast } from '$lib/utils/toast';

/**
 * Copy text to the clipboard and toast the outcome. Shared by the "Copy
 * reference" context-menu actions; returns whether it succeeded.
 */
export async function copyText(text: string, okMsg = 'Copied to clipboard'): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    toast.success(okMsg);
    return true;
  } catch {
    toast.error('Could not access the clipboard');
    return false;
  }
}
