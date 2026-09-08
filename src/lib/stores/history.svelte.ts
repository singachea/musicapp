import type { HistoryEntry, HistoryPayload } from "../types";

const MAX_ENTRIES = 100;

let entries = $state<HistoryEntry[]>([]);
let seq = 0;

function activeEntries(): HistoryEntry[] {
  return entries.filter((e) => !e.undone);
}

export const historyStore = {
  get entries() {
    return entries;
  },
  /** Newest active (not undone) action, if any */
  get latest() {
    const active = activeEntries();
    return active.length ? active[active.length - 1] : null;
  },
  get canUndo() {
    return activeEntries().length > 0;
  },
  get activeCount() {
    return activeEntries().length;
  },
  get totalCount() {
    return entries.length;
  },
  /**
   * Record a successful disk change. New actions after an undo
   * drop any undone tail (classic linear history).
   */
  push(label: string, detail: string, payload: HistoryPayload) {
    // Drop undone entries after a branch (like a redo stack clear)
    entries = entries.filter((e) => !e.undone);
    const entry: HistoryEntry = {
      id: ++seq,
      at: Date.now(),
      label,
      detail,
      payload,
      undone: false,
    };
    entries = [...entries, entry].slice(-MAX_ENTRIES);
  },
  /** Mark the latest active entry as undone (caller must reverse on disk). */
  markLatestUndone(): HistoryEntry | null {
    const latest = historyStore.latest;
    if (!latest) return null;
    entries = entries.map((e) =>
      e.id === latest.id ? { ...e, undone: true } : e,
    );
    return latest;
  },
  /**
   * Entries that must be undone to reach (and undo) targetId,
   * from newest active down to target, inclusive. Empty if invalid.
   */
  chainToUndo(targetId: number): HistoryEntry[] {
    const active = activeEntries();
    const idx = active.findIndex((e) => e.id === targetId);
    if (idx < 0) return [];
    // From newest to target
    return active.slice(idx).reverse();
  },
  markUndone(ids: number[]) {
    const set = new Set(ids);
    entries = entries.map((e) =>
      set.has(e.id) ? { ...e, undone: true } : e,
    );
  },
  clear() {
    entries = [];
  },
};
