<script lang="ts">
  import type { RenamePlan, TrackInfo } from "../types";
  import {
    TOOL_META,
    buildRenamePlan,
    defaultToolState,
    transformForTool,
    type FilenameToolId,
    type FilenameToolState,
  } from "../filenameOps";

  interface Props {
    tracks: TrackInfo[];
    selectedCount: number;
    busy: boolean;
    onPreview: (plan: RenamePlan) => void;
  }

  let { tracks, selectedCount, busy, onPreview }: Props = $props();

  let tool = $state<FilenameToolId>("prefix");
  let form = $state<FilenameToolState>(defaultToolState());

  const scopeLabel = $derived(
    selectedCount > 0
      ? `${selectedCount} selected`
      : "select tracks first",
  );

  const canRun = $derived(selectedCount > 0 && tracks.length > 0 && !busy);

  const livePlan = $derived.by(() => {
    if (tracks.length === 0) return null;
    const fn = transformForTool(tool, form);
    if (!fn) return null;
    return buildRenamePlan(tracks, fn);
  });

  const previewRows = $derived(
    livePlan?.items.slice(0, 8) ?? [],
  );

  function runPreview() {
    const fn = transformForTool(tool, form);
    if (!fn) return;
    const plan = buildRenamePlan(tracks, fn);
    onPreview(plan);
  }

  const tools = Object.keys(TOOL_META) as FilenameToolId[];
</script>

<section
  class="border-b border-surface-800 bg-surface-900/80 {selectedCount === 0
    ? 'opacity-75'
    : ''}"
>
  <div class="grid gap-0 lg:grid-cols-[14rem_1fr]">
    <!-- Tool picker -->
    <nav
      class="flex gap-1 overflow-x-auto border-b border-surface-800 p-2 lg:flex-col lg:border-b-0 lg:border-r"
      aria-label="Filename tools"
    >
      {#each tools as id}
        <button
          type="button"
          class="rounded-lg px-3 py-2 text-left text-sm transition {tool === id
            ? 'bg-accent-600/20 font-medium text-accent-400 ring-1 ring-accent-500/30'
            : 'text-surface-300 hover:bg-surface-800 hover:text-surface-100'}"
          onclick={() => (tool = id)}
        >
          {TOOL_META[id].label}
        </button>
      {/each}
    </nav>

    <!-- Tool body -->
    <div class="flex flex-col gap-3 p-4">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2 class="text-sm font-semibold text-surface-100">
            {TOOL_META[tool].label}
          </h2>
          <p class="mt-0.5 text-xs text-surface-400">
            {TOOL_META[tool].blurb} · applies to <span class="text-surface-300"
              >{scopeLabel}</span
            > · extension never changes
          </p>
        </div>
        <button
          type="button"
          class="rounded-lg bg-accent-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-500 disabled:cursor-not-allowed disabled:opacity-45"
          disabled={!canRun || !livePlan || livePlan.applyableCount === 0}
          onclick={runPreview}
          title={selectedCount === 0
            ? "Select one or more tracks in the table first"
            : "Review renames before applying"}
        >
          Preview & rename…
        </button>
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        {#if tool === "prefix"}
          <label class="flex flex-col gap-1 sm:col-span-2">
            <span class="text-[11px] font-medium text-surface-400">Prefix</span>
            <input
              class="field"
              placeholder="e.g. 2024 - "
              bind:value={form.prefix}
              disabled={busy}
            />
          </label>
        {:else if tool === "suffix"}
          <label class="flex flex-col gap-1 sm:col-span-2">
            <span class="text-[11px] font-medium text-surface-400"
              >Suffix (before extension)</span
            >
            <input
              class="field"
              placeholder="e.g. (Remastered)"
              bind:value={form.suffix}
              disabled={busy}
            />
          </label>
        {:else if tool === "replace"}
          <label class="flex flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-400">Find</span>
            <input
              class="field font-mono text-sm"
              placeholder={form.useRegex ? "e.g. _+" : "e.g. _"}
              bind:value={form.find}
              disabled={busy}
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-400"
              >Replace with</span
            >
            <input
              class="field font-mono text-sm"
              placeholder="e.g. space or empty"
              bind:value={form.replaceWith}
              disabled={busy}
            />
          </label>
          <label class="flex items-center gap-2 text-sm text-surface-300 sm:col-span-2">
            <input type="checkbox" bind:checked={form.useRegex} disabled={busy} />
            Use regular expression
          </label>
        {:else if tool === "remove"}
          <label class="flex flex-col gap-1 sm:col-span-2">
            <span class="text-[11px] font-medium text-surface-400"
              >Text to remove</span
            >
            <input
              class="field font-mono text-sm"
              placeholder={form.removeRegex ? "e.g. \\s*\\[.*?\\]" : "e.g. (Official Video)"}
              bind:value={form.removePattern}
              disabled={busy}
            />
          </label>
          <label class="flex items-center gap-2 text-sm text-surface-300 sm:col-span-2">
            <input
              type="checkbox"
              bind:checked={form.removeRegex}
              disabled={busy}
            />
            Use regular expression
          </label>
        {:else if tool === "case"}
          <fieldset class="flex flex-wrap gap-3 sm:col-span-2">
            <legend class="mb-1 text-[11px] font-medium text-surface-400">
              Case style
            </legend>
            {#each [
              ["lower", "lowercase"],
              ["upper", "UPPERCASE"],
              ["title", "Title Case"],
            ] as [value, label]}
              <label class="flex items-center gap-2 text-sm text-surface-200">
                <input
                  type="radio"
                  name="caseMode"
                  checked={form.caseMode === value}
                  onchange={() =>
                    (form.caseMode = value as FilenameToolState["caseMode"])}
                  disabled={busy}
                />
                {label}
              </label>
            {/each}
          </fieldset>
        {:else if tool === "spaces"}
          <p class="text-sm text-surface-400 sm:col-span-2">
            Trims leading/trailing spaces and collapses runs of whitespace to a
            single space. No extra options.
          </p>
        {:else if tool === "number"}
          <label class="flex flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-400"
              >Start at</span
            >
            <input
              class="field"
              type="number"
              min="0"
              bind:value={form.numberStart}
              disabled={busy}
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-[11px] font-medium text-surface-400"
              >Zero-pad width</span
            >
            <input
              class="field"
              type="number"
              min="1"
              max="6"
              bind:value={form.numberPad}
              disabled={busy}
            />
          </label>
          <label class="flex flex-col gap-1 sm:col-span-2">
            <span class="text-[11px] font-medium text-surface-400"
              >Separator after number</span
            >
            <input
              class="field font-mono text-sm"
              placeholder=" e.g.  -  or  _ "
              bind:value={form.numberSeparator}
              disabled={busy}
            />
          </label>
        {/if}
      </div>

      <!-- Live mini preview -->
      {#if livePlan && livePlan.items.length > 0}
        <div
          class="overflow-hidden rounded-lg border border-surface-800 bg-surface-950/50"
        >
          <div
            class="flex items-center justify-between border-b border-surface-800 px-3 py-1.5 text-[11px] text-surface-400"
          >
            <span>Live preview</span>
            <span>
              {livePlan.applyableCount} will rename
              {#if livePlan.collisionCount > 0}
                · <span class="text-warning-400"
                  >{livePlan.collisionCount} skipped</span
                >
              {/if}
            </span>
          </div>
          <table class="w-full text-left text-xs">
            <tbody>
              {#each previewRows as item}
                <tr class="border-t border-surface-800/60">
                  <td
                    class="max-w-[10rem] truncate px-3 py-1.5 font-mono text-surface-400"
                  >
                    {item.from.split(/[/\\]/).pop()}
                  </td>
                  <td class="px-1 text-surface-600">→</td>
                  <td
                    class="max-w-[12rem] truncate px-3 py-1.5 font-mono {item.skipped
                      ? 'text-surface-600'
                      : 'text-surface-100'}"
                  >
                    {item.to ? item.to.split(/[/\\]/).pop() : "—"}
                    {#if item.reason}
                      <span class="ml-1 text-[10px] text-warning-400"
                        >{item.reason}</span
                      >
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if livePlan.items.length > previewRows.length}
            <p class="border-t border-surface-800 px-3 py-1.5 text-[11px] text-surface-600">
              +{livePlan.items.length - previewRows.length} more — full list in the
              rename dialog
            </p>
          {/if}
        </div>
      {:else if selectedCount === 0}
        <p class="text-xs text-warning-400">
          Check one or more tracks in the table — renames only apply to selected
          files.
        </p>
      {:else if tracks.length > 0}
        <p class="text-xs text-surface-600">
          Fill in the options above to see a live preview.
        </p>
      {/if}
    </div>
  </div>
</section>

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
</style>
