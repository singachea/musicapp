<script lang="ts">
  import type { TrackInfo } from "../types";
  import { trackStore } from "../stores/tracks.svelte";

  type EditField = "filename" | "title" | "album" | "artist";

  interface Props {
    tracks: TrackInfo[];
    onCommitTag: (
      path: string,
      field: "title" | "album" | "artist",
      value: string,
    ) => void;
    onCommitFilename: (path: string, stem: string) => void;
    onOpenDetail: (path: string) => void;
  }
  let { tracks, onCommitTag, onCommitFilename, onOpenDetail }: Props =
    $props();

  let editing = $state<{ path: string; field: EditField } | null>(null);
  let editValue = $state("");

  function stemOf(filename: string): string {
    const i = filename.lastIndexOf(".");
    if (i <= 0) return filename;
    return filename.slice(0, i);
  }

  function extOf(filename: string): string {
    const i = filename.lastIndexOf(".");
    if (i <= 0) return "";
    return filename.slice(i); // includes dot
  }

  function parentDir(path: string): string {
    const parts = path.split(/[/\\]/);
    parts.pop();
    return parts.join("/") || path;
  }

  function startEdit(path: string, field: EditField, value: string) {
    editing = { path, field };
    editValue = value;
  }

  function commit() {
    if (!editing) return;
    const { path, field } = editing;
    const value = editValue;
    editing = null;
    if (field === "filename") {
      onCommitFilename(path, value.trim());
    } else {
      onCommitTag(path, field, value);
    }
  }

  function cancel() {
    editing = null;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      commit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancel();
    }
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  const allSelected = $derived(
    tracks.length > 0 && tracks.every((t) => trackStore.isSelected(t.path)),
  );

  function toggleAll(e: Event) {
    const checked = (e.currentTarget as HTMLInputElement).checked;
    if (checked) trackStore.selectAll();
    else trackStore.selectNone();
  }
</script>

<div class="min-h-0 flex-1 overflow-auto">
  <table class="w-full border-collapse text-left text-sm">
    <thead
      class="sticky top-0 z-10 bg-surface-900/95 text-[11px] uppercase tracking-wide text-surface-400 backdrop-blur"
    >
      <tr class="border-b border-surface-800">
        <th class="w-10 px-3 py-2">
          <input
            type="checkbox"
            checked={allSelected}
            onchange={toggleAll}
            aria-label="Select all"
          />
        </th>
        <th class="min-w-[12rem] px-3 py-2 font-medium">Filename</th>
        <th class="px-3 py-2 font-medium">Title</th>
        <th class="px-3 py-2 font-medium">Album</th>
        <th class="px-3 py-2 font-medium">Artist</th>
        <th class="w-16 px-2 py-2 text-center font-medium">Detail</th>
      </tr>
    </thead>
    <tbody>
      {#each tracks as track (track.path)}
        {@const stem = stemOf(track.filename)}
        {@const ext = extOf(track.filename)}
        <tr
          class="border-b border-surface-800/60 transition-colors hover:bg-surface-850/80 {trackStore.isSelected(
            track.path,
          )
            ? 'bg-accent-600/10'
            : ''} {track.error ? 'bg-danger-400/5' : ''}"
          title={parentDir(track.path)}
        >
          <td class="px-3 py-1.5 align-middle">
            <input
              type="checkbox"
              checked={trackStore.isSelected(track.path)}
              onchange={(e) =>
                trackStore.setSelected(track.path, e.currentTarget.checked)}
              aria-label="Select {track.filename}"
            />
          </td>

          <!-- Filename: edit stem only; extension locked -->
          <td
            class="max-w-[16rem] cursor-text px-1 py-0.5"
            ondblclick={() => startEdit(track.path, "filename", stem)}
          >
            {#if editing?.path === track.path && editing?.field === "filename"}
              <div class="flex items-center gap-0.5 px-1">
                <input
                  class="field min-w-0 flex-1 py-1 font-mono text-xs"
                  bind:value={editValue}
                  onblur={commit}
                  onkeydown={onKey}
                  use:focusOnMount
                  aria-label="Filename without extension"
                />
                {#if ext}
                  <span
                    class="shrink-0 select-none font-mono text-xs text-surface-600"
                    title="Extension is not editable"
                    >{ext}</span
                  >
                {/if}
              </div>
            {:else}
              <div
                class="flex items-center gap-1 rounded px-2 py-1.5 font-mono text-xs hover:bg-surface-800"
                title="{track.path} — double-click to rename (extension locked)"
              >
                <span class="min-w-0 truncate text-surface-100">{stem}</span>
                {#if ext}
                  <span class="shrink-0 text-surface-600">{ext}</span>
                {/if}
                {#if track.error}
                  <span
                    class="ml-1 shrink-0 text-[10px] text-danger-400"
                    title={track.error}>err</span
                  >
                {/if}
              </div>
            {/if}
          </td>

          {#each ["title", "album", "artist"] as field}
            {@const f = field as "title" | "album" | "artist"}
            <td
              class="max-w-[12rem] cursor-text truncate px-1 py-0.5"
              ondblclick={() => startEdit(track.path, f, track[f] ?? "")}
            >
              {#if editing?.path === track.path && editing?.field === f}
                <input
                  class="field w-full py-1 text-sm"
                  bind:value={editValue}
                  onblur={commit}
                  onkeydown={onKey}
                  use:focusOnMount
                />
              {:else}
                <div
                  class="rounded px-2 py-1.5 text-surface-100 hover:bg-surface-800"
                  title="Double-click to edit"
                >
                  {track[f] || "—"}
                </div>
              {/if}
            </td>
          {/each}

          <td class="px-2 py-1 text-center align-middle">
            <button
              type="button"
              class="inline-flex h-8 w-8 items-center justify-center rounded-lg text-surface-400 hover:bg-surface-800 hover:text-accent-400"
              title="Show metadata & play"
              aria-label="Details for {track.filename}"
              onclick={(e) => {
                e.stopPropagation();
                onOpenDetail(track.path);
              }}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                class="h-4 w-4"
              >
                <path
                  fill-rule="evenodd"
                  d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-7-4a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM9 9a.75.75 0 0 0 0 1.5h.253a.25.25 0 0 1 .244.304l-.459 2.066A1.75 1.75 0 0 0 10.747 15H11a.75.75 0 0 0 0-1.5h-.253a.25.25 0 0 1-.244-.304l.459-2.066A1.75 1.75 0 0 0 9.253 9H9Z"
                  clip-rule="evenodd"
                />
              </svg>
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
