<!-- Log Entry Component - Displays formatted log entries -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Copy from '~icons/fa6-solid/copy';
  import { toastActions } from '$lib/domains/shared/stores/toastStore';
  import ExpandableText from '$lib/components/shared/ExpandableText.svelte';
  import { getSeverityColor } from '../../utils/logSeverity';
  import LogSeverityBadge from './LogSeverityBadge.svelte';
  import LogStructuredFields from './LogStructuredFields.svelte';
  import type { K8sLog } from '../../types/k8s';

  interface Props {
    log: K8sLog;
    viewMode?: 'detailed' | 'compact' | 'raw';
    onFilterBySeverity?: (severity: string) => void;
  }

  let { log, viewMode = 'detailed', onFilterBySeverity }: Props = $props();

  // Parse structured log if it's JSON
  let parsedLog = $derived.by(() => {
    try {
      const parsed = JSON.parse(log.message);
      return {
        isStructured: true,
        data: parsed,
        rawMessage: log.message,
      };
    } catch {
      return {
        isStructured: false,
        data: null,
        rawMessage: log.message,
      };
    }
  });

  // Extract structured fields
  let structuredFields = $derived.by(() => {
    if (!parsedLog.isStructured || !parsedLog.data) {
      return null;
    }

    const data = parsedLog.data;
    return {
      severity: data.severity || data.level || data.logLevel || log.level,
      time: data.time || data.timestamp || data.ts || log.timestamp,
      message: data.message || data.msg || data.text || log.message,
      logger: data.logger || data.component || data.service,
      requestId: data.requestId || data.request_id || data.trace_id,
      traceId: data.traceId || data.trace_id || data.traceID || data.trace,
      error: data.error || data.err || data.exception,
      stackTrace: data.stackTrace || data.stack_trace || data.stack,
      method: data.method || data.httpMethod,
      url: data.url || data.path || data.endpoint,
      statusCode: data.statusCode || data.status_code || data.status,
      duration: data.duration || data.elapsed || data.responseTime,
    };
  });

  function formatTimestamp(timestamp: string): string {
    try {
      return new Date(timestamp).toLocaleString();
    } catch {
      return timestamp;
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toastActions.success('Copied to clipboard');
    } catch (err) {
      toastActions.error('Failed to copy');
    }
  }

  function handleSeverityClick(severity: string) {
    if (onFilterBySeverity) {
      onFilterBySeverity(severity);
    }
  }
</script>

<div class="mb-2 rounded-md p-3 {getSeverityColor(structuredFields?.severity || log.level)}">
  {#if viewMode === 'raw'}
    <!-- Raw Mode -->
    <div class="font-mono text-xs break-words whitespace-pre-wrap">
      <span class="text-slate-500 dark:text-slate-400">[{formatTimestamp(log.timestamp)}]</span>
      <span class="text-slate-600 dark:text-slate-300">[{log.level}]</span>
      <span class="ml-2">{parsedLog.rawMessage}</span>
      <Button
        variant="ghost"
        size="sm"
        onclick={() =>
          copyToClipboard(
            `[${formatTimestamp(log.timestamp)}] [${log.level}] ${parsedLog.rawMessage}`
          )}
        class="ml-2 h-6 w-6 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
        title="Copy log"
      >
        <Copy class="h-3 w-3" />
      </Button>
    </div>
  {:else if viewMode === 'compact'}
    <!-- Compact Mode -->
    <div class="flex items-center space-x-2 text-sm">
      <LogSeverityBadge
        severity={structuredFields?.severity || log.level}
        onclick={handleSeverityClick}
      />
      <span class="font-mono text-xs text-slate-500 dark:text-slate-400">
        {formatTimestamp(structuredFields?.time || log.timestamp)}
      </span>
      <span class="min-w-0 flex-1 break-words">
        {structuredFields?.message || parsedLog.rawMessage}
      </span>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => copyToClipboard(structuredFields?.message || parsedLog.rawMessage)}
        class="h-6 w-6 flex-shrink-0 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
        title="Copy message"
      >
        <Copy class="h-3 w-3" />
      </Button>
    </div>
  {:else}
    <!-- Detailed Mode -->
    <div class="space-y-2">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div class="flex flex-wrap items-center space-x-2">
          <LogSeverityBadge
            severity={structuredFields?.severity || log.level}
            onclick={handleSeverityClick}
          />
          <span class="font-mono text-xs text-slate-500 dark:text-slate-400">
            {formatTimestamp(structuredFields?.time || log.timestamp)}
          </span>
          <span class="text-xs text-slate-500 dark:text-slate-400">
            {log.pod}
            {#if log.container && log.container !== 'app'}
              /{log.container}
            {/if}
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => copyToClipboard(log.message)}
          class="h-8 w-8 p-0 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
          title="Copy log"
        >
          <Copy class="h-4 w-4" />
        </Button>
      </div>

      <!-- Message -->
      <div class="text-sm break-words">
        {#if structuredFields?.message}
          <div class="rounded border bg-white p-2 font-mono dark:bg-slate-800">
            <ExpandableText text={String(structuredFields.message)} />
          </div>
        {:else}
          <div class="rounded border bg-white p-2 font-mono whitespace-pre-wrap dark:bg-slate-800">
            <ExpandableText text={parsedLog.rawMessage} />
          </div>
        {/if}
      </div>

      {#if structuredFields}
        <LogStructuredFields fields={structuredFields} />
      {/if}

      <!-- Error -->
      {#if structuredFields?.error}
        <div class="text-sm">
          <span class="font-medium text-red-600 dark:text-red-400">Error:</span>
          <div
            class="mt-1 rounded border border-red-200 bg-red-50 p-2 font-mono break-words text-red-900 dark:border-red-800 dark:bg-red-900/20 dark:text-red-100"
          >
            <ExpandableText text={String(structuredFields.error)} />
          </div>
        </div>
      {/if}

      <!-- Stack Trace -->
      {#if structuredFields?.stackTrace}
        <div class="text-sm">
          <span class="font-medium text-red-600 dark:text-red-400">Stack Trace:</span>
          <div
            class="mt-1 rounded border border-red-200 bg-red-50 p-2 font-mono text-xs break-words whitespace-pre-wrap text-red-900 dark:border-red-800 dark:bg-red-900/20 dark:text-red-100"
          >
            <ExpandableText text={String(structuredFields.stackTrace)} limit={300} />
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
