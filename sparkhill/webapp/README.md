# sparkhill_webapp

A Dioxus web app replicating the functionality of the [blue.eel.education](https://blue.eel.education) interactive letter and word tracing practice application.

## About

Blue Eel is a letter/word tracing practice tool designed to help learners develop handwriting skills, with particular support for those with dyslexia or other reading difficulties. This Dioxus/Rust implementation replicates the core application:

- **Home page**: "Blue Eel" title, "Writing made simple!" subtitle, Begin button
- **Practice menu**: the ☰ button at the top left opens Letter Practice,
  Word Practice and Capital Practice, as the live app's left menu does
  (it has no capitals)
- **Settings panel**: the ⋮ button at the top right opens Student (the
  learner's name), Sequence (the letters or words to practise) and
  Technique, as on the live app (`src/settings.rs`,
  `src/components/settings_panel.rs`). Save keeps them in `localStorage`
  and starts the open level again with them. The live app's Upload
  Practice switch, which sends tracings to S3, is left out (SPRK-006)
- **Letter practice**: Canvas-based letter tracing with animated demonstrations and writing guidelines
- **Word practice**: Word-level tracing practice
- **Progress tracking**: Letter sequences matching the original app, with retry and stabilize logic
- **Levels**: a level ends once every batch is open and every expression in it is stable (traced right first time `STABILIZE_COUNT` times). A level finished with Tracing starts again with Pattern, then with Freeform (`Technique::next`, as on the live app, whose step from Pattern to Freeform never runs because of a bug); only after Freeform does "Go" open the next level, back with Tracing: the word level after letters, the capital letter level after words, and the congratulations page after capitals
- **Congratulations page**: Shown when all expressions are mastered
- **Each learner's progress**: the learner's name is typed as Student in the
  settings panel ("Standard" when blank, as in the live app), and `src/progress.rs` saves
  each level's progress under it in the browser's `localStorage` after every
  tracing, so a reload carries on where they were and each learner on a
  device has their own. Nothing leaves the device. `?practice=` sessions are
  never saved
- **Practising a few letters**: `?practice=` with letters or words, such as
  `/letter?practice=b,c`, `/word?practice=hi` or `/capital?practice=T`, practises only those, as one
  batch, instead of the whole level. Anything without a pattern is ignored.
  The settings panel's Sequence, such as `d, f, fun`, does the same on
  every visit: single lowercase letters on the letter level, capitals on
  the capital level and longer entries on the word level. A level with
  nothing in the sequence has its own groups, and a sequence's progress is
  saved apart from the whole level's
- **Techniques**: as on the live app, the settings panel's Technique picker (or
  `?tech=Tracing`, `?tech=Pattern` or `?tech=Freeform`, which wins) sets how
  much the demonstration shows (`src/technique.rs`). Tracing, the default,
  animates every stroke; Pattern shows only each stroke's start and end
  dots, which stay on the canvas while the learner draws; Freeform shows
  nothing. The choice is kept in `localStorage`, and moving on to the next
  technique after a level (see Levels) keeps that one

## Technical Details

### Canvas

The canvas component faithfully replicates blue.eel.education's `EelCanvas`:
- Writing guidelines at the correct ratios (cap, mean, base, beard lines)
- Color scheme: `#178CA4` user strokes, `#F9F7F0` background
- Letter and word demonstration animations from stroke patterns bundled in `src/patterns.json` (see "Letter patterns")

### Checking a tracing

When the learner presses ✓, the canvas hands its recording (the strokes in
canvas pixels and where its guide lines are) to `src/tracing_check.rs`, which
checks it against the letter's pattern. Both are scaled so the cap line to
the base line is 100 units and moved to the left edge, so a letter can be
drawn anywhere across the canvas but must sit on the right guide lines. It
passes when its size and height on the lines, the start and end of every
stroke (no stroke missing, none extra) and the shape of its ink are close to
the pattern's. The tolerances allow for a wobbly finger, and a test checks
that no letter passes as another. A miss says "Try again!"; after
`RETRY_LIMIT` misses the next letter comes up and the missed one returns
later.

### Letter Sequences

Uses the same letter groupings as the original app:
1. b, c, d, f
2. g, h, l, r, s, t
3. a, e, i, o, u
4. v, m, n, p
5. j, k, w
6. q, w, x, y, z

Capital letters (`/capital`) come after the words. The live app has a
recorded pattern for every capital, but its order isn't known, so they are
grouped from straight strokes to curves:
1. L, T, I, H, E, F
2. A, V, W, M, N
3. K, X, Y, Z
4. C, O, Q, G, S
5. D, P, B, R, J, U

### Letter patterns

The demonstrations draw from `src/patterns.json`, bundled into the app, so
the app fetches nothing at runtime and needs no outside host. There is one
pattern per letter, lowercase and capital, and a word joins its
letters' patterns.

They are the live app's own hand-recorded letters (Jim confirmed they are his
to publish, 2026-11-27): each is a copy of
`patterns/<letter>/master.json` from Blue Eel's `eel3-data` S3 bucket, made
once, with the guide lines from its `dimensions` (in pixels on a 449 pixel
high canvas) written as the `boundary`, points rounded to 0.1 pixel, and the
doubled end point the recorder left on some strokes removed. `type` marks
each stroke's `start` and `end`, and `src/patterns.rs` tests that every
practice letter and word has a pattern. To change a letter, edit its `path`
in the JSON. (Before that they were Hershey font approximations, PR #709.)

## Technology

Built with:
- [Dioxus](https://dioxuslabs.com/) - A React-like framework for Rust
- Dioxus Router for client-side navigation
- Canvas JavaScript interop for drawing
- Rust Edition 2024

## Building

To build the application:

```bash
cargo build
```

To run the development server:

```bash
dx serve
```

## Testing

To run Rust unit tests:

```bash
cargo test
```

The Playwright tests in `e2e/` drive the app in a browser: Begin, tracing a
letter right and a scribble ("Try again!"), the Pattern and Freeform
techniques (`e2e/technique.spec.ts`, which reads the canvas's pixels, and
"Go" from Tracing to Pattern to Freeform), and "Go" from the letter level to
the word level and on to the congratulations page. They keep each level short
with `?practice=`, and trace a letter by following the demonstration's own
path, which the canvas hands out as `window.eelCanvas.demonstrationPath()`.
They start `mise run dev` (dx serve on port 8080) themselves:

```bash
npm ci
npx playwright install --with-deps
npm test
```

`mise run test` runs the Rust tests and then these.

The tests tagged `@smoke` (Begin, tracing, every level, saved progress,
privacy and install) also run against the live site after every deploy, and
their report feeds the `cd` evals in `../evals.toml`. With `BASE_URL` set,
the tests start no dev server and record a video, screenshot and trace of
each test. To run them against production by hand:

```bash
BASE_URL=https://jimhillr-sparkhill-webapp.pages.dev npx playwright test --grep @smoke --project chromium
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```

## Deploying

`mise run release` bundles the app into `dist/public`, and
`../webapp_cloudflare` uploads that to Cloudflare Pages at
<https://jimhillr-sparkhill-webapp.pages.dev> on every push to `main` that
changes the app. `Dioxus.toml` and `index.html` give the page the live app's
title, description, theme colour and no-zoom viewport, and link the web app
manifest and icons in `public/` (dx copies them into the bundle's
`/assets/` as they are), so the
app installs to a phone's home screen and opens full screen like the live
one. The icons are drawn from `public/icons/icon.svg`.

## License

See LICENSE.md in the repository root.
