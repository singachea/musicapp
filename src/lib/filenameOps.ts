import type { RenameItem, RenamePlan, TrackInfo } from "./types";

export type FilenameToolId =
  | "prefix"
  | "suffix"
  | "replace"
  | "remove"
  | "case"
  | "spaces"
  | "number";

export type CaseMode = "lower" | "upper" | "title";

export interface FilenameToolState {
  prefix: string;
  suffix: string;
  find: string;
  replaceWith: string;
  useRegex: boolean;
  removePattern: string;
  removeRegex: boolean;
  caseMode: CaseMode;
  numberStart: number;
  numberPad: number;
  numberSeparator: string;
}

export const defaultToolState = (): FilenameToolState => ({
  prefix: "",
  suffix: "",
  find: "",
  replaceWith: "",
  useRegex: false,
  removePattern: "",
  removeRegex: false,
  caseMode: "title",
  numberStart: 1,
  numberPad: 2,
  numberSeparator: " - ",
});

export function stemOf(filename: string): string {
  const i = filename.lastIndexOf(".");
  if (i <= 0) return filename;
  return filename.slice(0, i);
}

export function extOf(filename: string): string {
  const i = filename.lastIndexOf(".");
  if (i <= 0) return "";
  return filename.slice(i);
}

/** Illegal filesystem chars — mirrors Rust sanitize. */
export function sanitizeStem(name: string): string {
  let out = "";
  for (const ch of name) {
    if ('/\\:*?"<>|\0'.includes(ch) || ch.charCodeAt(0) < 32) {
      out += "_";
    } else {
      out += ch;
    }
  }
  const trimmed = out.trim().replace(/\.+$/g, "");
  return trimmed.length ? trimmed : "untitled";
}

function titleCase(s: string): string {
  return s
    .toLowerCase()
    .split(/(\s+)/)
    .map((part) =>
      /^\s+$/.test(part)
        ? part
        : part
            .split(/([-_])/)
            .map((p) =>
              p === "-" || p === "_"
                ? p
                : p
                  ? p[0].toUpperCase() + p.slice(1)
                  : p,
            )
            .join(""),
    )
    .join("");
}

function applyCase(stem: string, mode: CaseMode): string {
  switch (mode) {
    case "lower":
      return stem.toLowerCase();
    case "upper":
      return stem.toUpperCase();
    case "title":
      return titleCase(stem);
  }
}

function replaceAll(
  stem: string,
  find: string,
  replacement: string,
  useRegex: boolean,
): { ok: true; value: string } | { ok: false; error: string } {
  if (!find) return { ok: true, value: stem };
  try {
    if (useRegex) {
      const re = new RegExp(find, "g");
      return { ok: true, value: stem.replace(re, replacement) };
    }
    // literal global replace
    return {
      ok: true,
      value: stem.split(find).join(replacement),
    };
  } catch (e) {
    return { ok: false, error: `Invalid regex: ${e}` };
  }
}

export type TransformFn = (
  stem: string,
  index: number,
) => { ok: true; value: string } | { ok: false; error: string };

export function transformForTool(
  tool: FilenameToolId,
  state: FilenameToolState,
): TransformFn | null {
  switch (tool) {
    case "prefix": {
      const p = state.prefix;
      if (!p) return null;
      return (stem) => ({ ok: true, value: p + stem });
    }
    case "suffix": {
      const s = state.suffix;
      if (!s) return null;
      return (stem) => ({ ok: true, value: stem + s });
    }
    case "replace": {
      if (!state.find) return null;
      return (stem) =>
        replaceAll(stem, state.find, state.replaceWith, state.useRegex);
    }
    case "remove": {
      if (!state.removePattern) return null;
      return (stem) =>
        replaceAll(stem, state.removePattern, "", state.removeRegex);
    }
    case "case":
      return (stem) => ({
        ok: true,
        value: applyCase(stem, state.caseMode),
      });
    case "spaces":
      return (stem) => ({
        ok: true,
        value: stem.trim().replace(/\s+/g, " "),
      });
    case "number": {
      const start = Number.isFinite(state.numberStart) ? state.numberStart : 1;
      const pad = Math.min(8, Math.max(1, state.numberPad || 2));
      const sep = state.numberSeparator;
      return (stem, index) => {
        const n = String(start + index).padStart(pad, "0");
        return { ok: true, value: `${n}${sep}${stem}` };
      };
    }
  }
}

export function buildRenamePlan(
  tracks: TrackInfo[],
  transform: TransformFn,
): RenamePlan {
  type Draft = {
    from: string;
    to: string;
    title: string;
    reason: string | null;
  };

  const drafts: Draft[] = [];
  const targetCounts = new Map<string, number>();

  tracks.forEach((track, index) => {
    const stem = stemOf(track.filename);
    const ext = extOf(track.filename);
    const result = transform(stem, index);
    if (!result.ok) {
      drafts.push({
        from: track.path,
        to: "",
        title: stem,
        reason: result.error,
      });
      return;
    }
    const safe = sanitizeStem(result.value);
    const parent = track.path.replace(/[/\\][^/\\]+$/, "");
    const sep = track.path.includes("\\") ? "\\" : "/";
    const to = parent + sep + safe + ext;

    if (to === track.path) {
      drafts.push({
        from: track.path,
        to,
        title: safe,
        reason: "Already named correctly",
      });
      return;
    }

    targetCounts.set(to, (targetCounts.get(to) ?? 0) + 1);
    drafts.push({ from: track.path, to, title: safe, reason: null });
  });

  let collisionCount = 0;
  let applyableCount = 0;
  const items: RenameItem[] = drafts.map((d) => {
    if (d.reason) {
      return {
        from: d.from,
        to: d.to,
        title: d.title,
        collision: false,
        skipped: true,
        reason: d.reason,
      };
    }
    const multi = (targetCounts.get(d.to) ?? 0) > 1;
    // Disk collision: another file exists and is not one of our sources that will leave that path
    // Lightweight check: if destination path equals another track's current path that isn't renaming away, or unknown existence.
    // We flag multi-target always; disk check is best-effort in UI (backend enforces on apply).
    const collision = multi;
    if (collision) {
      collisionCount += 1;
      return {
        from: d.from,
        to: d.to,
        title: d.title,
        collision: true,
        skipped: true,
        reason: "Multiple files would get the same name",
      };
    }
    applyableCount += 1;
    return {
      from: d.from,
      to: d.to,
      title: d.title,
      collision: false,
      skipped: false,
      reason: null,
    };
  });

  // Second pass: destinations that collide with an existing path that is not being renamed away
  const sources = new Set(tracks.map((t) => t.path));
  const leaving = new Set(
    items.filter((i) => !i.skipped && i.to).map((i) => i.from),
  );

  for (const item of items) {
    if (item.skipped) continue;
    // If another listed track currently occupies the dest and is not leaving, conflict
    const occupant = tracks.find((t) => t.path === item.to);
    if (occupant && !leaving.has(occupant.path)) {
      item.collision = true;
      item.skipped = true;
      item.reason = "Destination already exists in list";
      collisionCount += 1;
      applyableCount = Math.max(0, applyableCount - 1);
    } else if (!sources.has(item.to) && leaving.has(item.from)) {
      // can't know disk without backend — leave applyable; apply_renames checks disk
    }
  }

  return { items, collisionCount, applyableCount };
}

export const TOOL_META: Record<
  FilenameToolId,
  { label: string; blurb: string }
> = {
  prefix: {
    label: "Add prefix",
    blurb: "Insert text at the start of each name",
  },
  suffix: {
    label: "Add suffix",
    blurb: "Append text before the extension",
  },
  replace: {
    label: "Find & replace",
    blurb: "Swap text or a regex across names",
  },
  remove: {
    label: "Remove text",
    blurb: "Delete a phrase or regex match",
  },
  case: {
    label: "Change case",
    blurb: "lower, UPPER, or Title Case",
  },
  spaces: {
    label: "Clean spaces",
    blurb: "Trim edges and collapse repeated spaces",
  },
  number: {
    label: "Number files",
    blurb: "Add 01, 02… in selection order",
  },
};
