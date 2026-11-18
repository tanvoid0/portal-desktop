<!-- Learned user preferences list. -->
<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import type { UserPreference } from '$lib/domains/learning';
  import Star from '~icons/fa6-solid/star';
  import { formatDate, formatPatternData } from './format';

  interface Props {
    preferences: UserPreference[];
  }

  let { preferences }: Props = $props();
</script>

<div class="max-h-[600px] space-y-2 overflow-y-auto">
  {#each preferences as pref}
    <Card class="p-3">
      <div class="flex items-start justify-between gap-4">
        <div class="flex-1 space-y-1">
          <div class="flex items-center gap-2">
            <Badge variant="outline">
              {pref.preference_type}
            </Badge>
            {#if pref.is_important}
              <Badge variant="default" class="bg-yellow-500">
                <Star class="mr-1 h-3 w-3" />
                Important
              </Badge>
            {/if}
            {#if pref.context}
              <Badge variant="secondary">
                {pref.context}
              </Badge>
            {/if}
            <Badge variant="outline">
              {(pref.confidence * 100).toFixed(0)}% confidence
            </Badge>
          </div>
          <div class="rounded bg-muted p-2 font-mono text-sm">
            {formatPatternData(pref.preference_value)}
          </div>
          <div class="flex items-center gap-4 text-xs text-muted-foreground">
            {#if pref.learned_from}
              <span>Learned from: {pref.learned_from}</span>
            {/if}
            <span>Updated: {formatDate(pref.updated_at)}</span>
          </div>
        </div>
      </div>
    </Card>
  {/each}
</div>
