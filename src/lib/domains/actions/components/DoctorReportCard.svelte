<!-- Suggestion that Portal's built-in default command was wrong for this profile. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Copy from '~icons/fa6-solid/copy';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import { toast } from '$lib/utils/toast';
  import type { PipelineFixReport } from '../state/doctor.svelte';
  import LabeledCommand from './LabeledCommand.svelte';

  interface Props {
    report: PipelineFixReport;
  }

  let { report: r }: Props = $props();

  function reportMarkdown(): string {
    const profile = r.frameworks.length
      ? `${r.packageManager} · ${r.frameworks.join(', ')}`
      : r.packageManager;
    return [
      `Portal default needs a fix — action \`${r.actionId}\``,
      '',
      `- Profile: ${profile}`,
      `- Portal default command (failed): \`${r.defaultCommand}\``,
      `- Command that passed: \`${r.greenCommand}\``,
      '',
      'Portal generated the wrong default for this profile. Update `buildDefaultActions`',
      'in `src/lib/domains/actions/defaults.ts` so this profile maps to the passing command.',
    ].join('\n');
  }

  async function copyReport() {
    try {
      await navigator.clipboard.writeText(reportMarkdown());
      toast.success('Report copied to clipboard');
    } catch {
      toast.error('Could not copy report');
    }
  }
</script>

<SectionCard
  title="Portal default needs a fix"
  titleClass="text-base"
  headerClass="pb-2"
  class="border-amber-500/40"
  contentClass="space-y-2"
>
  {#snippet description()}
    A <code class="text-xs">.portal/pipeline.yml</code> override fixed the
    <code class="text-xs">{r.actionId}</code> action. Portal's built-in default was wrong for this profile
    — review it, nothing was changed in Portal.
  {/snippet}
  {#snippet actions()}
    <Badge variant="outline" class="shrink-0">suggestion</Badge>
  {/snippet}
  <p class="text-xs text-muted-foreground">
    Profile: {r.packageManager}{r.frameworks.length ? ` · ${r.frameworks.join(', ')}` : ''}
  </p>
  <LabeledCommand label="Portal default (failed)" command={r.defaultCommand} />
  <LabeledCommand label="Command that passed" command={r.greenCommand} />
  <Button type="button" size="sm" variant="outline" class="gap-1" onclick={copyReport}>
    <Copy class="h-3.5 w-3.5" />
    Copy report
  </Button>
</SectionCard>
