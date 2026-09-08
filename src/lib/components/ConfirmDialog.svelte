<script lang="ts">
  interface Props {
    title: string;
    body: string;
    detail?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
    busy?: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    body,
    detail,
    confirmLabel = "Continue",
    cancelLabel = "Cancel",
    danger = false,
    busy = false,
    onConfirm,
    onCancel,
  }: Props = $props();
</script>

<div
  class="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-6 backdrop-blur-sm"
  role="dialog"
  aria-modal="true"
  aria-labelledby="confirm-title"
>
  <div
    class="w-full max-w-md overflow-hidden rounded-xl border border-surface-700 bg-surface-900 shadow-2xl"
  >
    <div class="px-5 py-4">
      <h2 id="confirm-title" class="text-base font-semibold text-surface-100">
        {title}
      </h2>
      <p class="mt-2 text-sm leading-relaxed text-surface-300">{body}</p>
      {#if detail}
        <p
          class="mt-2 rounded-lg bg-surface-850 px-3 py-2 font-mono text-xs text-surface-400"
        >
          {detail}
        </p>
      {/if}
    </div>
    <footer
      class="flex items-center justify-end gap-2 border-t border-surface-800 px-5 py-3"
    >
      <button
        type="button"
        class="rounded-lg border border-surface-600 px-3 py-1.5 text-sm text-surface-200 hover:bg-surface-800 disabled:opacity-45"
        onclick={onCancel}
        disabled={busy}
      >
        {cancelLabel}
      </button>
      <button
        type="button"
        class="rounded-lg px-3 py-1.5 text-sm font-medium text-white disabled:opacity-45 {danger
          ? 'bg-danger-400/90 hover:bg-danger-400'
          : 'bg-accent-600 hover:bg-accent-500'}"
        onclick={onConfirm}
        disabled={busy}
      >
        {busy ? "Working…" : confirmLabel}
      </button>
    </footer>
  </div>
</div>
