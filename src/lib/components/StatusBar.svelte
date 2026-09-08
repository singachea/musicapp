<script lang="ts">
  interface Props {
    trackCount: number;
    selectedCount: number;
    errorCount: number;
    busy: boolean;
    canUndo: boolean;
    historyCount: number;
    undoLabel?: string | null;
    onUndo?: () => void;
    onOpenHistory?: () => void;
  }
  let {
    trackCount,
    selectedCount,
    errorCount,
    busy,
    canUndo,
    historyCount,
    undoLabel,
    onUndo,
    onOpenHistory,
  }: Props = $props();
</script>

<footer
  class="flex items-center gap-3 border-t border-surface-800 bg-surface-900 px-4 py-1.5 text-xs text-surface-400"
>
  <span>
    {trackCount}
    {trackCount === 1 ? "track" : "tracks"}
  </span>
  {#if selectedCount > 0}
    <span class="text-surface-600">·</span>
    <span>{selectedCount} selected</span>
  {/if}
  {#if errorCount > 0}
    <span class="text-surface-600">·</span>
    <span class="text-danger-400">{errorCount} with errors</span>
  {/if}
  <span class="ml-auto flex items-center gap-2">
    {#if historyCount > 0 && onOpenHistory}
      <button
        type="button"
        class="rounded-md px-2 py-0.5 text-surface-300 hover:bg-surface-800 hover:text-surface-100"
        onclick={onOpenHistory}
        title="Open action history"
      >
        History
        <span class="text-surface-500">({historyCount})</span>
      </button>
    {/if}
    {#if canUndo && onUndo}
      <button
        type="button"
        class="rounded-md px-2 py-0.5 font-medium text-accent-400 hover:bg-surface-800 disabled:opacity-45"
        onclick={onUndo}
        disabled={busy}
        title="Undo last disk change (⌘Z)"
      >
        Undo{undoLabel ? ` · ${undoLabel}` : ""}
      </button>
    {/if}
    {#if busy}
      <span
        class="inline-block h-2 w-2 animate-pulse rounded-full bg-accent-400"
      ></span>
      <span>Working…</span>
    {:else}
      <span class="text-surface-600">Ready</span>
    {/if}
  </span>
</footer>
