# Rust Design Patterns

- **Owner:** the rust-unofficial community.
- **Original:** <https://rust-unofficial.github.io/patterns/>
  ([source](https://github.com/rust-unofficial/patterns)).
- **Licence:** MPL-2.0. Summary only here.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- **Idioms:** borrowed types for arguments (`&str` over `&String`, `&[T]`
  over `&Vec<T>`), `Default`, constructors named `new`, `mem::take` to move
  out of a `&mut`.
- **Patterns:** newtype, RAII guards for cleanup, builder, and "compose
  structs together" to split borrows.
- **Anti-patterns:** `clone` to satisfy the borrow checker, `#![deny(warnings)]`
  in source (use CI flags instead), and `Deref` to fake inheritance.

## What we leave

The classic Gang of Four patterns translated to Rust (visitor, command,
strategy) unless the problem really calls for one; a plain enum and `match`
is usually simpler.

## Where it shows up

The `code-review` skill and the QUAL rules of the code quality process
(decision 0015).
