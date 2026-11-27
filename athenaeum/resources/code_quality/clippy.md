# Clippy

- **Owner:** the Rust project.
- **Original:** the [lint list](https://rust-lang.github.io/rust-clippy/master/index.html)
  and the [Clippy book](https://doc.rust-lang.org/clippy/)
  ([source](https://github.com/rust-lang/rust-clippy)).
- **Licence:** MIT or Apache-2.0. Summary only here.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- Each lint's page says what it catches, **why it is bad** and a fixed
  example. Read that before allowing a lint, and put the reason in the
  `#[allow]` or `#[expect]` comment.
- Lint groups: `correctness`, `suspicious`, `style`, `complexity` and `perf`
  are on by default. `pedantic` is stricter and noisier; we adopt it per
  project through a ratchet rather than all at once.
- `clippy.toml` settings such as `cognitive-complexity-threshold`,
  `too-many-lines-threshold` and `too-many-arguments-threshold` set the
  complexity limits.

## What we leave

The `restriction` group as a whole: its lints are opinions for specific
codebases, so we pick single ones (`unwrap_used`, `expect_used`) instead.
The `nursery` group, which is unstable.

## Where it shows up

LIN-001 and LIN-002 in [standards/docs/rust.md](../../../standards/docs/rust.md),
and the pedantic ratchet of the code quality process (decision 0015).
