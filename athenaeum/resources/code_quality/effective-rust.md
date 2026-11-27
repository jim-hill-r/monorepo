# Effective Rust

- **Owner:** David Drysdale.
- **Original:** <https://effective-rust.com/> (free online; also published by
  O'Reilly, 2024).
- **Licence:** copyright the author. Summary only here.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- **Use the type system to express your data** (items 1 and 2): make bad
  states unrepresentable with enums and newtypes, not runtime checks.
- **Prefer `Option` and `Result` transforms** (item 3) such as `map`,
  `ok_or`, `and_then` and `?` over explicit `match` chains and `unwrap`.
- **Error types** (item 4): a library returns a typed error enum; an
  application may use `anyhow`-style boxed errors.
- **Familiar traits and conversions** (items 5, 10): implement `From` rather
  than ad hoc conversion functions.
- **Don't clone to satisfy the borrow checker** (item 15): restructure
  ownership instead.
- **Listen to Clippy** (item 29) and **keep dependencies few** (items
  21 to 25).

## What we leave

The FFI, `unsafe` and `no_std` items, which almost no project here needs
(`unsafe` needs Jim's approval anyway).

## Where it shows up

[standards/docs/rust.md](../../../standards/docs/rust.md) error handling, and
the QUAL rules of the code quality process (decision 0015).
