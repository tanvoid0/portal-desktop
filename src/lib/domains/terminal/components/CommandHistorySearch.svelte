<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Search, X } from "@lucide/svelte";
  import { commandHistoryStore } from "../stores/commandHistoryStore";

  export const tabId: string = "";

  let searchQuery = "";
  let isSearching = false;

  function handleSearch() {
    if (searchQuery.trim()) {
      isSearching = true;
      commandHistoryStore.setSearchQuery(searchQuery);
    } else {
      clearSearch();
    }
  }

  function clearSearch() {
    searchQuery = "";
    isSearching = false;
    commandHistoryStore.setSearchQuery("");
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      handleSearch();
    } else if (event.key === "Escape") {
      clearSearch();
    }
  }
</script>

<div class="command-history-search border-b border-border bg-muted/50 p-3">
  <div class="flex items-center gap-2">
    <div class="relative flex-1">
      <Search
        class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
      />
      <Input
        bind:value={searchQuery}
        placeholder="Search command history..."
        class="pl-10 pr-10"
        onkeydown={handleKeydown}
      />
      {#if searchQuery}
        <Button
          variant="ghost"
          size="sm"
          onclick={clearSearch}
          class="absolute right-1 top-1/2 h-6 w-6 -translate-y-1/2 transform p-0"
        >
          <X class="h-4 w-4" />
        </Button>
      {/if}
    </div>

    <Button
      variant="outline"
      size="sm"
      onclick={handleSearch}
      disabled={!searchQuery.trim()}
    >
      Search
    </Button>

    {#if isSearching}
      <Button variant="ghost" size="sm" onclick={clearSearch}>Clear</Button>
    {/if}
  </div>

  {#if isSearching}
    <div class="mt-2 text-xs text-muted-foreground">
      Searching for: "{searchQuery}"
    </div>
  {/if}
</div>
