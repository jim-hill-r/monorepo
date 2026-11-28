// The demo script format: one JSON file per demo, next to the deck that shows
// it (docs/reviews/<folder>/demos/<name>.json). The format is described in
// ../README.md.

/** One browser action. Each object has exactly one key. */
export type Action =
  | { goto: string }
  | { click: string }
  | { hover: string }
  | { type: { selector: string; text: string } }
  | { press: string }
  | { scroll: number }
  | { waitFor: string }
  | { wait: number };

/** One narrated step: the line is spoken while the actions run. */
export interface Step {
  say: string;
  actions?: Action[];
  /** Extra seconds to hold after the line ends, to let a page be seen. */
  hold?: number;
}

export interface Spec {
  title: string;
  /** The live app; DEMO_BASE_URL overrides it (a preview or local build). */
  url: string;
  viewport?: { width: number; height: number };
  /** A Kokoro voice name; see ../README.md. */
  voice?: string;
  speed?: number;
  /** Which step's last frame becomes the poster image (default 0). */
  poster?: number;
  steps: Step[];
}

const ACTION_KEYS = ["goto", "click", "hover", "type", "press", "scroll", "waitFor", "wait"];

/** Checks a parsed spec and returns the problems found, empty when valid. */
export function validate(value: unknown): string[] {
  const errors: string[] = [];
  if (typeof value !== "object" || value === null) return ["the spec must be a JSON object"];
  const spec = value as Record<string, unknown>;
  if (typeof spec.title !== "string" || spec.title === "") errors.push("title: a non-empty string");
  if (typeof spec.url !== "string" || !/^https?:\/\//.test(spec.url)) {
    errors.push("url: an http(s) URL");
  }
  if (!Array.isArray(spec.steps) || spec.steps.length === 0) {
    errors.push("steps: a non-empty list");
    return errors;
  }
  spec.steps.forEach((raw, i) => {
    const step = raw as Record<string, unknown>;
    if (typeof step.say !== "string" || step.say.trim() === "") {
      errors.push(`steps[${i}].say: the narration, a non-empty string`);
    }
    if (step.actions !== undefined && !Array.isArray(step.actions)) {
      errors.push(`steps[${i}].actions: a list`);
    }
    for (const [j, action] of ((step.actions as unknown[]) ?? []).entries()) {
      const keys = typeof action === "object" && action !== null ? Object.keys(action) : [];
      if (keys.length !== 1 || !ACTION_KEYS.includes(keys[0]!)) {
        errors.push(`steps[${i}].actions[${j}]: one of ${ACTION_KEYS.join(", ")}`);
      }
    }
  });
  if (
    spec.poster !== undefined &&
    (typeof spec.poster !== "number" || spec.poster < 0 || spec.poster >= spec.steps.length)
  ) {
    errors.push("poster: the index of a step");
  }
  return errors;
}

/** Resolves a `goto` target against the spec's (or the overridden) base URL. */
export function resolveUrl(base: string, target: string): string {
  return new URL(target, base.endsWith("/") ? base : `${base}/`).toString();
}
