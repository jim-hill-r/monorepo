// Closed captions for a demo, built from the narration script itself, so no
// speech recognition is needed: each narration line is spoken from a known
// time for a known length, and its text is the caption.

/** One spoken narration line, placed on the video's timeline. */
export interface Line {
  /** Seconds from the start of the video when the line starts playing. */
  start: number;
  /** Seconds the synthesized audio lasts. */
  duration: number;
  text: string;
}

/** One caption on screen. */
export interface Cue {
  start: number;
  end: number;
  text: string;
}

/** Longest caption, in characters, before a line is split into several cues. */
export const MAX_CUE_CHARS = 84;
/** Longest caption row; longer cues wrap onto a second row. */
export const MAX_ROW_CHARS = 42;

/**
 * Splits narration into caption-sized chunks: first at sentence ends, then a
 * long sentence at commas or semicolons, then at word boundaries.
 */
export function chunk(text: string, max = MAX_CUE_CHARS): string[] {
  const clean = text.replace(/\s+/g, " ").trim();
  if (clean === "") return [];
  const sentences = clean.match(/[^.!?]+[.!?]+["')\]]*|[^.!?]+$/g) ?? [clean];
  const out: string[] = [];
  for (const raw of sentences) {
    const sentence = raw.trim();
    if (sentence.length <= max) {
      out.push(sentence);
      continue;
    }
    for (const clause of splitClauses(sentence)) {
      out.push(...splitWords(clause, max));
    }
  }
  return mergeShort(out, max);
}

function splitClauses(sentence: string): string[] {
  return (sentence.match(/[^,;:]+[,;:]?/g) ?? [sentence])
    .map((s) => s.trim())
    .filter((s) => s !== "");
}

function splitWords(text: string, max: number): string[] {
  const out: string[] = [];
  let current = "";
  for (const word of text.split(" ")) {
    const next = current === "" ? word : `${current} ${word}`;
    if (next.length > max && current !== "") {
      out.push(current);
      current = word;
    } else {
      current = next;
    }
  }
  if (current !== "") out.push(current);
  return out;
}

// Joins neighbouring chunks that fit together, so a split at every comma
// doesn't leave captions of two words.
function mergeShort(chunks: string[], max: number): string[] {
  const out: string[] = [];
  for (const c of chunks) {
    const last = out[out.length - 1];
    if (last !== undefined && `${last} ${c}`.length <= max && !/[.!?]["')\]]*$/.test(last)) {
      out[out.length - 1] = `${last} ${c}`;
    } else {
      out.push(c);
    }
  }
  return out;
}

/** Wraps a caption onto at most two balanced rows. */
export function wrap(text: string, row = MAX_ROW_CHARS): string {
  if (text.length <= row) return text;
  const words = text.split(" ");
  let best = text;
  let bestScore = Number.POSITIVE_INFINITY;
  for (let i = 1; i < words.length; i++) {
    const a = words.slice(0, i).join(" ");
    const b = words.slice(i).join(" ");
    const score = Math.abs(a.length - b.length);
    if (score < bestScore) {
      bestScore = score;
      best = `${a}\n${b}`;
    }
  }
  return best;
}

/**
 * Turns timed narration lines into caption cues. A line split into several
 * chunks shares its time between them by length, which tracks speech closely
 * enough for captions.
 */
export function cues(lines: Line[]): Cue[] {
  const out: Cue[] = [];
  for (const line of lines) {
    const chunks = chunk(line.text);
    const total = chunks.reduce((n, c) => n + c.length, 0);
    let t = line.start;
    for (const c of chunks) {
      const len = total === 0 ? 0 : (line.duration * c.length) / total;
      out.push({ start: t, end: t + len, text: wrap(c) });
      t += len;
    }
  }
  return out;
}

/** Formats seconds as a WebVTT timestamp, `HH:MM:SS.mmm`. */
export function timestamp(seconds: number): string {
  const ms = Math.max(0, Math.round(seconds * 1000));
  const h = Math.floor(ms / 3_600_000);
  const m = Math.floor((ms % 3_600_000) / 60_000);
  const s = Math.floor((ms % 60_000) / 1000);
  const pad = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}.${pad(ms % 1000, 3)}`;
}

/** A WebVTT file for the cues. */
export function webvtt(list: Cue[]): string {
  const body = list
    .map((c, i) => `${i + 1}\n${timestamp(c.start)} --> ${timestamp(c.end)}\n${c.text}\n`)
    .join("\n");
  return `WEBVTT\n\n${body}`;
}
