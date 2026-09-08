export type AppTab = "tags" | "filenames";

export interface TrackInfo {
  path: string;
  filename: string;
  title: string;
  album: string;
  artist: string;
  format: string;
  error?: string | null;
}

export interface TrackDetails {
  path: string;
  filename: string;
  title: string;
  album: string;
  artist: string;
  albumArtist: string;
  genre: string;
  comment: string;
  year: string;
  date: string;
  trackNumber?: number | null;
  trackTotal?: number | null;
  discNumber?: number | null;
  discTotal?: number | null;
  format: string;
  durationSecs: number;
  durationLabel: string;
  bitrateKbps?: number | null;
  sampleRateHz?: number | null;
  channels?: number | null;
  bitDepth?: number | null;
  fileSizeBytes: number;
  fileSizeLabel: string;
  coverDataUrl?: string | null;
  coverType?: string | null;
  error?: string | null;
}

export interface TagUpdate {
  path: string;
  title?: string | null;
  album?: string | null;
  artist?: string | null;
}

export interface OpResult {
  path: string;
  ok: boolean;
  error?: string | null;
  track?: TrackInfo | null;
}

export interface RenameItem {
  from: string;
  to: string;
  title: string;
  collision: boolean;
  skipped: boolean;
  reason?: string | null;
}

export interface RenamePlan {
  items: RenameItem[];
  collisionCount: number;
  applyableCount: number;
}

export interface RenameApplyResult {
  from: string;
  to?: string | null;
  ok: boolean;
  error?: string | null;
  track?: TrackInfo | null;
}

export interface Toast {
  id: number;
  kind: "info" | "success" | "error" | "warning";
  message: string;
  actionLabel?: string;
  onAction?: () => void;
}

/** Payload needed to reverse a completed disk operation. */
export type HistoryPayload =
  | {
      kind: "tags";
      /** Full previous field values to write back */
      previous: TagUpdate[];
    }
  | {
      kind: "renames";
      /** Successful renames as from→to; undo renames to→from */
      pairs: { from: string; to: string }[];
    };

export interface HistoryEntry {
  id: number;
  /** epoch ms */
  at: number;
  /** Short name shown in UI */
  label: string;
  /** e.g. "3 tracks" */
  detail: string;
  payload: HistoryPayload;
  /** True after this step was undone (linear stack) */
  undone: boolean;
}
