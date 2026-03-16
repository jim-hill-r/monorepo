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
- Letter demonstration animations fetched from S3

### Letter Sequences

Uses the same letter groupings as the original app:
1. b, c, d, f
2. g, h, l, r, s, t
3. a, e, i, o, u
4. v, m, n, p
5. j, k, w
6. q, w, x, y, z

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
cast ci
```

## License

See LICENSE.md in the repository root.
