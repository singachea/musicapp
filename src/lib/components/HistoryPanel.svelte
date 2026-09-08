<script lang="ts">
  import type { HistoryEntry } from "../types";

  interface Props {
    entries: HistoryEntry[];
    busy: boolean;
    onClose: () => void;
    onUndoLatest: () => void;
    onUndoTo: (id: number) => void;
    onClear: () => void;
  }

  let { entries, busy, onClose, onUndoLatest, onUndoTo, onClear }: Props =
    $props();

  const newestFirst = $derived([...entries].reverse());
  const activeCount = $derived(entries.filter((e) => !e.undone).length);

  function formatTime(at: number): string {
    return new Date(at).toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function kindLabel(e: HistoryEntry): string {
    return e.payload.kind === "tags" ? "Tags" : "Rename";
  }

  /** Newest active entry can undo alone; older active → “Undo to here”. */
  function isLatestActive(e: HistoryEntry): boolean {
    if (e.undone) return false;
    const active = entries.filter((x) => !x.undone);
    return active.length > 0 && active[active.length - 1].id === e.id;
  }

  function canUndoTo(e: HistoryEntry): boolean {
    return !e.undone && !isLatestActive(e);
  }
</script>

<div
  class="fixed inset-0 z-40 flex justify-end bg-black/50 backdrop-blur-sm"
  role="dialog"
  aria-modal="true"
  aria-labelledby="history-title"
>
  <button
    type="button"
    class="min-w-0 flex-1 cursor-default"
    aria-label="Close history"
    onclick={onClose}
  ></button>

  <aside
    class="flex h-full w-full max-w-md flex-col border-l border-surface-700 bg-surface-900 shadow-2xl"
  >
    <header
      class="flex items-center justify-between border-b border-surface-800 px-4 py-3"
    >
      <div>
        <h2 id="history-title" class="text-sm font-semibold text-surface-100">
          Action history
        </h2>
        <p class="mt-0.5 text-[11px] text-surface-400">
          {activeCount} can be undone · {entries.length} total
        </p>
      </div>
      <div class="flex items-center gap-1">
        {#if activeCount > 0}
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs font-medium text-accent-400 hover:bg-surface-800 disabled:opacity-45"
            onclick={onUndoLatest}
            disabled={busy}
            title="⌘Z"
          >
            Undo last
          </button>
        {/if}
        <button
          type="button"
          class="rounded-md px-2 py-1 text-xs text-surface-400 hover:bg-surface-800 hover:text-surface-200"
          onclick={onClose}
        >
          Close
        </button>
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto">
      {#if newestFirst.length === 0}
        <p class="px-4 py-8 text-center text-sm text-surface-500">
          No disk changes yet. Edits and renames will show up here so you can
          undo them.
        </p>
      {:else}
        <ul class="divide-y divide-surface-800/80">
          {#each newestFirst as entry (entry.id)}
            <li
              class="px-4 py-3 {entry.undone
                ? 'opacity-50'
                : 'bg-surface-900'}"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0">
                  <div class="flex flex-wrap items-center gap-2">
                    <span
                      class="text-sm font-medium {entry.undone
                        ? 'text-surface-500 line-through'
                        : 'text-surface-100'}"
                    >
                      {entry.label}
                    </span>
                    <span
                      class="rounded bg-surface-800 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-surface-400"
                    >
                      {kindLabel(entry)}
                    </span>
                    {#if entry.undone}
                      <span class="text-[10px] text-surface-600">undone</span>
                    {/if}
                  </div>
                  <p class="mt-0.5 text-xs text-surface-400">
                    {entry.detail}
                    <span class="text-surface-600">·</span>
                    {formatTime(entry.at)}
                  </p>
                </div>
                <div class="flex shrink-0 flex-col items-end gap-1">
                  {#if isLatestActive(entry)}
                    <button
                      type="button"
                      class="rounded-md bg-accent-600/20 px-2 py-1 text-xs font-medium text-accent-400 ring-1 ring-accent-500/30 hover:bg-accent-600/30 disabled:opacity-45"
                      onclick={onUndoLatest}
                      disabled={busy}
                    >
                      Undo
                    </button>
                  {:else if canUndoTo(entry)}
                    <button
                      type="button"
                      class="rounded-md px-2 py-1 text-xs text-surface-400 hover:bg-surface-800 hover:text-surface-200 disabled:opacity-45"
                      onclick={() => onUndoTo(entry.id)}
                      disabled={busy}
                      title="Undo this and every action after it"
                    >
                      Undo to here
                    </button>
                  {/if}
                </div>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    {#if entries.length > 0}
      <footer class="border-t border-surface-800 px-4 py-2">
        <button
          type="button"
          class="text-xs text-surface-500 hover:text-danger-400 disabled:opacity-45"
          onclick={onClear}
          disabled={busy}
        >
          Clear history list
        </button>
        <p class="mt-1 text-[10px] text-surface-600">
          Clearing the list does not change files. Undo actually rewrites tags /
          renames on disk.
        </p>
      </footer>
    {/if}
  </aside>
</div>
