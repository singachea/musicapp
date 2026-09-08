<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { TrackDetails } from "../types";

  interface Props {
    details: TrackDetails | null;
    loading: boolean;
    error?: string | null;
    onClose: () => void;
  }

  let { details, loading, error = null, onClose }: Props = $props();

  let audioEl = $state<HTMLAudioElement | null>(null);
  let playing = $state(false);
  let playError = $state<string | null>(null);
  let currentTime = $state(0);
  let duration = $state(0);
  let scrubbing = $state(false);
  let scrubValue = $state(0);

  const audioSrc = $derived.by(() => {
    if (!details?.path) return null;
    try {
      return convertFileSrc(details.path);
    } catch {
      return null;
    }
  });

  const displayTime = $derived(scrubbing ? scrubValue : currentTime);
  const displayDuration = $derived(
    duration > 0
      ? duration
      : details?.durationSecs && details.durationSecs > 0
        ? details.durationSecs
        : 0,
  );
  const progressPct = $derived(
    displayDuration > 0
      ? Math.min(100, Math.max(0, (displayTime / displayDuration) * 100))
      : 0,
  );

  function formatTime(secs: number): string {
    if (!Number.isFinite(secs) || secs < 0) return "0:00";
    const total = Math.floor(secs);
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const s = total % 60;
    if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    return `${m}:${String(s).padStart(2, "0")}`;
  }

  function row(label: string, value: string | number | null | undefined) {
    if (value === null || value === undefined || value === "") return null;
    return { label, value: String(value) };
  }

  const metaRows = $derived.by(() => {
    if (!details) return [];
    const track =
      details.trackNumber != null
        ? details.trackTotal != null
          ? `${details.trackNumber} / ${details.trackTotal}`
          : String(details.trackNumber)
        : null;
    const disc =
      details.discNumber != null
        ? details.discTotal != null
          ? `${details.discNumber} / ${details.discTotal}`
          : String(details.discNumber)
        : null;
    const audioBits = [
      details.bitrateKbps != null ? `${details.bitrateKbps} kbps` : null,
      details.sampleRateHz != null
        ? `${(details.sampleRateHz / 1000).toFixed(1)} kHz`
        : null,
      details.channels != null
        ? details.channels === 1
          ? "Mono"
          : details.channels === 2
            ? "Stereo"
            : `${details.channels} ch`
        : null,
      details.bitDepth != null ? `${details.bitDepth}-bit` : null,
    ]
      .filter(Boolean)
      .join(" · ");

    return [
      row("Title", details.title),
      row("Artist", details.artist),
      row("Album artist", details.albumArtist),
      row("Album", details.album),
      row("Genre", details.genre),
      row("Year", details.year),
      row("Date", details.date !== details.year ? details.date : null),
      row("Track", track),
      row("Disc", disc),
      row("Comment", details.comment),
      row("Duration", details.durationLabel),
      row("Format", details.format?.toUpperCase()),
      row("Audio", audioBits || null),
      row("File size", details.fileSizeLabel),
      row("Path", details.path),
    ].filter(Boolean) as { label: string; value: string }[];
  });

  async function togglePlay() {
    playError = null;
    if (!audioEl) return;
    try {
      if (playing) {
        audioEl.pause();
        playing = false;
      } else {
        await audioEl.play();
        playing = true;
      }
    } catch (e) {
      playError = String(e);
      playing = false;
    }
  }

  function onTimeUpdate() {
    if (!audioEl || scrubbing) return;
    currentTime = audioEl.currentTime;
  }

  function onLoadedMetadata() {
    if (!audioEl) return;
    if (Number.isFinite(audioEl.duration) && audioEl.duration > 0) {
      duration = audioEl.duration;
    }
  }

  function onAudioEnded() {
    playing = false;
    currentTime = 0;
  }

  function startScrub(e: Event) {
    scrubbing = true;
    const t = e.currentTarget as HTMLInputElement;
    scrubValue = Number(t.value);
  }

  function onScrubInput(e: Event) {
    const t = e.currentTarget as HTMLInputElement;
    scrubValue = Number(t.value);
  }

  function endScrub(e: Event) {
    const t = e.currentTarget as HTMLInputElement;
    const value = Number(t.value);
    scrubbing = false;
    currentTime = value;
    if (audioEl && Number.isFinite(value)) {
      audioEl.currentTime = value;
    }
  }

  function seekRelative(delta: number) {
    if (!audioEl) return;
    const max = displayDuration || audioEl.duration || 0;
    const next = Math.min(max, Math.max(0, (audioEl.currentTime || 0) + delta));
    audioEl.currentTime = next;
    currentTime = next;
  }

  // Reset + stop when track changes or panel closes
  $effect(() => {
    const path = details?.path;
    currentTime = 0;
    duration = details?.durationSecs && details.durationSecs > 0
      ? details.durationSecs
      : 0;
    scrubbing = false;
    playError = null;
    playing = false;
    return () => {
      if (audioEl) {
        audioEl.pause();
      }
      void path;
    };
  });
</script>

<div
  class="fixed inset-0 z-40 flex justify-end bg-black/50 backdrop-blur-sm"
  role="dialog"
  aria-modal="true"
  aria-labelledby="detail-title"
>
  <button
    type="button"
    class="min-w-0 flex-1 cursor-default"
    aria-label="Close details"
    onclick={onClose}
  ></button>

  <aside
    class="flex h-full w-full max-w-md flex-col border-l border-surface-700 bg-surface-900 shadow-2xl"
  >
    <header
      class="flex items-start justify-between gap-3 border-b border-surface-800 px-4 py-3"
    >
      <div class="min-w-0">
        <h2
          id="detail-title"
          class="truncate text-sm font-semibold text-surface-100"
        >
          {details?.title || details?.filename || "Track details"}
        </h2>
        <p class="mt-0.5 truncate text-[11px] text-surface-400">
          {details?.artist || details?.filename || ""}
        </p>
      </div>
      <button
        type="button"
        class="shrink-0 rounded-md px-2 py-1 text-xs text-surface-400 hover:bg-surface-800 hover:text-surface-100"
        onclick={onClose}
      >
        Close
      </button>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
      {#if loading}
        <p class="text-sm text-surface-400">Loading metadata…</p>
      {:else if error}
        <p class="text-sm text-danger-400">{error}</p>
      {:else if details}
        {#if details.error}
          <p
            class="mb-3 rounded-lg bg-danger-400/10 px-3 py-2 text-xs text-danger-400"
          >
            {details.error}
          </p>
        {/if}

        <div class="mb-4 flex gap-4">
          <div
            class="flex h-28 w-28 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-surface-850 ring-1 ring-surface-700"
          >
            {#if details.coverDataUrl}
              <img
                src={details.coverDataUrl}
                alt="Cover art"
                class="h-full w-full object-cover"
              />
            {:else}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="currentColor"
                class="h-10 w-10 text-surface-600"
              >
                <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6z" />
              </svg>
            {/if}
          </div>
          <div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
            <p class="truncate text-sm font-medium text-surface-100">
              {details.title || details.filename}
            </p>
            <p class="truncate text-xs text-surface-400">
              {details.artist || "Unknown artist"}
              {#if details.format}
                · {details.format.toUpperCase()}
              {/if}
            </p>
          </div>
        </div>

        {#if audioSrc}
          <div
            class="mb-5 rounded-xl border border-surface-800 bg-surface-850/60 p-3"
          >
            <div class="mb-2 flex items-center gap-2">
              <button
                type="button"
                class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-accent-600 text-white hover:bg-accent-500"
                onclick={togglePlay}
                aria-label={playing ? "Pause" : "Play"}
              >
                {#if playing}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    class="h-4 w-4"
                  >
                    <path
                      d="M5.75 3A1.75 1.75 0 0 0 4 4.75v10.5c0 .966.784 1.75 1.75 1.75h1.5A1.75 1.75 0 0 0 9 15.25V4.75A1.75 1.75 0 0 0 7.25 3h-1.5ZM12.75 3A1.75 1.75 0 0 0 11 4.75v10.5c0 .966.784 1.75 1.75 1.75h1.5A1.75 1.75 0 0 0 16 15.25V4.75A1.75 1.75 0 0 0 14.25 3h-1.5Z"
                    />
                  </svg>
                {:else}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                    class="ml-0.5 h-4 w-4"
                  >
                    <path
                      d="M6.3 2.84A1.5 1.5 0 0 0 4 4.11v11.78a1.5 1.5 0 0 0 2.3 1.27l9.344-5.891a1.5 1.5 0 0 0 0-2.538L6.3 2.841Z"
                    />
                  </svg>
                {/if}
              </button>

              <button
                type="button"
                class="rounded-md px-2 py-1 text-[11px] text-surface-400 hover:bg-surface-800 hover:text-surface-200"
                onclick={() => seekRelative(-10)}
                title="Back 10s"
              >
                −10s
              </button>
              <button
                type="button"
                class="rounded-md px-2 py-1 text-[11px] text-surface-400 hover:bg-surface-800 hover:text-surface-200"
                onclick={() => seekRelative(10)}
                title="Forward 10s"
              >
                +10s
              </button>

              <span class="ml-auto font-mono text-[11px] tabular-nums text-surface-300">
                {formatTime(displayTime)}
                <span class="text-surface-600">/</span>
                {formatTime(displayDuration)}
              </span>
            </div>

            <!-- Seek bar -->
            <div class="relative pt-1">
              <div
                class="pointer-events-none absolute left-0 right-0 top-[11px] h-1.5 overflow-hidden rounded-full bg-surface-700"
              >
                <div
                  class="h-full rounded-full bg-accent-500"
                  style="width: {progressPct}%"
                ></div>
              </div>
              <input
                type="range"
                class="seek-range relative z-10 w-full"
                min="0"
                max={displayDuration || 0}
                step="0.1"
                value={displayTime}
                disabled={displayDuration <= 0}
                aria-label="Seek"
                onpointerdown={startScrub}
                oninput={onScrubInput}
                onpointerup={endScrub}
                onchange={endScrub}
              />
            </div>

            <audio
              bind:this={audioEl}
              src={audioSrc}
              preload="metadata"
              ontimeupdate={onTimeUpdate}
              onloadedmetadata={onLoadedMetadata}
              ondurationchange={onLoadedMetadata}
              onended={onAudioEnded}
              onpause={() => (playing = false)}
              onplay={() => (playing = true)}
            ></audio>

            {#if playError}
              <p class="mt-2 text-[11px] text-danger-400">{playError}</p>
            {/if}
          </div>
        {:else}
          <p class="mb-4 text-[11px] text-surface-500">
            Playback unavailable for this path
          </p>
        {/if}

        <dl class="space-y-2">
          {#each metaRows as r}
            <div class="grid grid-cols-[6.5rem_1fr] gap-2 text-sm">
              <dt
                class="text-[11px] font-medium uppercase tracking-wide text-surface-500"
              >
                {r.label}
              </dt>
              <dd
                class="break-all text-surface-200 {r.label === 'Path'
                  ? 'font-mono text-xs text-surface-400'
                  : ''}"
              >
                {r.value}
              </dd>
            </div>
          {/each}
        </dl>
      {/if}
    </div>
  </aside>
</div>

<style>
  .seek-range {
    -webkit-appearance: none;
    appearance: none;
    height: 1.25rem;
    background: transparent;
    cursor: pointer;
  }
  .seek-range:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  .seek-range::-webkit-slider-runnable-track {
    height: 6px;
    background: transparent;
    border-radius: 999px;
  }
  .seek-range::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    margin-top: -4px;
    border-radius: 999px;
    background: var(--color-accent-400);
    box-shadow: 0 0 0 2px var(--color-surface-900);
    border: none;
  }
  .seek-range::-moz-range-track {
    height: 6px;
    background: transparent;
    border-radius: 999px;
  }
  .seek-range::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--color-accent-400);
    border: 2px solid var(--color-surface-900);
  }
</style>
