<script lang="ts">
  import type { AppTab } from "../types";

  interface Props {
    trackCount: number;
    selectedCount: number;
    busy: boolean;
    albumDraft: string;
    artistDraft: string;
    activeTab: AppTab;
    onTabChange: (tab: AppTab) => void;
    onOpenFiles: () => void;
    onOpenFolder: () => void;
    onFilenameToTitle: () => void;
    onTitleToFilename: () => void;
    onApplyBulk: () => void;
    onClear: () => void;
    onRemoveSelected: () => void;
    onAlbumDraft: (v: string) => void;
    onArtistDraft: (v: string) => void;
  }

  let {
    trackCount,
    selectedCount,
    busy,
    albumDraft,
    artistDraft,
    activeTab,
    onTabChange,
    onOpenFiles,
    onOpenFolder,
    onFilenameToTitle,
    onTitleToFilename,
    onApplyBulk,
    onClear,
    onRemoveSelected,
    onAlbumDraft,
    onArtistDraft,
  }: Props = $props();

  let addOpen = $state(false);

  const hasTracks = $derived(trackCount > 0);
  const hasSelection = $derived(selectedCount > 0);
  const canApplyBulk = $derived(
    hasSelection && (!!albumDraft.trim() || !!artistDraft.trim()),
  );
  const selectionLabel = $derived(
    hasSelection
      ? `${selectedCount} selected`
      : "Select tracks in the table",
  );

  function pickFiles() {
    addOpen = false;
    onOpenFiles();
  }
  function pickFolder() {
    addOpen = false;
    onOpenFolder();
  }
</script>

<header class="border-b border-surface-800 bg-surface-900/95 backdrop-blur">
  <!-- Top row: brand + import + tabs + list hygiene -->
  <div class="flex flex-wrap items-center gap-3 px-4 py-2.5">
    <div class="flex items-center gap-2.5">
      <span
        class="flex h-8 w-8 items-center justify-center rounded-xl bg-accent-600/20 text-accent-400 ring-1 ring-accent-500/25"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          class="h-4 w-4"
        >
          <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z" />
        </svg>
      </span>
      <div class="hidden sm:block">
        <div class="text-sm font-semibold leading-none tracking-tight">
          Music Tags
        </div>
        <div class="mt-0.5 text-[11px] text-surface-400">
          Edit tags · rename files
        </div>
      </div>
    </div>

    <div class="relative">
      <div class="flex overflow-hidden rounded-lg shadow-sm">
        <button
          type="button"
          class="inline-flex items-center gap-1.5 bg-accent-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-500 disabled:opacity-45"
          onclick={pickFiles}
          disabled={busy}
          title="Open audio files (⌘O)"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="h-4 w-4 opacity-90"
          >
            <path
              d="M10.75 4.75a.75.75 0 0 0-1.5 0v4.5h-4.5a.75.75 0 0 0 0 1.5h4.5v4.5a.75.75 0 0 0 1.5 0v-4.5h4.5a.75.75 0 0 0 0-1.5h-4.5v-4.5Z"
            />
          </svg>
          Add files
        </button>
        <button
          type="button"
          class="border-l border-white/15 bg-accent-600 px-2 py-1.5 text-white hover:bg-accent-500 disabled:opacity-45"
          onclick={() => (addOpen = !addOpen)}
          disabled={busy}
          aria-label="More add options"
          aria-expanded={addOpen}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="h-4 w-4"
          >
            <path
              fill-rule="evenodd"
              d="M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>
      {#if addOpen}
        <div
          class="absolute left-0 top-full z-20 mt-1 min-w-[11rem] overflow-hidden rounded-lg border border-surface-700 bg-surface-850 py-1 shadow-xl"
          role="menu"
        >
          <button
            type="button"
            class="block w-full px-3 py-2 text-left text-sm text-surface-100 hover:bg-surface-800"
            onclick={pickFiles}
            role="menuitem"
          >
            Audio files…
          </button>
          <button
            type="button"
            class="block w-full px-3 py-2 text-left text-sm text-surface-100 hover:bg-surface-800"
            onclick={pickFolder}
            role="menuitem"
          >
            Folder…
          </button>
        </div>
      {/if}
    </div>

    <!-- Tabs -->
    <div
      class="flex rounded-lg border border-surface-700 bg-surface-850 p-0.5"
      role="tablist"
      aria-label="Main views"
    >
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "tags"}
        class="rounded-md px-3 py-1 text-sm font-medium transition {activeTab ===
        'tags'
          ? 'bg-surface-700 text-surface-100 shadow-sm'
          : 'text-surface-400 hover:text-surface-200'}"
        onclick={() => onTabChange("tags")}
      >
        Tags
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === "filenames"}
        class="rounded-md px-3 py-1 text-sm font-medium transition {activeTab ===
        'filenames'
          ? 'bg-surface-700 text-surface-100 shadow-sm'
          : 'text-surface-400 hover:text-surface-200'}"
        onclick={() => onTabChange("filenames")}
      >
        Filenames
      </button>
    </div>

    <div class="ml-auto flex items-center gap-1">
      {#if selectedCount > 0}
        <button
          type="button"
          class="rounded-md px-2.5 py-1.5 text-sm text-surface-400 hover:bg-surface-800 hover:text-danger-400 disabled:opacity-45"
          onclick={onRemoveSelected}
          disabled={busy}
        >
          Remove from list
        </button>
      {/if}
      <button
        type="button"
        class="rounded-md px-2.5 py-1.5 text-sm text-surface-400 hover:bg-surface-800 hover:text-surface-100 disabled:opacity-45"
        onclick={onClear}
        disabled={busy || !hasTracks}
      >
        Clear list
      </button>
    </div>
  </div>

  {#if activeTab === "tags"}
    <div
      class="border-t border-surface-800/80 px-4 py-3 {hasSelection
        ? ''
        : 'opacity-70'}"
    >
      <!-- Primary actions -->
      <div class="flex flex-wrap items-end gap-3">
        <div class="flex min-w-0 flex-1 flex-col gap-1 sm:max-w-xs">
          <span class="text-[11px] font-medium text-surface-400"
            >Quick · {selectionLabel}</span
          >
          <button
            type="button"
            class="rounded-lg border border-surface-600 bg-surface-800 px-3 py-2 text-left text-sm font-medium text-surface-100 hover:bg-surface-700 disabled:cursor-not-allowed disabled:opacity-45"
            onclick={onFilenameToTitle}
            disabled={busy || !hasSelection}
            title="Write the title tag from each file’s base name"
          >
            Use filename as title
          </button>
        </div>

        <label class="flex min-w-[14rem] flex-[2] flex-col gap-1">
          <span class="text-[11px] font-medium text-surface-400">Album</span>
          <div class="flex gap-2">
            <input
              type="text"
              class="field"
              placeholder="Set album on selected…"
              value={albumDraft}
              oninput={(e) => onAlbumDraft(e.currentTarget.value)}
              disabled={busy || !hasTracks}
            />
            <button
              type="button"
              class="shrink-0 rounded-lg bg-accent-600 px-3 py-2 text-sm font-medium text-white hover:bg-accent-500 disabled:cursor-not-allowed disabled:opacity-45"
              onclick={onApplyBulk}
              disabled={busy || !canApplyBulk}
              title="Writes album and/or artist (if set under More) to selected tracks"
            >
              {hasSelection ? `Apply · ${selectedCount}` : "Apply"}
            </button>
          </div>
        </label>
      </div>

      <!-- Secondary / less common -->
      <details class="group mt-3 border-t border-surface-800/60 pt-2">
        <summary
          class="cursor-pointer list-none text-xs text-surface-500 marker:content-none hover:text-surface-300 [&::-webkit-details-marker]:hidden"
        >
          <span class="inline-flex items-center gap-1">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 16 16"
              fill="currentColor"
              class="h-3 w-3 transition group-open:rotate-90"
            >
              <path
                fill-rule="evenodd"
                d="M6.22 4.22a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06l-3.25 3.25a.75.75 0 0 1-1.06-1.06L8.94 8 6.22 5.28a.75.75 0 0 1 0-1.06Z"
                clip-rule="evenodd"
              />
            </svg>
            More actions
            <span class="text-surface-600"
              >· rename from title, set artist</span
            >
          </span>
        </summary>

        <div
          class="mt-3 flex flex-wrap items-end gap-3 rounded-lg border border-surface-800/80 bg-surface-850/40 p-3"
        >
          <div class="flex flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-500"
              >Less common</span
            >
            <button
              type="button"
              class="rounded-md px-2.5 py-1.5 text-left text-sm text-surface-400 underline-offset-2 hover:bg-surface-800 hover:text-warning-400 hover:underline disabled:cursor-not-allowed disabled:opacity-45"
              onclick={onTitleToFilename}
              disabled={busy || !hasSelection}
              title="Rename files on disk from title tags (preview first)"
            >
              Rename files from title…
            </button>
          </div>

          <label class="flex min-w-[12rem] flex-1 flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-500">Artist</span>
            <input
              type="text"
              class="field"
              placeholder="Optional · applied with Apply above"
              value={artistDraft}
              oninput={(e) => onArtistDraft(e.currentTarget.value)}
              disabled={busy || !hasTracks}
            />
          </label>
        </div>
      </details>
    </div>
  {/if}
</header>

{#if addOpen}
  <button
    type="button"
    class="fixed inset-0 z-10 cursor-default bg-transparent"
    aria-label="Close menu"
    onclick={() => (addOpen = false)}
  ></button>
{/if}

<style>
  :global(.field) {
    width: 100%;
    border-radius: 0.5rem;
    border: 1px solid var(--color-surface-600);
    background: var(--color-surface-900);
    padding: 0.45rem 0.65rem;
    font-size: 0.875rem;
    color: var(--color-surface-100);
    outline: none;
  }
  :global(.field:focus) {
    border-color: var(--color-accent-500);
    box-shadow: 0 0 0 2px
      color-mix(in srgb, var(--color-accent-500) 25%, transparent);
  }
  :global(.field:disabled) {
    opacity: 0.5;
  }
  :global(.field::placeholder) {
    color: var(--color-surface-600);
  }
</style>
