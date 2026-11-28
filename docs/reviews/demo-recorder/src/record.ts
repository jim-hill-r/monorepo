// Records narrated, captioned demo videos of a live app from demo scripts
// (src/spec.ts), for the review decks. See ../README.md.
//
//   node src/record.ts --out DIR SPEC.json [SPEC.json ...]
//
// For each SPEC it writes DIR/<name>.mp4 (H.264 video, AAC narration and a
// soft caption track), DIR/<name>.vtt (the captions for the deck's <video>)
// and DIR/<name>.jpg (the poster, which is also what a PDF of the deck shows).
//
// Environment:
//   DEMO_BASE_URL   record against this URL instead of the spec's `url`
//   DEMO_TTS        kokoro (default) or flite (robotic, but needs only ffmpeg)
//   DEMO_PYTHON     the Python with tts/requirements.txt installed (default python3)
//   DEMO_MODELS     where the Kokoro model files are kept (default ~/.cache/jimhillr-demos)
//   PLAYWRIGHT_CHROMIUM_EXECUTABLE  a Chromium to use instead of Playwright's own

import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Locator, type Page } from "playwright";
import { cues, webvtt, type Line } from "./captions.ts";
import { resolveUrl, validate, type Action, type Spec } from "./spec.ts";

/** Seconds between a step starting and its narration starting. */
const LEAD = 0.3;
/** Seconds of quiet after each line before the next step. */
const GAP = 0.6;
/** Seconds to hold on the last frame. */
const TAIL = 1.5;

const here = dirname(fileURLToPath(import.meta.url));

function run(cmd: string, args: string[], input?: string): string {
  return execFileSync(cmd, args, { input, encoding: "utf8", maxBuffer: 64 << 20 });
}

function seconds(file: string): number {
  const out = run("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", file]);
  return Number.parseFloat(out.trim());
}

/** Synthesizes each step's line to <work>/<i>.wav and returns the durations. */
function synthesize(spec: Spec, work: string): number[] {
  const engine = process.env.DEMO_TTS ?? "kokoro";
  const lines = spec.steps.map((s, i) => ({ id: String(i), text: s.say.replace(/\s+/g, " ").trim() }));
  if (engine === "flite") {
    return lines.map((l) => {
      const text = join(work, `${l.id}.txt`);
      const wav = join(work, `${l.id}.wav`);
      writeFileSync(text, l.text);
      run("ffmpeg", ["-v", "error", "-y", "-f", "lavfi", "-i", `flite=textfile=${text}:voice=slt`, wav]);
      return seconds(wav);
    });
  }
  if (engine !== "kokoro") throw new Error(`DEMO_TTS must be kokoro or flite, not ${engine}`);
  const out = run(
    process.env.DEMO_PYTHON ?? "python3",
    [
      "-I",
      join(here, "..", "tts", "kokoro_tts.py"),
      "--out",
      work,
      "--models",
      process.env.DEMO_MODELS ?? join(homedir(), ".cache", "jimhillr-demos"),
      "--voice",
      spec.voice ?? "af_heart",
      "--speed",
      String(spec.speed ?? 1.0),
    ],
    JSON.stringify(lines),
  );
  const durations = JSON.parse(out) as Record<string, number>;
  return lines.map((l) => durations[l.id]!);
}

// Headless recordings don't show the mouse, so draw one that follows it and
// pulses on click, so a viewer can see what is being pressed.
const CURSOR = `
  addEventListener("DOMContentLoaded", () => {
    const dot = document.createElement("div");
    dot.setAttribute("aria-hidden", "true");
    dot.style.cssText = "position:fixed;z-index:2147483647;pointer-events:none;width:22px;height:22px;" +
      "margin:-11px 0 0 -11px;border-radius:50%;background:rgba(255,200,0,.55);border:2px solid #222;" +
      "left:-50px;top:-50px;transition:transform .12s";
    document.body.appendChild(dot);
    addEventListener("mousemove", (e) => { dot.style.left = e.clientX + "px"; dot.style.top = e.clientY + "px"; }, true);
    addEventListener("mousedown", () => { dot.style.transform = "scale(.6)"; }, true);
    addEventListener("mouseup", () => { dot.style.transform = ""; }, true);
  });`;

async function pointAt(page: Page, target: Locator): Promise<void> {
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  if (box) await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, { steps: 25 });
}

async function act(page: Page, base: string, action: Action): Promise<void> {
  if ("goto" in action) {
    await page.goto(resolveUrl(base, action.goto), { waitUntil: "networkidle" });
  } else if ("click" in action) {
    const target = page.locator(action.click).first();
    await pointAt(page, target);
    await target.click();
    await page.waitForLoadState("networkidle");
  } else if ("hover" in action) {
    await pointAt(page, page.locator(action.hover).first());
  } else if ("type" in action) {
    const target = page.locator(action.type.selector).first();
    await pointAt(page, target);
    await target.click();
    await target.pressSequentially(action.type.text, { delay: 70 });
  } else if ("press" in action) {
    await page.keyboard.press(action.press);
  } else if ("scroll" in action) {
    const steps = 20;
    for (let i = 0; i < steps; i++) {
      await page.mouse.wheel(0, action.scroll / steps);
      await page.waitForTimeout(25);
    }
  } else if ("waitFor" in action) {
    await page.locator(action.waitFor).first().waitFor();
  } else {
    await page.waitForTimeout(action.wait);
  }
}

async function record(specPath: string, outDir: string): Promise<void> {
  const parsed: unknown = JSON.parse(readFileSync(specPath, "utf8"));
  const errors = validate(parsed);
  if (errors.length > 0) throw new Error(`${specPath}:\n  ${errors.join("\n  ")}`);
  const spec = parsed as Spec;
  const name = basename(specPath, ".json");
  const base = process.env.DEMO_BASE_URL ?? spec.url;
  const viewport = spec.viewport ?? { width: 1280, height: 720 };
  const work = mkdtempSync(join(tmpdir(), `demo-${name}-`));

  const durations = synthesize(spec, work);

  const browser = await chromium.launch({
    executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || undefined,
  });
  const defaultUserAgent = async () => {
    const probe = await browser.newPage();
    const agent = await probe.evaluate(() => navigator.userAgent);
    await probe.close();
    return agent.replace("HeadlessChrome", "Chrome");
  };
  const context = await browser.newContext({
    viewport,
    recordVideo: { dir: work, size: viewport },
    // Synthetic traffic is tagged, so analytics can leave it out
    // (standards/docs/security.md).
    userAgent: `${await defaultUserAgent()} jimhillr-demos`,
    extraHTTPHeaders: { "x-jimhillr-synthetic": "1" },
  });
  await context.addInitScript(CURSOR);
  const page = await context.newPage();
  const t0 = Date.now();
  const now = () => (Date.now() - t0) / 1000;

  // Load the app before the first line, then cut the blank start.
  await page.goto(base, { waitUntil: "networkidle" });
  const trim = now();

  const lines: Line[] = [];
  const stepEnds: number[] = [];
  for (const [i, step] of spec.steps.entries()) {
    const start = now();
    lines.push({ start: start + LEAD - trim, duration: durations[i]!, text: step.say });
    for (const action of step.actions ?? []) await act(page, base, action);
    const left = LEAD + durations[i]! + GAP + (step.hold ?? 0) - (now() - start);
    if (left > 0) await page.waitForTimeout(left * 1000);
    stepEnds.push(now() - trim);
  }
  await page.waitForTimeout(TAIL * 1000);
  const video = page.video();
  await context.close();
  await browser.close();
  if (!video) throw new Error("Playwright recorded no video");
  const webm = await video.path();

  mkdirSync(outDir, { recursive: true });
  const vtt = join(outDir, `${name}.vtt`);
  writeFileSync(vtt, webvtt(cues(lines)));

  const inputs = ["-ss", trim.toFixed(3), "-i", webm];
  const filters: string[] = [];
  lines.forEach((line, i) => {
    inputs.push("-i", join(work, `${i}.wav`));
    const ms = Math.round(line.start * 1000);
    filters.push(`[${i + 1}:a]adelay=${ms}:all=1[a${i}]`);
  });
  const mix = `${lines.map((_, i) => `[a${i}]`).join("")}amix=inputs=${lines.length}:normalize=0,apad[aout]`;
  inputs.push("-i", vtt);
  const mp4 = join(outDir, `${name}.mp4`);
  const tmp = join(work, `${name}.mp4`);
  run("ffmpeg", [
    "-v", "error", "-y", ...inputs,
    "-filter_complex", [...filters, mix].join(";"),
    "-map", "0:v", "-map", "[aout]", "-map", `${lines.length + 1}:s`,
    "-c:v", "libx264", "-preset", "slow", "-crf", "30", "-tune", "stillimage", "-pix_fmt", "yuv420p",
    "-r", "25", "-c:a", "aac", "-b:a", "64k", "-ac", "1",
    "-c:s", "mov_text", "-metadata:s:s:0", "language=eng", "-metadata:s:s:0", "title=English captions",
    "-metadata", `title=${spec.title}`, "-shortest", "-movflags", "+faststart", tmp,
  ]);
  renameSync(tmp, mp4);

  const posterAt = Math.max(0, (stepEnds[spec.poster ?? 0] ?? 0) - 0.2);
  run("ffmpeg", [
    "-v", "error", "-y", "-ss", posterAt.toFixed(3), "-i", mp4, "-frames:v", "1", "-q:v", "4",
    join(outDir, `${name}.jpg`),
  ]);
  console.log(`${mp4}: ${seconds(mp4).toFixed(1)} s, ${spec.steps.length} lines`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const at = args.indexOf("--out");
  if (at < 0 || args.length < at + 2) {
    console.error("usage: node src/record.ts --out DIR SPEC.json [SPEC.json ...]");
    process.exit(2);
  }
  const out = resolve(args[at + 1]!);
  const specs = args.filter((_, i) => i !== at && i !== at + 1);
  for (const spec of specs) await record(resolve(spec), out);
}

await main();
