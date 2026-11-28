import { test } from "node:test";
import assert from "node:assert/strict";
import { chunk, cues, timestamp, webvtt, wrap } from "./captions.ts";

test("short narration is one caption", () => {
  assert.deepEqual(chunk("Open the week."), ["Open the week."]);
});

test("sentences become separate captions", () => {
  assert.deepEqual(chunk("Open the week. Pick a recipe!"), ["Open the week.", "Pick a recipe!"]);
});

test("a long sentence splits at commas and keeps every word", () => {
  const text =
    "The shopping list adds up every ingredient for the week, groups the staples you probably already have, and works on a phone in the shop.";
  const parts = chunk(text);
  assert.ok(parts.length > 1);
  assert.ok(parts.every((p) => p.length <= 84), parts.join(" | "));
  assert.equal(parts.join(" "), text);
});

test("a long run with no punctuation splits at words", () => {
  const text = Array.from({ length: 40 }, (_, i) => `word${i}`).join(" ");
  const parts = chunk(text);
  assert.ok(parts.every((p) => p.length <= 84));
  assert.equal(parts.join(" "), text);
});

test("wrap balances two rows", () => {
  assert.equal(wrap("short"), "short");
  const rows = wrap("Every recipe opens on its own page with the steps in order.").split("\n");
  assert.equal(rows.length, 2);
  assert.ok(Math.abs(rows[0]!.length - rows[1]!.length) < 12);
});

test("cues share a line's time by length and end with the audio", () => {
  const list = cues([{ start: 2, duration: 4, text: "One two three. Four five six." }]);
  assert.equal(list.length, 2);
  assert.equal(list[0]!.start, 2);
  assert.equal(list[1]!.start, list[0]!.end);
  assert.ok(Math.abs(list[1]!.end - 6) < 1e-9);
});

test("timestamps are WebVTT", () => {
  assert.equal(timestamp(0), "00:00:00.000");
  assert.equal(timestamp(61.2345), "00:01:01.235");
  assert.equal(timestamp(3725.5), "01:02:05.500");
});

test("webvtt numbers its cues", () => {
  const vtt = webvtt([{ start: 0, end: 1.5, text: "Hello." }]);
  assert.equal(vtt, "WEBVTT\n\n1\n00:00:00.000 --> 00:00:01.500\nHello.\n");
});
