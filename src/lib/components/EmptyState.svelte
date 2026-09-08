<script lang="ts">
  interface Props {
    onOpenFiles: () => void;
    onOpenFolder: () => void;
  }
  let { onOpenFiles, onOpenFolder }: Props = $props();
</script>

<div
  class="flex flex-1 flex-col items-center justify-center gap-4 px-6 text-center"
>
  <div
    class="flex h-16 w-16 items-center justify-center rounded-2xl bg-surface-800 ring-1 ring-surface-700"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      class="h-8 w-8 text-accent-400"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        d="M9 19V6l12-2v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-2c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2z"
      />
    </svg>
  </div>
  <div>
    <h2 class="text-lg font-semibold tracking-tight text-surface-100">
      No tracks yet
    </h2>
    <p class="mt-1 max-w-sm text-sm text-surface-400">
      Drop audio files or a folder here, or open them from the toolbar. Edit
      titles, albums, and rename files from tags.
    </p>
  </div>
  <div class="flex flex-wrap items-center justify-center gap-2">
    <button
      type="button"
      class="rounded-lg bg-accent-600 px-4 py-2 text-sm font-medium text-white shadow hover:bg-accent-500"
      onclick={onOpenFiles}
    >
      Open files
    </button>
    <button
      type="button"
      class="rounded-lg border border-surface-600 bg-surface-800 px-4 py-2 text-sm font-medium text-surface-100 hover:bg-surface-700"
      onclick={onOpenFolder}
    >
      Open folder
    </button>
  </div>
  <p class="text-xs text-surface-600">
    Supports MP3, M4A, FLAC, OGG, WAV and more
  </p>
</div>
