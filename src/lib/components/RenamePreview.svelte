<script lang="ts">
  import type { RenamePlan } from "../types";

  interface Props {
    plan: RenamePlan;
    onCancel: () => void;
    onApply: () => void;
    applying?: boolean;
  }
  let { plan, onCancel, onApply, applying = false }: Props = $props();
</script>

<div
  class="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-6 backdrop-blur-sm"
  role="dialog"
  aria-modal="true"
>
  <div
    class="flex max-h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl border border-surface-700 bg-surface-900 shadow-2xl"
  >
    <header class="border-b border-surface-800 px-5 py-4">
      <h2 class="text-base font-semibold text-surface-100">
        Rename from titles
      </h2>
      <p class="mt-1 text-sm text-surface-400">
        {plan.applyableCount} will rename
        {#if plan.collisionCount > 0}
          · <span class="text-warning-400"
            >{plan.collisionCount} skipped (collisions / empty)</span
          >
        {/if}
      </p>
    </header>

    <div class="min-h-0 flex-1 overflow-auto">
      <table class="w-full text-left text-sm">
        <thead
          class="sticky top-0 bg-surface-850 text-xs uppercase tracking-wide text-surface-400"
        >
          <tr>
            <th class="px-4 py-2 font-medium">From</th>
            <th class="px-4 py-2 font-medium">To</th>
            <th class="px-4 py-2 font-medium">Status</th>
          </tr>
        </thead>
        <tbody>
          {#each plan.items as item}
            <tr
              class="border-t border-surface-800/80 {item.skipped ||
              item.collision
                ? 'opacity-70'
                : ''}"
            >
              <td class="max-w-[12rem] truncate px-4 py-2 font-mono text-xs text-surface-300">
                {item.from.split(/[/\\]/).pop()}
              </td>
              <td class="max-w-[14rem] truncate px-4 py-2 font-mono text-xs text-surface-100">
                {item.to ? item.to.split(/[/\\]/).pop() : "—"}
              </td>
              <td class="px-4 py-2 text-xs">
                {#if item.collision}
                  <span class="text-warning-400"
                    >{item.reason ?? "Collision"}</span
                  >
                {:else if item.skipped}
                  <span class="text-surface-400"
                    >{item.reason ?? "Skipped"}</span
                  >
                {:else}
                  <span class="text-success-400">Ready</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <footer
      class="flex items-center justify-end gap-2 border-t border-surface-800 px-5 py-3"
    >
      <button
        type="button"
        class="rounded-lg border border-surface-600 px-3 py-1.5 text-sm text-surface-200 hover:bg-surface-800"
        onclick={onCancel}
        disabled={applying}
      >
        Cancel
      </button>
      <button
        type="button"
        class="rounded-lg bg-accent-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-500 disabled:cursor-not-allowed disabled:opacity-50"
        onclick={onApply}
        disabled={applying || plan.applyableCount === 0}
      >
        {applying ? "Renaming…" : `Apply ${plan.applyableCount}`}
      </button>
    </footer>
  </div>
</div>
