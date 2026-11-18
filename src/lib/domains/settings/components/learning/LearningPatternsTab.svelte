<!-- Learned patterns list with search, type filter and important toggle. -->
<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import Select from '$lib/components/ui/select.svelte';
  import type { LearnedPattern } from '$lib/domains/learning';
  import Star from '~icons/fa6-solid/star';
  import TrendingUp from '~icons/fa6-solid/arrow-trend-up';
  import Clock from '~icons/fa6-solid/clock';
  import CheckCircle2 from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import { formatDate, formatPatternData, getPatternTypeLabel } from './format';

  interface Props {
    patterns: LearnedPattern[];
    onToggleImportant: (id: number, isImportant: boolean) => void;
  }

  let { patterns, onToggleImportant }: Props = $props();

  let patternsFilter = $state<'all' | 'command' | 'framework' | 'workflow' | 'code'>('all');
  let searchQuery = $state('');

  const filteredPatterns = $derived.by(() => {
    let filtered = patterns;

    if (patternsFilter !== 'all') {
      filtered = filtered.filter((p) => p.pattern_type === patternsFilter);
    }

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((p) => {
        const dataStr = JSON.stringify(p.pattern_data).toLowerCase();
        const contextStr = (p.context || '').toLowerCase();
        return dataStr.includes(query) || contextStr.includes(query);
      });
    }

    return filtered;
  });
</script>

<div class="space-y-2">
  <div class="flex items-center gap-2">
    <div class="flex flex-1 items-center gap-2">
      <Search class="h-4 w-4 text-muted-foreground" />
      <Input bind:value={searchQuery} placeholder="Search patterns..." class="max-w-sm flex-1" />
    </div>
    <Select
      options={[
        { value: 'all', label: 'All Types' },
        { value: 'command', label: 'Commands' },
        { value: 'framework', label: 'Frameworks' },
        { value: 'workflow', label: 'Workflows' },
        { value: 'code', label: 'Code' },
      ]}
      defaultValue={patternsFilter}
      onSelect={(value) => (patternsFilter = (value || 'all') as typeof patternsFilter)}
    />
  </div>

  <div class="max-h-[600px] space-y-2 overflow-y-auto">
    {#each filteredPatterns as pattern (pattern.id)}
      <Card class="p-4">
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1 space-y-2">
            <div class="flex items-center gap-2">
              <Badge variant="outline">
                {getPatternTypeLabel(pattern.pattern_type)}
              </Badge>
              {#if pattern.is_important}
                <Badge variant="default" class="bg-yellow-500">
                  <Star class="mr-1 h-3 w-3" />
                  Important
                </Badge>
              {/if}
              {#if pattern.context}
                <Badge variant="secondary">
                  {pattern.context}
                </Badge>
              {/if}
            </div>
            <div class="max-h-32 overflow-y-auto rounded bg-muted p-2 font-mono text-sm text-xs">
              {formatPatternData(pattern.pattern_data)}
            </div>
            <div class="flex items-center gap-4 text-xs text-muted-foreground">
              <div class="flex items-center gap-1">
                <TrendingUp class="h-3 w-3" />
                Used {pattern.frequency} times
              </div>
              <div class="flex items-center gap-1">
                {#if pattern.success_rate >= 0.7}
                  <CheckCircle2 class="h-3 w-3 text-green-500" />
                {:else}
                  <XCircle class="h-3 w-3 text-red-500" />
                {/if}
                {(pattern.success_rate * 100).toFixed(0)}% success
              </div>
              {#if pattern.last_used}
                <div class="flex items-center gap-1">
                  <Clock class="h-3 w-3" />
                  {formatDate(pattern.last_used)}
                </div>
              {/if}
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onclick={() => onToggleImportant(pattern.id, !pattern.is_important)}
            title={pattern.is_important ? 'Mark as not important' : 'Mark as important'}
          >
            <Star class="h-4 w-4" fill={pattern.is_important ? 'currentColor' : 'none'} />
          </Button>
        </div>
      </Card>
    {/each}
  </div>

  {#if filteredPatterns.length === 0 && patterns.length > 0}
    <div class="py-4 text-center text-sm text-muted-foreground">No patterns match your filter</div>
  {/if}
</div>
