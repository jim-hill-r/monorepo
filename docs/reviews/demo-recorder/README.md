# Demo recorder

Records narrated demo videos of a live app, with closed captions, for the
[review decks](../README.md). A demo is a short script, so an agent can
change the words or the clicks and record it again, and the same script
gives the same narration every time. Why it works this way, and the options
it was chosen over, are in
[decision 0025](../../../standards/docs/decisions/0025-review-deck-demo-videos.md).

## How it works

1. **Narration.** Each step's `say` line is spoken by
   [Kokoro](https://github.com/thewh1teagle/kokoro-onnx), an open-weight
   text-to-speech model (Apache 2.0) that runs on the CPU in a couple of
   seconds a line. Nothing is sent to a service and nothing costs money.
   `tts/kokoro_tts.py` downloads its two model files (120 MB) from the
   kokoro-onnx GitHub release on first use and checks their SHA-256.
2. **Video.** Playwright opens the live app in Chromium at 1280x720 and
   records the screen while it runs each step's actions. Each step lasts as
   long as its spoken line (plus a short gap), so the picture and the voice
   stay together. A yellow dot shows the mouse, because headless recordings
   don't.
3. **Captions.** The captions are the script itself, timed from the audio,
   so there is no speech recognition to get words wrong. Long lines are
   split at sentences and commas into two-row captions.
4. **Assembly.** ffmpeg puts the lines on the timeline and writes an H.264
   MP4 with the narration and a soft caption track, a WebVTT file for the
   deck's `<video>` player, and a JPEG poster (which is what a PDF of the
   deck shows).

A one-minute demo is about 1 MB and takes about a minute to record.

## Writing a demo

Put one JSON file per demo in the deck's folder, as
`docs/reviews/<folder>/demos/<name>.json`:

```json
{
  "title": "Cookbook: find today's recipe and plan the week",
  "url": "https://jimhillr-cookbook.pages.dev",
  "poster": 1,
  "steps": [
    { "say": "This is the cookbook, live on Cloudflare Pages.", "hold": 0.5 },
    {
      "say": "Browse Recipes opens today's recipe.",
      "actions": [{ "click": ".recipe-card a:has-text(\"Browse Recipes\")" }]
    }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `title` | The video's title. |
| `url` | The live app. The recorder opens it before the first line. |
| `viewport` | `{ "width": 1280, "height": 720 }` by default. Use `{ "width": 390, "height": 844 }` for a phone. |
| `voice`, `speed` | A Kokoro voice (default `af_heart`; others include `af_bella`, `am_michael` and `bf_emma`) and speed (default 1.0). |
| `poster` | The step whose last frame is the poster (default 0). |
| `steps[].say` | The narration, which is also the captions. Write it for the ear: short sentences, no symbols or URLs. |
| `steps[].actions` | What happens while the line is spoken, in order. |
| `steps[].hold` | Extra seconds to stay on the step after the line. |

Actions, one key each: `goto` (a path or URL), `click`, `hover` (a
[Playwright selector](https://playwright.dev/docs/other-locators)), `type`
(`{ "selector": ..., "text": ... }`, typed a key at a time), `press` (a
key, such as `Enter`), `scroll` (pixels, negative for up), `waitFor` (a
selector) and `wait` (milliseconds). Reuse the selectors from the project's
own Playwright tests where you can.

Show the demo on a slide with:

```html
<video controls preload="metadata" width="800" poster="demos/<name>.jpg" src="demos/<name>.mp4"><track default kind="captions" srclang="en" label="English" src="demos/<name>.vtt"></video>
```

Pick the two or three journeys that matter most to the project's users and
keep each to about a minute.

## Recording

The videos are build output, like the deck's HTML and PDF, so they are not
committed. The `Review decks` workflow records the demos of every deck a pull
request changes and puts them in the `review-decks` artifact next to the
deck. To record again (after changing a demo, or to show the app as it is
now), push a change to the demo's JSON, or run the workflow by hand with
`demos` set to the deck folder.

Cloud sandboxes usually can't reach the live `*.pages.dev` sites, so let the
workflow record against production. To try a demo in a sandbox against a
site it can reach, or a local build:

```sh
cd docs/reviews/demo-recorder
npm ci
python3 -m venv .venv && .venv/bin/pip install -r tts/requirements.txt
DEMO_PYTHON=.venv/bin/python DEMO_BASE_URL=http://localhost:8080 \
  node src/record.ts --out /tmp/demos ../<folder>/demos/<name>.json
```

`DEMO_TTS=flite` uses ffmpeg's built-in robotic voice instead of Kokoro, for
a quick check with no Python. `PLAYWRIGHT_CHROMIUM_EXECUTABLE` points the
recorder at an installed Chromium. `npm test` type-checks the recorder and
runs the caption tests.
