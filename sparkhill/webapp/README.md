# sparkhill_webapp

A Dioxus web app replicating the functionality of the [blue.eel.education](https://blue.eel.education) interactive letter and word tracing practice application.

## About

Blue Eel is a letter/word tracing practice tool designed to help learners develop handwriting skills, with particular support for those with dyslexia or other reading difficulties. This Dioxus/Rust implementation replicates the core application:

- **Home page**: "Blue Eel" title, "Writing made simple!" subtitle, Begin button
- **Letter practice**: Canvas-based letter tracing with animated demonstrations and writing guidelines
- **Word practice**: Word-level tracing practice
- **Progress tracking**: Letter sequences matching the original app, with retry and stabilize logic
- **Congratulations page**: Shown when all expressions are mastered

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

### Letter patterns

The demonstrations draw from `src/patterns.json`, bundled into the app, so
the app fetches nothing at runtime and needs no outside host (it used to load
them from Blue Eel's AWS S3 bucket). There is one pattern per lowercase
letter, and a word joins its letters' patterns.

The strokes come from the Hershey Simplex Roman font (`futural`), which is in
the public domain (Hershey fonts, Dr. A. V. Hershey, U.S. National Bureau of
Standards). They were converted once: each glyph's height is mapped onto the
canvas guide lines (ascender to the cap line, x-height to the mean line,
descender to the beard line), points are spaced about 8 units apart, and
strokes are put in handwriting order (bowl before stem in a, d, g and q, stem
before dot in i and j, and v, w and z drawn as one stroke). To change a
letter, edit its `path` in the JSON; `type` marks each stroke's `start` and
`end`, and `src/patterns.rs` tests that every practice letter and word has a
pattern.

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

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```

## License

See LICENSE.md in the repository root.
