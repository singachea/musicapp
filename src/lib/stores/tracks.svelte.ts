import type { OpResult, RenameApplyResult, TrackInfo } from "../types";

let tracks = $state<TrackInfo[]>([]);
let selected = $state<Set<string>>(new Set());
let busy = $state(false);
let lastDirectory = $state<string | null>(null);

export const trackStore = {
  get tracks() {
    return tracks;
  },
  get selected() {
    return selected;
  },
  get busy() {
    return busy;
  },
  get lastDirectory() {
    return lastDirectory;
  },
  setBusy(v: boolean) {
    busy = v;
  },
  setLastDirectory(dir: string | null) {
    lastDirectory = dir;
  },
  clear() {
    tracks = [];
    selected = new Set();
  },
  removeSelected() {
    if (selected.size === 0) return;
    tracks = tracks.filter((t) => !selected.has(t.path));
    selected = new Set();
  },
  selectAll() {
    selected = new Set(tracks.map((t) => t.path));
  },
  selectNone() {
    selected = new Set();
  },
  toggle(path: string) {
    const next = new Set(selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    selected = next;
  },
  setSelected(path: string, on: boolean) {
    const next = new Set(selected);
    if (on) next.add(path);
    else next.delete(path);
    selected = next;
  },
  isSelected(path: string) {
    return selected.has(path);
  },
  /** Only explicitly checked rows — never falls back to “all”. */
  selectedPaths(): string[] {
    return tracks.filter((t) => selected.has(t.path)).map((t) => t.path);
  },
  addTracks(incoming: TrackInfo[]) {
    const byPath = new Map(tracks.map((t) => [t.path, t]));
    for (const t of incoming) {
      byPath.set(t.path, t);
    }
    tracks = Array.from(byPath.values()).sort((a, b) =>
      a.path.localeCompare(b.path),
    );
  },
  applyOpResults(results: OpResult[]) {
    const byPath = new Map(tracks.map((t) => [t.path, t]));
    for (const r of results) {
      if (r.ok && r.track) {
        byPath.set(r.track.path, r.track);
      } else if (!r.ok) {
        const existing = byPath.get(r.path);
        if (existing) {
          byPath.set(r.path, { ...existing, error: r.error ?? "Write failed" });
        }
      }
    }
    tracks = Array.from(byPath.values()).sort((a, b) =>
      a.path.localeCompare(b.path),
    );
  },
  applyRenameResults(results: RenameApplyResult[]) {
    const byPath = new Map(tracks.map((t) => [t.path, t]));
    const nextSelected = new Set(selected);
    for (const r of results) {
      if (r.ok && r.track) {
        byPath.delete(r.from);
        byPath.set(r.track.path, r.track);
        if (nextSelected.has(r.from)) {
          nextSelected.delete(r.from);
          nextSelected.add(r.track.path);
        }
      } else if (!r.ok) {
        const existing = byPath.get(r.from);
        if (existing) {
          byPath.set(r.from, {
            ...existing,
            error: r.error ?? "Rename failed",
          });
        }
      }
    }
    tracks = Array.from(byPath.values()).sort((a, b) =>
      a.path.localeCompare(b.path),
    );
    selected = nextSelected;
  },
  updateLocal(path: string, patch: Partial<TrackInfo>) {
    tracks = tracks.map((t) => (t.path === path ? { ...t, ...patch } : t));
  },
  errorCount() {
    return tracks.filter((t) => t.error).length;
  },
};
