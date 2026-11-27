# Rust API Guidelines

- **Owner:** the Rust library team.
- **Original:** <https://rust-lang.github.io/api-guidelines/>, with the
  [checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
  ([source](https://github.com/rust-lang/api-guidelines)).
- **Licence:** MIT or Apache-2.0. Summary only here; the checklist is short
  enough to read at the source.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

Cite items by their ID in reviews and issue files.

- **Naming:** `C-CASE` (RFC 430 casing), `C-CONV` (`as_` is free and
  borrowed, `to_` is expensive, `into_` consumes), `C-GETTER` (no `get_`
  prefix on getters), `C-ITER` (`iter`, `iter_mut`, `into_iter`).
- **Traits:** `C-COMMON-TRAITS` (derive `Debug`, `Clone`, `PartialEq`,
  `Default` and friends where they make sense), `C-DEBUG` (every public type
  is `Debug`).
- **Errors:** `C-GOOD-ERR` (error types implement `std::error::Error`, are
  `Send + Sync`, and say what went wrong).
- **Types:** `C-NEWTYPE` (newtypes for distinct meanings of one primitive),
  `C-CUSTOM-TYPE` (an enum or struct instead of a `bool` argument),
  `C-BUILDER` (builders for types with many optional fields).
- **Docs:** `C-EXAMPLE` (public items have an example), `C-FAILURE`
  (`# Errors` and `# Panics` sections).

## What we leave

The crates.io publishing items (`C-METADATA`, `C-STABLE` and the rest of
"Necessities" for published crates), since we don't publish our crates.

## Where it shows up

The QUAL-3xx API rules of the code quality process (decision 0015), the
`code-review` skill, and Clippy's pedantic lints, several of which check
these items.
