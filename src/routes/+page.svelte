<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/api";
  import type {
    AppTab,
    HistoryEntry,
    RenamePlan,
    TagUpdate,
    TrackDetails,
    TrackInfo,
  } from "$lib/types";
  import { trackStore } from "$lib/stores/tracks.svelte";
  import { toastStore } from "$lib/stores/toasts.svelte";
  import { historyStore } from "$lib/stores/history.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import TrackTable from "$lib/components/TrackTable.svelte";
  import FilenameTools from "$lib/components/FilenameTools.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import RenamePreview from "$lib/components/RenamePreview.svelte";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import HistoryPanel from "$lib/components/HistoryPanel.svelte";
  import TrackDetailPanel from "$lib/components/TrackDetailPanel.svelte";

  const AUDIO_FILTER = {
    name: "Audio",
    extensions: [
      "mp3",
      "m4a",
      "m4b",
      "aac",
      "flac",
      "ogg",
      "oga",
      "opus",
      "wav",
      "aiff",
      "aif",
      "wma",
    ],
  };

  let albumDraft = $state("");
  let artistDraft = $state("");
  let renamePlan = $state<RenamePlan | null>(null);
  let applyingRename = $state(false);
  let dragOver = $state(false);
  let activeTab = $state<AppTab>("tags");
  let historyOpen = $state(false);
  let detailOpen = $state(false);
  let detailLoading = $state(false);
  let detailError = $state<string | null>(null);
  let detailData = $state<TrackDetails | null>(null);

  /** Only explicitly checked rows — never falls back to all tracks. */
  const filenameTargets = $derived(
    trackStore.tracks.filter((t) => trackStore.isSelected(t.path)),
  );

  type PendingConfirm =
    | {
        kind: "filenameToTitle";
        paths: string[];
      }
    | {
        kind: "bulkTags";
        paths: string[];
        album: string | null;
        artist: string | null;
      };

  let pendingConfirm = $state<PendingConfirm | null>(null);
  let confirmBusy = $state(false);

  function snapshotTags(paths: string[]): TagUpdate[] {
    return paths
      .map((path) => trackStore.tracks.find((t) => t.path === path))
      .filter((t): t is TrackInfo => !!t)
      .map((t) => ({
        path: t.path,
        title: t.title,
        album: t.album,
        artist: t.artist,
      }));
  }

  function notifyDone(message: string) {
    toastStore.push("success", message, {
      actionLabel: historyStore.canUndo ? "History" : undefined,
      onAction: historyStore.canUndo
        ? () => {
            historyOpen = true;
          }
        : undefined,
      ms: 4000,
    });
  }

  async function reverseEntry(entry: HistoryEntry): Promise<boolean> {
    if (entry.payload.kind === "tags") {
      const results = await api.writeTags(entry.payload.previous);
      trackStore.applyOpResults(results);
      return results.some((r) => r.ok);
    }
    const reversePlan: RenamePlan = {
      items: entry.payload.pairs.map((p) => ({
        from: p.to,
        to: p.from,
        title: "",
        collision: false,
        skipped: false,
        reason: null,
      })),
      collisionCount: 0,
      applyableCount: entry.payload.pairs.length,
    };
    const results = await api.applyRenames(reversePlan);
    trackStore.applyRenameResults(results);
    return results.some((r) => r.ok);
  }

  async function performUndoLatest() {
    const entry = historyStore.latest;
    if (!entry || trackStore.busy) return;
    trackStore.setBusy(true);
    try {
      const ok = await reverseEntry(entry);
      historyStore.markLatestUndone();
      toastStore.push(
        ok ? "info" : "warning",
        ok ? `Undid: ${entry.label}` : `Could not fully undo: ${entry.label}`,
      );
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  /** Undo target and every newer action (newest first). */
  async function performUndoTo(targetId: number) {
    const chain = historyStore.chainToUndo(targetId);
    if (chain.length === 0 || trackStore.busy) return;
    trackStore.setBusy(true);
    try {
      const undoneIds: number[] = [];
      for (const entry of chain) {
        await reverseEntry(entry);
        undoneIds.push(entry.id);
      }
      historyStore.markUndone(undoneIds);
      toastStore.push(
        "info",
        chain.length === 1
          ? `Undid: ${chain[0].label}`
          : `Undid ${chain.length} actions`,
      );
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  async function loadPaths(paths: string[]) {
    if (paths.length === 0) return;
    trackStore.setBusy(true);
    try {
      const tracks = await api.readTracks(paths);
      trackStore.addTracks(tracks);
      const errors = tracks.filter((t) => t.error).length;
      toastStore.push(
        errors ? "warning" : "success",
        errors
          ? `Loaded ${tracks.length} · ${errors} with read errors`
          : `Loaded ${tracks.length} track${tracks.length === 1 ? "" : "s"}`,
      );
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  async function openFiles() {
    const selected = await open({
      multiple: true,
      filters: [AUDIO_FILTER],
      defaultPath: trackStore.lastDirectory ?? undefined,
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths[0]) {
      const parent = paths[0].replace(/[/\\][^/\\]+$/, "");
      trackStore.setLastDirectory(parent);
    }
    await loadPaths(paths);
  }

  async function openFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: trackStore.lastDirectory ?? undefined,
    });
    if (!selected || Array.isArray(selected)) return;
    trackStore.setLastDirectory(selected);
    trackStore.setBusy(true);
    try {
      const paths = await api.scanFolder(selected, true);
      if (paths.length === 0) {
        toastStore.push("warning", "No audio files found in that folder");
        return;
      }
      await loadPaths(paths);
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  function requireSelection(): string[] | null {
    const paths = trackStore.selectedPaths();
    if (paths.length === 0) {
      toastStore.push(
        "warning",
        "Select one or more tracks first (use the checkboxes)",
      );
      return null;
    }
    return paths;
  }

  /** Confirm when writing tags for 2+ files; single file is undo-only. */
  function requestTagConfirm(pending: PendingConfirm) {
    if (pending.paths.length >= 2) {
      pendingConfirm = pending;
      return;
    }
    void runConfirmed(pending);
  }

  async function runConfirmed(pending: PendingConfirm) {
    if (pending.kind === "filenameToTitle") {
      await doFilenameToTitle(pending.paths);
    } else {
      await doBulkTags(pending.paths, pending.album, pending.artist);
    }
  }

  async function doFilenameToTitle(paths: string[]) {
    const previous = snapshotTags(paths);
    trackStore.setBusy(true);
    try {
      const results = await api.titlesFromFilenames(paths);
      trackStore.applyOpResults(results);
      const ok = results.filter((r) => r.ok).length;
      const fail = results.length - ok;
      if (ok > 0) {
        historyStore.push("Titles from filename", `${ok} track${ok === 1 ? "" : "s"}`, {
          kind: "tags",
          previous,
        });
      }
      if (fail) {
        toastStore.push(
          "warning",
          `Updated ${ok} titles · ${fail} failed`,
          ok
            ? {
                actionLabel: "History",
                onAction: () => {
                  historyOpen = true;
                },
              }
            : undefined,
        );
      } else {
        notifyDone(
          `Set title from filename on ${ok} track${ok === 1 ? "" : "s"}`,
        );
      }
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  function filenameToTitle() {
    const paths = requireSelection();
    if (!paths) return;
    requestTagConfirm({ kind: "filenameToTitle", paths });
  }

  async function titleToFilename() {
    const paths = requireSelection();
    if (!paths) return;
    trackStore.setBusy(true);
    try {
      renamePlan = await api.planRenames(paths);
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  async function applyRename() {
    if (!renamePlan) return;
    applyingRename = true;
    trackStore.setBusy(true);
    try {
      const results = await api.applyRenames(renamePlan);
      trackStore.applyRenameResults(results);
      const okPairs = results
        .filter((r) => r.ok && r.to)
        .map((r) => ({ from: r.from, to: r.to as string }));
      const ok = okPairs.length;
      const fail = results.length - ok;
      if (ok > 0) {
        historyStore.push(
          "Rename files",
          `${ok} file${ok === 1 ? "" : "s"}`,
          { kind: "renames", pairs: okPairs },
        );
      }
      renamePlan = null;
      if (fail && ok === 0) {
        toastStore.push("error", "Rename failed");
      } else {
        notifyDone(
          `Renamed ${ok}${fail ? ` · ${fail} skipped/failed` : ""}`,
        );
      }
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      applyingRename = false;
      trackStore.setBusy(false);
    }
  }

  async function doBulkTags(
    paths: string[],
    album: string | null,
    artist: string | null,
  ) {
    const previous = snapshotTags(paths);
    trackStore.setBusy(true);
    try {
      const results = await api.setFieldBulk(paths, album, artist);
      trackStore.applyOpResults(results);
      const ok = results.filter((r) => r.ok).length;
      const parts = [album ? "album" : null, artist ? "artist" : null].filter(
        Boolean,
      );
      if (ok > 0) {
        historyStore.push(
          `Set ${parts.join(" + ")}`,
          `${ok} track${ok === 1 ? "" : "s"}`,
          { kind: "tags", previous },
        );
      }
      notifyDone(
        `Updated ${parts.join(" + ")} on ${ok} track${ok === 1 ? "" : "s"}`,
      );
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  function applyBulk() {
    const album = albumDraft.trim() || null;
    const artist = artistDraft.trim() || null;
    if (!album && !artist) {
      toastStore.push("warning", "Enter an album and/or artist first");
      return;
    }
    const paths = requireSelection();
    if (!paths) return;
    requestTagConfirm({ kind: "bulkTags", paths, album, artist });
  }

  async function commitCell(
    path: string,
    field: "title" | "album" | "artist",
    value: string,
  ) {
    const current = trackStore.tracks.find((t) => t.path === path);
    if (!current || current[field] === value) return;
    const previous: TagUpdate[] = [
      {
        path,
        title: current.title,
        album: current.album,
        artist: current.artist,
      },
    ];
    trackStore.updateLocal(path, { [field]: value });
    try {
      const results = await api.writeTags([
        {
          path,
          title: field === "title" ? value : null,
          album: field === "album" ? value : null,
          artist: field === "artist" ? value : null,
        },
      ]);
      trackStore.applyOpResults(results);
      if (!results[0]?.ok) {
        toastStore.push("error", results[0]?.error ?? "Failed to save tag");
        trackStore.updateLocal(path, {
          title: previous[0].title ?? "",
          album: previous[0].album ?? "",
          artist: previous[0].artist ?? "",
        });
      } else {
        historyStore.push(`Edit ${field}`, current.filename, {
          kind: "tags",
          previous,
        });
      }
    } catch (e) {
      toastStore.push("error", String(e));
    }
  }

  function stemOfFilename(filename: string): string {
    const i = filename.lastIndexOf(".");
    if (i <= 0) return filename;
    return filename.slice(0, i);
  }

  async function commitFilename(path: string, stem: string) {
    const current = trackStore.tracks.find((t) => t.path === path);
    if (!current) return;
    const currentStem = stemOfFilename(current.filename);
    if (!stem || stem === currentStem) return;

    trackStore.setBusy(true);
    try {
      const result = await api.renameFileStem(path, stem);
      if (result.ok && result.to) {
        trackStore.applyRenameResults([result]);
        historyStore.push("Rename file", result.track?.filename ?? stem, {
          kind: "renames",
          pairs: [{ from: result.from, to: result.to }],
        });
        notifyDone(`Renamed to ${result.track?.filename ?? stem}`);
      } else {
        toastStore.push("error", result.error ?? "Rename failed");
      }
    } catch (e) {
      toastStore.push("error", String(e));
    } finally {
      trackStore.setBusy(false);
    }
  }

  async function openDetail(path: string) {
    detailOpen = true;
    detailLoading = true;
    detailError = null;
    detailData = null;
    try {
      detailData = await api.readTrackDetails(path);
    } catch (e) {
      detailError = String(e);
    } finally {
      detailLoading = false;
    }
  }

  async function handleDropPaths(paths: string[]) {
    const files: string[] = [];
    const folders: string[] = [];
    for (const p of paths) {
      const base = p.split(/[/\\]/).pop() ?? "";
      if (base.includes(".")) files.push(p);
      else folders.push(p);
    }
    if (files.length) await loadPaths(files);
    for (const folder of folders) {
      try {
        const scanned = await api.scanFolder(folder, true);
        await loadPaths(scanned);
      } catch {
        await loadPaths([folder]);
      }
    }
  }

  async function onConfirmDialog() {
    if (!pendingConfirm) return;
    const pending = pendingConfirm;
    confirmBusy = true;
    try {
      await runConfirmed(pending);
      pendingConfirm = null;
    } finally {
      confirmBusy = false;
    }
  }

  const confirmCopy = $derived.by(() => {
    if (!pendingConfirm) return null;
    const n = pendingConfirm.paths.length;
    const tracks = `${n} selected track${n === 1 ? "" : "s"}`;
    if (pendingConfirm.kind === "filenameToTitle") {
      return {
        title: "Update titles from filenames?",
        body: `This will overwrite the title tag on ${tracks} using each file’s name. Files themselves are not renamed.`,
        detail: "You can undo this afterward with Undo or ⌘Z.",
        confirmLabel: `Update ${n} title${n === 1 ? "" : "s"}`,
        danger: false,
      };
    }
    const fields = [
      pendingConfirm.album ? `album “${pendingConfirm.album}”` : null,
      pendingConfirm.artist ? `artist “${pendingConfirm.artist}”` : null,
    ]
      .filter(Boolean)
      .join(" and ");
    return {
      title: "Apply bulk tags?",
      body: `Write ${fields} to ${tracks} on disk.`,
      detail: "You can undo this afterward with Undo or ⌘Z.",
      confirmLabel: `Apply to ${n}`,
      danger: false,
    };
  });

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;
      const t = e.target as HTMLElement | null;
      const inField =
        t &&
        (t.tagName === "INPUT" ||
          t.tagName === "TEXTAREA" ||
          t.isContentEditable);

      if (meta && e.key.toLowerCase() === "z" && !e.shiftKey && !inField) {
        if (historyStore.canUndo) {
          e.preventDefault();
          void performUndoLatest();
        }
        return;
      }
      if (meta && e.shiftKey && e.key.toLowerCase() === "z") {
        if (historyStore.totalCount > 0) {
          e.preventDefault();
          historyOpen = true;
        }
      }
      if (meta && e.key.toLowerCase() === "o" && !e.shiftKey) {
        e.preventDefault();
        openFiles();
      }
      if (meta && e.shiftKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        openFolder();
      }
      if (meta && e.key.toLowerCase() === "a" && trackStore.tracks.length) {
        if (inField) return;
        e.preventDefault();
        trackStore.selectAll();
      }
      if (e.key === "Escape" && pendingConfirm) {
        pendingConfirm = null;
      }
    };
    window.addEventListener("keydown", onKey);

    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        unlisten = await getCurrentWindow().onDragDropEvent((event) => {
          const { type } = event.payload;
          if (type === "over" || type === "enter") {
            dragOver = true;
          } else if (type === "leave") {
            dragOver = false;
          } else if (type === "drop") {
            dragOver = false;
            const paths = event.payload.paths ?? [];
            void handleDropPaths(paths);
          }
        });
      } catch {
        // browser preview without Tauri
      }
    })();

    return () => {
      window.removeEventListener("keydown", onKey);
      unlisten?.();
    };
  });
</script>

<div
  class="flex h-screen flex-col overflow-hidden bg-surface-950 text-surface-100 {dragOver
    ? 'ring-2 ring-inset ring-accent-500'
    : ''}"
>
  <Toolbar
    trackCount={trackStore.tracks.length}
    selectedCount={trackStore.selected.size}
    busy={trackStore.busy}
    {albumDraft}
    {artistDraft}
    {activeTab}
    onTabChange={(tab) => (activeTab = tab)}
    onOpenFiles={openFiles}
    onOpenFolder={openFolder}
    onFilenameToTitle={filenameToTitle}
    onTitleToFilename={titleToFilename}
    onApplyBulk={applyBulk}
    onClear={() => trackStore.clear()}
    onRemoveSelected={() => trackStore.removeSelected()}
    onAlbumDraft={(v) => (albumDraft = v)}
    onArtistDraft={(v) => (artistDraft = v)}
  />

  {#if activeTab === "filenames" && trackStore.tracks.length > 0}
    <FilenameTools
      tracks={filenameTargets}
      selectedCount={trackStore.selected.size}
      busy={trackStore.busy}
      onPreview={(plan) => {
        if (filenameTargets.length === 0) {
          toastStore.push(
            "warning",
            "Select one or more tracks first (use the checkboxes)",
          );
          return;
        }
        if (plan.applyableCount === 0 && plan.items.length > 0) {
          toastStore.push(
            "warning",
            "Nothing to rename — check options or collisions",
          );
        }
        renamePlan = plan;
      }}
    />
  {/if}

  {#if trackStore.tracks.length === 0}
    <EmptyState onOpenFiles={openFiles} onOpenFolder={openFolder} />
  {:else}
    <TrackTable
      tracks={trackStore.tracks}
      onCommitTag={commitCell}
      onCommitFilename={commitFilename}
      onOpenDetail={openDetail}
    />
  {/if}

  <StatusBar
    trackCount={trackStore.tracks.length}
    selectedCount={trackStore.selected.size}
    errorCount={trackStore.errorCount()}
    busy={trackStore.busy}
    canUndo={historyStore.canUndo}
    historyCount={historyStore.totalCount}
    undoLabel={historyStore.latest?.label ?? null}
    onUndo={() => void performUndoLatest()}
    onOpenHistory={() => (historyOpen = true)}
  />

  <ToastStack />

  {#if historyOpen}
    <HistoryPanel
      entries={historyStore.entries}
      busy={trackStore.busy}
      onClose={() => (historyOpen = false)}
      onUndoLatest={() => void performUndoLatest()}
      onUndoTo={(id) => void performUndoTo(id)}
      onClear={() => historyStore.clear()}
    />
  {/if}

  {#if detailOpen}
    <TrackDetailPanel
      details={detailData}
      loading={detailLoading}
      error={detailError}
      onClose={() => {
        detailOpen = false;
        detailData = null;
        detailError = null;
      }}
    />
  {/if}

  {#if renamePlan}
    <RenamePreview
      plan={renamePlan}
      applying={applyingRename}
      onCancel={() => (renamePlan = null)}
      onApply={applyRename}
    />
  {/if}

  {#if pendingConfirm && confirmCopy}
    <ConfirmDialog
      title={confirmCopy.title}
      body={confirmCopy.body}
      detail={confirmCopy.detail}
      confirmLabel={confirmCopy.confirmLabel}
      danger={confirmCopy.danger}
      busy={confirmBusy}
      onCancel={() => (pendingConfirm = null)}
      onConfirm={() => void onConfirmDialog()}
    />
  {/if}

  {#if dragOver}
    <div
      class="pointer-events-none absolute inset-0 z-30 flex items-center justify-center bg-accent-600/10"
    >
      <div
        class="rounded-xl border border-dashed border-accent-400 bg-surface-900/90 px-8 py-6 text-center shadow-xl"
      >
        <p class="text-sm font-medium text-accent-400">
          Drop audio files or folders
        </p>
      </div>
    </div>
  {/if}
</div>
